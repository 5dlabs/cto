<identity>
You are rex working on subtask 2006 of task 2.
</identity>

<context>
<scope>
Write an Axum middleware layer that enforces a 100 requests/minute sliding-window rate limit per client IP (or tenant_id claim), using the Redis ConnectionManager.
</scope>
</context>

<implementation_plan>
Create src/middleware/rate_limit.rs. Use the redis::aio::ConnectionManager from AppState. Sliding window algorithm with Redis sorted sets: key = rate_limit:{identifier} where identifier = tenant_id from JWT claims if present, else client IP from ConnectInfo. Use MULTI/EXEC (pipeline): ZREMRANGEBYSCORE key 0 (now_ms - 60000); ZADD key now_ms now_ms (using timestamp as both score and member, append a unique suffix to member to avoid deduplication); ZCARD key; EXPIRE key 61. If ZCARD > 100 return HTTP 429 with Retry-After header set to (60 - seconds_since_oldest_entry). Implement as tower::Layer wrapping the inner service. Extract client IP via axum::extract::ConnectInfo<SocketAddr>. Add unit tests: simulate 100 calls increment count, 101st triggers 429 logic. Wire the RateLimitLayer onto the Router in main.rs, applied to /api/v1/* routes only (exclude /health and /metrics from rate limiting).
</implementation_plan>

<validation>
cargo test middleware::rate_limit passes. Integration test: send 101 sequential requests from the same IP to GET /api/v1/catalog/products — the 101st returns HTTP 429 with Retry-After header present. After 60 seconds (or mocking time), requests succeed again. /health/live and /metrics are not subject to rate limiting (101 requests to /health/live all return 200).
</validation>