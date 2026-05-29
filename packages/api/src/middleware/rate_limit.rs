use std::sync::Arc;

use axum::{
    body::Body,
    extract::{MatchedPath, State},
    http::{HeaderValue, Request, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use redis_store::{
    r_types::AppError,
    rate_limit::{
        Policy, RedisLimiter,
        limiter::{Limiter, algo_tag, keys_for},
    },
};

use crate::{APIContext, middleware::client_ip::extract_client_ip};

/// State passed into the rate-limit middleware. Cheap to clone — both fields
/// are `Arc`.
#[derive(Clone)]
pub struct RateLimitState {
    pub ctx: Arc<APIContext>,
    pub policy: Policy,
    /// IPs of trusted reverse proxies (HAProxy, load balancer). The middleware
    /// uses these to identify the *real* client IP from `X-Forwarded-For`.
    pub trusted_proxies: Arc<Vec<std::net::IpAddr>>,
}

impl RateLimitState {
    pub fn new(ctx: Arc<APIContext>, policy: Policy) -> Self {
        Self {
            ctx,
            policy,
            trusted_proxies: Arc::new(Vec::new()),
        }
    }

    pub fn with_trusted_proxies(
        mut self,
        proxies: Vec<std::net::IpAddr>,
    ) -> Self {
        self.trusted_proxies = Arc::new(proxies);
        self
    }
}

/// Axum middleware that checks the request against `state.policy` before
/// passing it on. Designed to be wrapped per-route via
/// `axum::middleware::from_fn_with_state`.
///
/// On allow: passes through, and attaches `X-RateLimit-*` headers to the
/// downstream response.
/// On deny: returns `429 Too Many Requests` with `Retry-After` and JSON body.
/// On Redis error: applies `policy.fail_mode` (Open → pass; Closed → 429).
pub async fn rate_limit(
    State(state): State<RateLimitState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    // 1. Resolve route template (or fall back to literal path — slightly
    //    higher cardinality but still bounded by route count).
    let route = req
        .extensions()
        .get::<MatchedPath>()
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| req.uri().path().to_string());
    let normalized = redis_store::rate_limit::normalize_route(&route);

    // 2. Auth middleware (when present) inserts the user id as
    //    `Extension<String>`. We tolerate either presence or absence —
    //    `keys_for` falls back to IP-only when no user id is available.
    let user_id = req.extensions().get::<String>().map(|s| s.to_string());

    let ip = extract_client_ip(req.headers(), &state.trusted_proxies);
    if ip.is_empty() && user_id.is_none() {
        // No identifier at all — refuse rather than create a global bucket.
        // This only happens if both auth and the proxy chain are misconfigured.
        return AppError::ValidationError(
            "no client identifier available".into(),
        )
        .into_response();
    }

    let limiter = RedisLimiter::new(&state.ctx.redis);
    let policy = &state.policy;
    let atag = algo_tag(policy.algo);

    // 3. Build one or two keys depending on scope, check each.
    //    A request is admitted only if every key admits it.
    let scopes = keys_for(policy.scope, user_id.as_deref(), &ip);
    let mut final_decision: Option<redis_store::rate_limit::Decision> = None;

    for (scope_tag, identifier) in &scopes {
        let key = redis_store::rate_limit::build_key(
            atag,
            scope_tag,
            identifier,
            &normalized,
        );
        match limiter.check(&key, policy).await {
            Ok(d) if !d.allowed => {
                // Reject immediately on the first denying key — no need to
                // touch the second bucket once we know we're returning 429.
                return reject(&key, d, policy.capacity);
            }
            Ok(d) => final_decision = Some(d),
            Err(AppError::TooManyRequests {
                key,
                retry_after_seconds,
            }) => {
                // fail_mode=Closed surfaced as TooManyRequests
                return reject_synthetic(
                    &key,
                    retry_after_seconds,
                    policy.capacity,
                );
            }
            Err(e) => return e.into_response(),
        }
    }

    // 4. All buckets admitted — run the inner handler and stamp headers.
    let mut response = next.run(req).await;
    if let Some(d) = final_decision {
        attach_headers(&mut response, &d, policy.capacity);
    }
    response
}

fn reject(
    key: &str,
    d: redis_store::rate_limit::Decision,
    capacity: u32,
) -> Response {
    let retry = d.retry_after_secs();
    let body = format!(
        "{{\"success\":false,\"error\":\"rate_limit_exceeded\",\"retry_after_seconds\":{}}}",
        retry
    );
    let mut resp = (StatusCode::TOO_MANY_REQUESTS, body).into_response();
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    if let Ok(v) = HeaderValue::from_str(&retry.to_string()) {
        resp.headers_mut().insert("Retry-After", v);
    }
    set_rate_headers(&mut resp, capacity, 0, d.reset_ms);
    tracing::warn!(key = %key, retry_after = retry, "rate_limit: rejected");
    resp
}

fn reject_synthetic(
    key: &str,
    retry_after_secs: u64,
    capacity: u32,
) -> Response {
    let body = format!(
        "{{\"success\":false,\"error\":\"rate_limit_exceeded\",\"retry_after_seconds\":{}}}",
        retry_after_secs
    );
    let mut resp = (StatusCode::TOO_MANY_REQUESTS, body).into_response();
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    if let Ok(v) = HeaderValue::from_str(&retry_after_secs.to_string()) {
        resp.headers_mut().insert("Retry-After", v);
    }
    set_rate_headers(&mut resp, capacity, 0, 0);
    tracing::warn!(
        key = %key,
        retry_after = retry_after_secs,
        "rate_limit: rejected (fail-closed synthetic)"
    );
    resp
}

fn attach_headers(
    resp: &mut Response,
    d: &redis_store::rate_limit::Decision,
    capacity: u32,
) {
    set_rate_headers(resp, capacity, d.remaining, d.reset_ms);
}

fn set_rate_headers(
    resp: &mut Response,
    limit: u32,
    remaining: u64,
    reset_ms: u64,
) {
    let h = resp.headers_mut();
    if let Ok(v) = HeaderValue::from_str(&limit.to_string()) {
        h.insert("X-RateLimit-Limit", v);
    }
    if let Ok(v) = HeaderValue::from_str(&remaining.to_string()) {
        h.insert("X-RateLimit-Remaining", v);
    }
    let reset_secs = reset_ms / 1000;
    if reset_secs > 0
        && let Ok(v) = HeaderValue::from_str(&reset_secs.to_string())
    {
        h.insert("X-RateLimit-Reset", v);
    }
}
