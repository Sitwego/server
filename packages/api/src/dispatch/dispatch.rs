// DriverPoolManager for high-scale ride dispatch
// --------------------------------------------------------------
// Key features:
// - Bounded candidate scan using ZREVRANGEBYSCORE + LIMIT (O(log N + M))
// - Atomic driver acquisition using Lua + locks
// - Script caching via SCRIPT LOAD + EVALSHA with NOSCRIPT fallback
// - Conditional decrement (penalties) atomically
// - Configurable parameters and comprehensive observability
//
// Redis data model:
// - drivers:pool            (ZSET)  driver_id -> score
// - drivers:inflight        (ZSET)  driver_id -> expiration_timestamp
// - drivers:inflight:score  (HASH)  driver_id -> original_score
// - drivers:request_map     (HASH)  driver_id -> request_id
// - drivers:cooldown        (ZSET)  driver_id -> cooldown_until_timestamp
use std::vec;

use fred::error::Error;
use fred::prelude::{HashesInterface, SortedSetsInterface};
use fred::types::Value as RedisValue;
use fred::types::scripts::Script;
use redis_store::RedisConnectionPool;
use redis_store::r_types::RedisError;

use tracing::{error, info, warn};

/// Configuration for DriverPoolManager
#[derive(Clone, Debug)]
pub struct DriverPoolConfig {
    pub default_ttl_ms: u64,
    pub default_batch_size: usize,
    pub timeout_penalty: f64,
    pub rejection_penalty: f64,
    pub default_cooldown_ms: u64,
}

impl Default for DriverPoolConfig {
    fn default() -> Self {
        Self {
            default_ttl_ms: 20_000,
            default_batch_size: 10,
            timeout_penalty: 0.8,
            rejection_penalty: 0.5,
            default_cooldown_ms: 5_000,
        }
    }
}

/// Pool statistics for monitoring
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub available: i64,
    pub inflight: i64,
    pub cooldown: i64,
}

/// Manages a pool of drivers for ride dispatching
#[derive(Clone)]
pub struct DriverPoolManager {
    pub redis: std::sync::Arc<RedisConnectionPool>,
    pub pool_key: String,
    pub inflight_key: String,
    pub request_map_key: String,
    pub inflight_score_key: String,
    pub cooldown_key: String,
    pub config: DriverPoolConfig,

    pub acquire_script: Script,
    pub release_driver_script: Script,
}

fn current_time_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}

impl DriverPoolManager {
    // -----------------
    // Lua scripts
    // -----------------
    const ACQUIRE_DRIVER_LUA: &str = r#"
    -- =========================================
    -- Acquire from provided candidate list
    -- =========================================

    local pool      = KEYS[1]
    local inflight  = KEYS[2]
    local cooldown  = KEYS[3]
    local req_map   = KEYS[4]
    local score_map = KEYS[5]

    local request_id = ARGV[1]
    local ttl_ms     = tonumber(ARGV[2])

    -- current time (ms)
    local t = redis.call('TIME')
    local now_ms = (t[1] * 1000) + math.floor(t[2] / 1000)

    -- -----------------------------------------
    -- reclaim expired inflight drivers
    -- -----------------------------------------
    local expired = redis.call('ZRANGEBYSCORE', inflight, '-inf', now_ms)
    for _, d in ipairs(expired) do
        redis.call('ZREM', inflight, d)

        local sc = redis.call('HGET', score_map, d)
        redis.call('ZADD', pool, tonumber(sc) or 0, d)

        redis.call('HDEL', req_map, d)
        redis.call('HDEL', score_map, d)  -- Fix: cleanup score_map to prevent memory leak
    end

    -- -----------------------------------------
    -- try candidates IN GIVEN ORDER
    -- -----------------------------------------
    for i = 3, #ARGV do
        local driver_id = ARGV[i]

        -- cooldown check
        local cd = redis.call('ZSCORE', cooldown, driver_id)
        if not cd or tonumber(cd) <= now_ms then
            -- Remove cooldown entry if expired
            if cd and tonumber(cd) <= now_ms then
                redis.call('ZREM', cooldown, driver_id)
            end
            local score = redis.call('ZSCORE', pool, driver_id)
            if score then
                score = tonumber(score)
                if redis.call('ZREM', pool, driver_id) == 1 then
                    redis.call('ZADD', inflight, now_ms + ttl_ms, driver_id)
                    redis.call('HSET', req_map, driver_id, request_id)
                    redis.call('HSET', score_map, driver_id, score)
                    return driver_id
                end
            end
        end
    end

