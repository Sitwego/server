//! Atomic Lua scripts for the three rate-limiting algorithms.
//!
//! All scripts share the same return shape so the Rust side can decode them
//! uniformly:
//!
//!   `{ allowed (0/1), remaining, retry_after_ms, reset_ms }`
//!
//! All time is sourced from `redis.call('TIME')` inside the script so the
//! limiter is consistent across API nodes with skewed clocks.

/// Token bucket — refills continuously, supports burst up to `capacity`.
///
/// KEYS[1] = bucket key (HASH: tokens, last_ms)
/// ARGV[1] = capacity            (integer, max tokens)
/// ARGV[2] = refill_per_sec_x1k  (integer, refill rate * 1000 to stay in ints)
/// ARGV[3] = cost                (integer, tokens this request consumes)
/// ARGV[4] = ttl_secs            (integer, key TTL)
pub const TOKEN_BUCKET: &str = r#"
local t = redis.call('TIME')
local now_ms = tonumber(t[1]) * 1000 + math.floor(tonumber(t[2]) / 1000)

local capacity   = tonumber(ARGV[1])
local refill_rate_per_ms = tonumber(ARGV[2]) / 1000.0 / 1000.0
local cost       = tonumber(ARGV[3])
local ttl        = tonumber(ARGV[4])

local h = redis.call('HMGET', KEYS[1], 'tokens', 'last_ms')
local tokens  = tonumber(h[1])
local last_ms = tonumber(h[2])

if tokens == nil then
    tokens = capacity
    last_ms = now_ms
else
    local elapsed = now_ms - last_ms
    if elapsed < 0 then elapsed = 0 end
    tokens = math.min(capacity, tokens + elapsed * refill_rate_per_ms)
end

local allowed = 0
local retry_ms = 0
if tokens >= cost then
    tokens = tokens - cost
    allowed = 1
else
    -- ceil((cost - tokens) / refill_rate)
    local deficit = cost - tokens
    if refill_rate_per_ms > 0 then
        retry_ms = math.ceil(deficit / refill_rate_per_ms)
    else
        retry_ms = ttl * 1000
    end
end

redis.call('HSET', KEYS[1], 'tokens', tokens, 'last_ms', now_ms)
redis.call('EXPIRE', KEYS[1], ttl)

local reset_ms = 0
if refill_rate_per_ms > 0 then
    reset_ms = now_ms + math.ceil((capacity - tokens) / refill_rate_per_ms)
end

return { allowed, math.floor(tokens), retry_ms, reset_ms }
"#;

/// Sliding-window log — exact request counting via a sorted set.
///
/// KEYS[1] = zset key (members: "<now_ms>:<rand>", score: now_ms)
/// ARGV[1] = window_ms
/// ARGV[2] = limit
/// ARGV[3] = ttl_secs
pub const SLIDING_WINDOW_LOG: &str = r#"
local t = redis.call('TIME')
local now_ms = tonumber(t[1]) * 1000 + math.floor(tonumber(t[2]) / 1000)

local window = tonumber(ARGV[1])
local limit  = tonumber(ARGV[2])
local ttl    = tonumber(ARGV[3])

-- Drop entries older than window
redis.call('ZREMRANGEBYSCORE', KEYS[1], 0, now_ms - window)
local count = redis.call('ZCARD', KEYS[1])

if count < limit then
    -- random suffix so two concurrent inserts at the same ms don't collide
    local member = now_ms .. ':' .. redis.call('INCR', KEYS[1] .. ':seq')
    redis.call('ZADD', KEYS[1], now_ms, member)
    redis.call('EXPIRE', KEYS[1], ttl)
    redis.call('EXPIRE', KEYS[1] .. ':seq', ttl)
    return { 1, limit - count - 1, 0, now_ms + window }
end

-- Reject: retry_after = window - age_of_oldest
local oldest = redis.call('ZRANGE', KEYS[1], 0, 0, 'WITHSCORES')
local retry_ms = window
if oldest[2] ~= nil then
    retry_ms = window - (now_ms - tonumber(oldest[2]))
    if retry_ms < 1 then retry_ms = 1 end
end
return { 0, 0, retry_ms, now_ms + retry_ms }
"#;

/// Sliding-window counter — two integers (curr_frame_count, prev_frame_count)
/// weighted by elapsed fraction of the current frame. O(1) memory.
///
/// KEYS[1] = hash key (fields: frame, curr, prev)
/// ARGV[1] = window_ms (frame length)
/// ARGV[2] = limit
/// ARGV[3] = ttl_secs
pub const SLIDING_WINDOW_COUNTER: &str = r#"
local t = redis.call('TIME')
local now_ms = tonumber(t[1]) * 1000 + math.floor(tonumber(t[2]) / 1000)

local window = tonumber(ARGV[1])
local limit  = tonumber(ARGV[2])
local ttl    = tonumber(ARGV[3])

local frame_id  = math.floor(now_ms / window)
local elapsed   = now_ms - (frame_id * window)
local weight    = 1.0 - (elapsed / window)

local h = redis.call('HMGET', KEYS[1], 'frame', 'curr', 'prev')
local stored_frame = tonumber(h[1])
local curr = tonumber(h[2]) or 0
local prev = tonumber(h[3]) or 0

if stored_frame == nil then
    stored_frame = frame_id
elseif stored_frame == frame_id - 1 then
    -- one frame elapsed: shift curr → prev
    prev = curr
    curr = 0
    stored_frame = frame_id
elseif stored_frame < frame_id - 1 then
    -- gap of 2+ frames: both counters stale
    prev = 0
    curr = 0
    stored_frame = frame_id
end

-- Estimate = prev * remaining_weight + curr
local estimate = math.floor(prev * weight + 0.5) + curr

if estimate + 1 <= limit then
    curr = curr + 1
    redis.call('HSET', KEYS[1], 'frame', stored_frame, 'curr', curr, 'prev', prev)
    redis.call('EXPIRE', KEYS[1], ttl)
    local remaining = limit - (estimate + 1)
    if remaining < 0 then remaining = 0 end
    return { 1, remaining, 0, (frame_id + 1) * window }
else
    -- Conservative: ask the caller to wait until the next frame boundary
    local retry_ms = window - elapsed
    if retry_ms < 1 then retry_ms = 1 end
    return { 0, 0, retry_ms, now_ms + retry_ms }
end
"#;
