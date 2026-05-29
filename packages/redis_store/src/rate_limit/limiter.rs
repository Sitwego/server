use std::time::Duration;

use fred::{
    prelude::{LuaInterface, Pool},
    types::{Key as RedisKey, Value as RedisValue},
};

use crate::{
    RedisConnectionPool,
    r_types::AppError,
    rate_limit::{
        policy::{Algo, FailMode, KeyScope, Policy},
        scripts,
    },
};

/// What the limiter decided. Carries the metadata the middleware needs to
/// set `X-RateLimit-*` headers and 429 bodies without a second round trip.
#[derive(Debug, Clone, Copy)]
pub struct Decision {
    pub allowed: bool,
    pub remaining: u64,
    /// Milliseconds the caller should wait before retrying. `0` when allowed.
    pub retry_after_ms: u64,
    /// Unix-ms timestamp when the bucket is fully replenished. Used for
    /// `X-RateLimit-Reset`.
    pub reset_ms: u64,
}

impl Decision {
    pub fn retry_after_secs(&self) -> u64 {
        // ceil(retry_after_ms / 1000), min 1 if not allowed
        let s = self.retry_after_ms.div_ceil(1000);
        if !self.allowed && s == 0 { 1 } else { s }
    }
}

/// Limiter trait. All three algorithms implement this so callers can swap
/// based on `Policy.algo` without changing the call site.
pub trait Limiter {
    /// Check whether `n=cost` requests can be admitted to the bucket at `key`,
    /// and (if so) record them atomically.
    fn check(
        &self,
        key: &str,
        policy: &Policy,
    ) -> impl std::future::Future<Output = Result<Decision, AppError>> + Send;
}

/// Redis-backed limiter. Each `check()` call is exactly one Redis EVAL,
/// so the entire admission decision is atomic per Redis slot.
#[derive(Clone)]
pub struct RedisLimiter {
    pool: Pool,
    /// Hard timeout for the Redis call. Past this we apply `policy.fail_mode`
    /// rather than holding up the request path.
    timeout: Duration,
}

impl RedisLimiter {
    pub fn new(pool: &RedisConnectionPool) -> Self {
        Self {
            pool: pool.pool.clone(),
            timeout: Duration::from_millis(50),
        }
    }

    /// Override the per-call Redis timeout. Defaults to 50ms.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Execute one of the Lua scripts and decode its 4-element response.
    /// All scripts return `{ allowed, remaining, retry_after_ms, reset_ms }`.
    async fn eval_script(
        &self,
        script: &'static str,
        key: &str,
        args: Vec<RedisValue>,
    ) -> Result<Decision, AppError> {
        let keys: Vec<RedisKey> = vec![key.into()];

        let fut = self.pool.eval::<Vec<i64>, _, _, _>(script, keys, args);

        let raw = tokio::time::timeout(self.timeout, fut)
            .await
            .map_err(|_| {
                AppError::InternalError("rate_limit: redis timeout".into())
            })?
            .map_err(|e| {
                AppError::InternalError(format!("rate_limit: redis: {e}"))
            })?;

        if raw.len() < 4 {
            return Err(AppError::InternalError(
                "rate_limit: malformed script response".into(),
            ));
        }

        Ok(Decision {
            allowed: raw[0] == 1,
            remaining: raw[1].max(0) as u64,
            retry_after_ms: raw[2].max(0) as u64,
            reset_ms: raw[3].max(0) as u64,
        })
    }

    /// Apply the fail-mode policy to a Redis error. For `Open` we synthesize
    /// an allow-decision and let the request through. For `Closed` we bubble
    /// the error up as a 429-equivalent.
    fn apply_fail_mode(
        err: AppError,
        policy: &Policy,
        key: &str,
    ) -> Result<Decision, AppError> {
        tracing::warn!(
            error = %err,
            key = %key,
            algo = ?policy.algo,
            "rate_limit: limiter call failed; applying fail_mode",
        );
        match policy.fail_mode {
            FailMode::Open => Ok(Decision {
                allowed: true,
                remaining: policy.capacity as u64,
                retry_after_ms: 0,
                reset_ms: 0,
            }),
            FailMode::Closed => Err(AppError::TooManyRequests {
                key: key.to_string(),
                // Conservative: ask client to wait one window
                retry_after_seconds: policy.window.as_secs().max(1),
            }),
        }
    }
}