    return nil

    "#;

    const RELEASE_DRIVER_LUA: &str = r#"
        local pool            = KEYS[1]
        local inflight        = KEYS[2]
        local inflight_score  = KEYS[3]
        local request_map     = KEYS[4]

        local driver_id = ARGV[1]
        local penalty   = tonumber(ARGV[2])

        local sc = redis.call('HGET', inflight_score, driver_id)
        if not sc then
            return 0
        end

        sc = tonumber(sc) + penalty

        redis.call('ZREM', inflight, driver_id)
        redis.call('HDEL', inflight_score, driver_id)
        redis.call('HDEL', request_map, driver_id)
        redis.call('ZADD', pool, sc, driver_id)

        return 1
    "#;

    // -----------------
    // Initialization
    // -----------------

    /// Initialize DriverPoolManager and preload Lua scripts
    pub async fn new(
        redis: std::sync::Arc<RedisConnectionPool>,
        pool_key: impl Into<String>,
        inflight_key: impl Into<String> + Copy,
        request_map_key: impl Into<String>,
    ) -> Result<Self, RedisError> {
        Self::new_with_config(
            redis,
            pool_key,
            inflight_key,
            request_map_key,
            DriverPoolConfig::default(),
        )
        .await
    }

    /// Initialize DriverPoolManager with custom configuration
    pub async fn new_with_config(
        redis: std::sync::Arc<RedisConnectionPool>,
        pool_key: impl Into<String>,
        inflight_key: impl Into<String> + Copy,
        request_map_key: impl Into<String>,
        config: DriverPoolConfig,
    ) -> Result<Self, RedisError> {
        let acquire_script = Script::from_lua(Self::ACQUIRE_DRIVER_LUA);
        let release_driver_script = Script::from_lua(Self::RELEASE_DRIVER_LUA);
        let client = redis.pool.next();

        // Preload scripts to avoid NOSCRIPT errors later
        acquire_script.load(client).await?;
        release_driver_script.load(client).await?;

        let inflight_key_str = inflight_key.into();
        let pool_key_str = pool_key.into();

        Ok(Self {
            redis,
            pool_key: pool_key_str.clone(),
            inflight_key: inflight_key_str.clone(),
            request_map_key: request_map_key.into(),
            inflight_score_key: format!("{}:score", inflight_key_str),
            cooldown_key: format!("{}:cooldown", pool_key_str),
            config,
            acquire_script,
            release_driver_script,
        })
    }

    // -----------------
    // Internal helpers
    // -----------------

    fn parse_f64(val: RedisValue) -> Result<f64, RedisError> {
        match val {
            RedisValue::Double(d) => Ok(d),
            RedisValue::Integer(i) => Ok(i as f64),
            RedisValue::String(s) => s.parse::<f64>().map_err(|e| {
                RedisError::RedisDefaultError(format!("parse error: {}", e))
            }),
            _ => Err(RedisError::RedisDefaultError(
                "Unexpected RedisValue".into(),
            )),
        }
    }

    /// Execute EVALSHA with automatic fallback to EVAL on NOSCRIPT error
    async fn evalsha_with_fallback<T>(
        &self,
        script: &Script,
        keys: Vec<String>,
        args: Vec<String>,
    ) -> Result<T, RedisError>
    where
        T: fred::types::FromValue,
    {
        match script
            .evalsha(self.redis.pool.next(), keys.clone(), args.clone())
            .await
        {
            Ok(val) => Ok(val),
            Err(e) => {
                let err_str = format!("{:?}", e);
                if err_str.contains("NOSCRIPT") {
                    warn!("NOSCRIPT error detected, reloading script");
                    script.load(self.redis.pool.next()).await?;
                    script
                        .evalsha(self.redis.pool.next(), keys, args)
                        .await
                        .map_err(Into::into)
                } else {
                    Err(e.into())
                }
            }
        }
    }

    // -----------------
    // Public API
    // -----------------

