use std::time::Duration;

/// Which rate-limiting algorithm a policy uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algo {
    /// Sorted-set log of request timestamps. Exact, slightly more memory per key.
    /// Best for auth/login endpoints where precision matters.
    SlidingWindowLog,
    /// Two-counter sliding window (curr + prev frame, weighted).
    /// O(1) memory, ~1% precision error. Good default for high-volume APIs.
    SlidingWindowCounter,
    /// Token bucket — refills continuously at `refill_per_sec`.
    /// Native burst support. Best for user-facing bursty APIs.
    TokenBucket,
}

/// What the middleware should do when Redis is unreachable / errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailMode {
    /// Reject the request on limiter error. Use for auth/sensitive endpoints —
    /// a Redis outage must not let brute-forcers through.
    Closed,
    /// Allow the request but record an error metric. Use for general APIs
    /// where availability beats strict limiting.
    Open,
}

/// What key the limiter is scoped to. Composed by the middleware.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyScope {
    /// One bucket per client IP. Use for unauthenticated endpoints.
    Ip,
    /// One bucket per authenticated user.
    User,
    /// Both — applies the policy twice. Use for expensive endpoints
    /// (search, fare estimation) where you want both per-user AND per-IP caps.
    UserAndIp,
}

/// A complete rate-limit policy attached to a route.
#[derive(Debug, Clone)]
pub struct Policy {
    pub algo: Algo,
    pub scope: KeyScope,
    /// Max requests per `window` for SlidingWindow*; capacity for TokenBucket.
    pub capacity: u32,
    /// Window length (SlidingWindow*) or full-refill duration (TokenBucket).
    pub window: Duration,
    pub fail_mode: FailMode,
    /// Cost in tokens per request (TokenBucket only). Default 1.
    /// Use >1 to make expensive endpoints "cost more" against the same bucket.
    pub cost: u32,
}

impl Policy {
    /// Strict per-IP limiter intended for auth endpoints (login, OTP send,
    /// password reset). Fail-closed because allowing through during a Redis
    /// outage would defeat the brute-force protection these endpoints exist for.
    pub fn auth_strict() -> Self {
        Self {
            algo: Algo::SlidingWindowLog,
            scope: KeyScope::Ip,
            capacity: 10,
            window: Duration::from_secs(60),
            fail_mode: FailMode::Closed,
            cost: 1,
        }
    }

    /// Default for authenticated user APIs — generous bucket that refills
    /// smoothly. Fail-open because losing rate limiting beats losing the API.
    pub fn user_default() -> Self {
        Self {
            algo: Algo::TokenBucket,
            scope: KeyScope::User,
            capacity: 60,
            window: Duration::from_secs(60),
            fail_mode: FailMode::Open,
            cost: 1,
        }
    }

    /// Expensive endpoints (fare estimation, nearby drivers) — applies the
    /// limit per-user AND per-IP. Stops both single-user abuse and a
    /// botnet sharing one stolen account.
    pub fn expensive() -> Self {
        Self {
            algo: Algo::TokenBucket,
            scope: KeyScope::UserAndIp,
            capacity: 20,
            window: Duration::from_secs(60),
            fail_mode: FailMode::Open,
            cost: 1,
        }
    }

    /// Endpoints that legitimately fire several times per second per user —
    /// e.g. driver location pings, websocket pongs. Capacity 120 over 60s =
    /// 2 req/sec sustained with 120-burst headroom. Fail-open: dropping
    /// location updates is worse than rate-limit consistency here.
    pub fn high_frequency() -> Self {
        Self {
            algo: Algo::TokenBucket,
            scope: KeyScope::User,
            capacity: 120,
            window: Duration::from_secs(60),
            fail_mode: FailMode::Open,
            cost: 1,
        }
    }
}