impl Limiter for RedisLimiter {
    async fn check(
        &self,
        key: &str,
        policy: &Policy,
    ) -> Result<Decision, AppError> {
        let ttl_secs = (policy.window.as_secs() * 2).max(1);
        let window_ms = policy.window.as_millis() as i64;

        let result = match policy.algo {
            Algo::TokenBucket => {
                // refill_per_sec = capacity / window_secs; multiplied by 1000
                // so we pass integers to Lua and avoid float drift in args.
                let window_secs = policy.window.as_secs_f64().max(0.001);
                let refill_per_sec_x1k =
                    ((policy.capacity as f64 / window_secs) * 1000.0) as i64;

                let args: Vec<RedisValue> = vec![
                    (policy.capacity as i64).into(),
                    refill_per_sec_x1k.into(),
                    (policy.cost as i64).into(),
                    (ttl_secs as i64).into(),
                ];
                self.eval_script(scripts::TOKEN_BUCKET, key, args).await
            }
            Algo::SlidingWindowLog => {
                let args: Vec<RedisValue> = vec![
                    window_ms.into(),
                    (policy.capacity as i64).into(),
                    (ttl_secs as i64).into(),
                ];
                self.eval_script(scripts::SLIDING_WINDOW_LOG, key, args).await
            }
            Algo::SlidingWindowCounter => {
                let args: Vec<RedisValue> = vec![
                    window_ms.into(),
                    (policy.capacity as i64).into(),
                    (ttl_secs as i64).into(),
                ];
                self.eval_script(scripts::SLIDING_WINDOW_COUNTER, key, args)
                    .await
            }
        };

        match result {
            Ok(d) => Ok(d),
            Err(e) => Self::apply_fail_mode(e, policy, key),
        }
    }
}

/// Helper for the middleware: turn a `(scope, user, ip, route)` quadruple
/// into the list of (algo_tag, scope_tag, identifier) tuples to check.
///
/// Returns `Vec` because `UserAndIp` produces two checks.
pub fn keys_for(
    scope: KeyScope,
    user_id: Option<&str>,
    ip: &str,
) -> Vec<(&'static str, String)> {
    match scope {
        KeyScope::Ip => vec![("ip", ip.to_string())],
        KeyScope::User => match user_id {
            Some(u) => vec![("user", u.to_string())],
            // No user id available: fall back to IP so anonymous traffic is
            // still bounded.
            None => vec![("ip", ip.to_string())],
        },
        KeyScope::UserAndIp => {
            let mut v = Vec::with_capacity(2);
            if let Some(u) = user_id {
                v.push(("user", u.to_string()));
            }
            v.push(("ip", ip.to_string()));
            v
        }
    }
}

pub fn algo_tag(algo: Algo) -> &'static str {
    match algo {
        Algo::TokenBucket => "tb",
        Algo::SlidingWindowLog => "swl",
        Algo::SlidingWindowCounter => "swc",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_after_secs_rounds_up_and_clamps_min_one() {
        let d = Decision {
            allowed: false,
            remaining: 0,
            retry_after_ms: 0,
            reset_ms: 0,
        };
        assert_eq!(d.retry_after_secs(), 1);

        let d = Decision {
            allowed: false,
            remaining: 0,
            retry_after_ms: 1_500,
            reset_ms: 0,
        };
        assert_eq!(d.retry_after_secs(), 2);

        let d = Decision {
            allowed: true,
            remaining: 5,
            retry_after_ms: 0,
            reset_ms: 0,
        };
        assert_eq!(d.retry_after_secs(), 0);
    }

    #[test]
    fn keys_for_scope_ip_returns_one_key() {
        let k = keys_for(KeyScope::Ip, Some("u1"), "1.2.3.4");
        assert_eq!(k.len(), 1);
        assert_eq!(k[0].0, "ip");
    }

    #[test]
    fn keys_for_scope_user_falls_back_to_ip_when_anonymous() {
        let k = keys_for(KeyScope::User, None, "1.2.3.4");
        assert_eq!(k.len(), 1);
        assert_eq!(k[0].0, "ip");
    }

    #[test]
    fn keys_for_scope_user_and_ip_yields_both_when_authenticated() {
        let k = keys_for(KeyScope::UserAndIp, Some("u1"), "1.2.3.4");
        assert_eq!(k.len(), 2);
        assert_eq!(k[0].0, "user");
        assert_eq!(k[1].0, "ip");
    }
}