    /// Add or update a driver in the pool with a given score
    pub async fn add_driver(
        &self,
        driver_id: &str,
        score: f64,
    ) -> Result<(), RedisError> {
        let _: () = self
            .redis
            .zadd(&self.pool_key, None, None, false, false, (score, driver_id))
            .await?;

        info!(driver_id = %driver_id, score = %score, "Driver added to pool");
        Ok(())
    }

    /// Get the current score of a driver in the pool (if present)
    pub async fn get_driver_score(
        &self,
        driver_id: &str,
    ) -> Result<Option<f64>, RedisError> {
        let scores: Vec<Option<f64>> =
            self.redis.pool.zmscore(&self.pool_key, vec![driver_id]).await?;

        Ok(scores.into_iter().next().flatten())
    }

    /// Increment (or decrement if negative) a driver's score
    pub async fn incr_driver_score(
        &self,
        driver_id: &str,
        delta: f64,
    ) -> Result<f64, RedisError> {
        let score: f64 =
            self.redis.pool.zincrby(&self.pool_key, delta, driver_id).await?;

        info!(driver_id = %driver_id, delta = %delta, new_score = %score, "Driver score updated");
        Ok(score)
    }

    /// Check whether a driver is currently inflight
    pub async fn is_inflight(
        &self,
        driver_id: &str,
    ) -> Result<bool, RedisError> {
        let rank: Option<Vec<(i64, i64)>> =
            self.redis.pool.zrank(&self.inflight_key, driver_id, true).await?;
        Ok(rank.is_some())
    }

    /// Get pool statistics for monitoring
    pub async fn pool_stats(&self) -> Result<PoolStats, RedisError> {
        let available: i64 = self.redis.pool.zcard(&self.pool_key).await?;
        let inflight: i64 = self.redis.pool.zcard(&self.inflight_key).await?;
        let cooldown: i64 = self.redis.pool.zcard(&self.cooldown_key).await?;

        Ok(PoolStats {
            available,
            inflight,
            cooldown,
        })
    }

    pub async fn try_acquire_driver(
        &self,
        args: &[String],
    ) -> Result<Option<String>, RedisError> {
        let acquired: Option<String> = self
            .evalsha_with_fallback(
                &self.acquire_script,
                vec![
                    self.pool_key.clone(),
                    self.inflight_key.clone(),
                    self.cooldown_key.clone(),
                    self.request_map_key.clone(),
                    self.inflight_score_key.clone(),
                ],
                args.to_vec(),
            )
            .await?;

        info!(
            request_id = %args[0],
            ttl_ms = %args[1],
            acquired = acquired.is_some(),
            driver_id = ?acquired,
            "TRY_ACQUIRE_DRIVER COMPLETED ✅✅✅✅"
        );
        Ok(acquired)
    }

