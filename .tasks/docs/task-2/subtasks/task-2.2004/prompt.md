Implement subtask 2004: Implement shared rate limiting middleware using Valkey

## Objective
Build a configurable per-route rate limiting middleware in the shared crate using a sliding window algorithm backed by Valkey (redis-rs).

## Steps
1. In `shared/src/rate_limit.rs`, define `RateLimitConfig { max_requests: u32, window_secs: u64 }`.
2. Implement sliding window rate limiting using Valkey sorted sets:
   - Key: `ratelimit:{identifier}:{route}` where identifier is IP or API key.
   - On each request: ZADD current timestamp, ZREMRANGEBYSCORE to remove entries outside the window, ZCARD to count.
   - If count > max_requests, return 429 with `Retry-After` header.
3. Create Axum middleware `pub fn rate_limit_layer(config: RateLimitConfig, valkey: Client) -> impl Layer`.
4. Ensure the middleware is configurable per-route (e.g., applied selectively via Axum's `.layer()` on specific route groups).
5. Add appropriate error handling: if Valkey is unavailable, log warning and allow the request (fail-open).

## Validation
Integration test with real Valkey: send max_requests within the window and verify all return 200. Send one more and verify 429 with Retry-After header. Test fail-open behavior: disconnect Valkey and verify requests still pass through. Test window expiry: send max requests, wait for window to elapse, verify requests succeed again.