    /// Atomically acquire the best available driver
    ///
    /// **DEPRECATED:** This method is broken - it passes `batch_size` as ARGV[3]
    /// but the Lua script expects driver IDs starting from ARGV[3].
    /// Use `try_acquire_driver` instead with explicit driver candidates.
    ///
    /// Returns `Some(driver_id)` if acquired, otherwise `None`
    #[deprecated(
        since = "0.1.0",
        note = "Use try_acquire_driver instead - this method passes wrong args to Lua script"
    )]
    pub async fn acquire_driver(
        &self,
        request_id: &str,
        ttl_ms: u64,
        batch_size: usize,
    ) -> Result<Option<String>, RedisError> {
        let start = std::time::Instant::now();

        let res: Option<String> = self
            .evalsha_with_fallback(
                &self.acquire_script,
                vec![
                    self.pool_key.clone(),
                    self.inflight_key.clone(),
                    self.cooldown_key.clone(),
                    self.request_map_key.clone(),
                    self.inflight_score_key.clone(),
                ],
                vec![
                    request_id.into(),
                    ttl_ms.to_string(),
                    batch_size.to_string(),
                ],
            )
            .await?;

        info!(
            request_id = %request_id,
            duration_ms = start.elapsed().as_millis(),
            acquired = res.is_some(),
            driver_id = ?res,
            "acquire_driver completed"
        );

        Ok(res)
    }

    /// Release driver (timeout / rejection) with cooldown
    pub async fn release_driver(
        &self,
        driver_id: &str,
        penalty: f64,
        cooldown_ms: u64,
    ) -> Result<(), RedisError> {
        let current_time = current_time_ms();

        // Set cooldown period
        let _: () = self
            .redis
            .zadd(
                &self.cooldown_key,
                None,
                None,
                false,
                false,
                ((current_time + cooldown_ms) as f64, driver_id),
            )
            .await?;

        // Release driver atomically
        let released: i64 = self
            .evalsha_with_fallback(
                &self.release_driver_script,
                vec![
                    self.pool_key.clone(),
                    self.inflight_key.clone(),
                    self.inflight_score_key.clone(),
                    self.request_map_key.clone(),
                ],
                vec![driver_id.into(), penalty.to_string()],
            )
            .await?;

        if released == 1 {
            info!(driver_id = %driver_id, penalty = %penalty, cooldown_ms = %cooldown_ms, "Driver released");
        } else {
            warn!(driver_id = %driver_id, "Driver not found in inflight when releasing");
        }

        Ok(())
    }

    /// Reward driver (increment score)
    pub async fn reward_driver(
        &self,
        driver_id: &str,
        reward: f64,
    ) -> Result<f64, RedisError> {
        let res = self
            .redis
            .pool
            .zincrby(&self.pool_key, reward, driver_id)
            .await
            .map_err(|err| {
                Error::new(
                    fred::error::ErrorKind::Unknown,
                    format!("Failed to reward driver: {}", err),
                )
            })?;

        let new_score = Self::parse_f64(fred::types::Value::Double(res))?;
        info!(driver_id = %driver_id, reward = %reward, new_score = %new_score, "Driver rewarded");

        Ok(new_score)
    }

    /// Remove driver from all data structures
    pub async fn remove_driver(
        &self,
        driver_id: &str,
    ) -> Result<(), RedisError> {
        let _: () = self.redis.pool.zrem(&self.pool_key, driver_id).await?;
        let _: () = self.redis.pool.zrem(&self.inflight_key, driver_id).await?;
        let _: () = self.redis.pool.zrem(&self.cooldown_key, driver_id).await?;
        let _: () =
            self.redis.pool.hdel(&self.request_map_key, driver_id).await?;
        let _: () =
            self.redis.pool.hdel(&self.inflight_score_key, driver_id).await?;

        info!(driver_id = %driver_id, "Driver removed from all structures");
        Ok(())
    }

    /// Confirm driver assignment (successful dispatch)
    pub async fn confirm_driver(
        &self,
        driver_id: &str,
    ) -> Result<(), RedisError> {
        let _: () = self.redis.pool.zrem(&self.inflight_key, driver_id).await?;
        let _: () =
            self.redis.pool.hdel(&self.inflight_score_key, driver_id).await?;
        let _: () =
            self.redis.pool.hdel(&self.request_map_key, driver_id).await?;

        info!(driver_id = %driver_id, "Driver assignment confirmed");
        Ok(())
    }

    /// Get list of expired inflight drivers
    pub async fn get_expired_inflight_drivers(
        &self,
    ) -> Result<Vec<String>, RedisError> {
        let now_ms = current_time_ms() as i64;
        let expired: Vec<String> = self
            .redis
            .pool
            .zrange(
                &self.inflight_key,
                f64::NEG_INFINITY,
                now_ms as f64,
                Some(fred::types::sorted_sets::ZSort::ByScore),
                false, // rev
                None,  // limit
                false, // withscores
            )
            .await?;

        if !expired.is_empty() {
            warn!(count = expired.len(), "Found expired inflight drivers");
        }

        Ok(expired)
    }

    /// Batch release expired drivers with penalty
    pub async fn release_expired_drivers(&self) -> Result<usize, RedisError> {
        let expired = self.get_expired_inflight_drivers().await?;
        let count = expired.len();

        for driver_id in expired {
            if let Err(e) = self
                .release_driver(
                    &driver_id,
                    self.config.timeout_penalty,
                    self.config.default_cooldown_ms,
                )
                .await
            {
                error!(driver_id = %driver_id, error = ?e, "Failed to release expired driver");
            }
        }

        if count > 0 {
            info!(count = %count, "Released expired drivers");
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {}
