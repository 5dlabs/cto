Implement subtask 2010: Implement Valkey-based rate limiting middleware

## Objective
Build token-bucket rate limiting middleware using Valkey, supporting per-IP and per-API-key limits configurable via environment variables.

## Steps
1. Create `middleware/rate_limit.rs` module.
2. Implement token bucket algorithm using Valkey: use a Lua script or MULTI/EXEC for atomic increment-and-check. Key format: `ratelimit:{identifier}:{window}`. Use INCR + EXPIRE pattern or sliding window with sorted sets.
3. Identifier extraction: if request has valid JWT, use subject claim as identifier; otherwise use client IP from `x-forwarded-for` or `ConnectInfo`.
4. Configuration via env vars: `RATE_LIMIT_PUBLIC` (default 100), `RATE_LIMIT_AUTHENTICATED` (default 1000), `RATE_LIMIT_WINDOW_SECONDS` (default 60).
5. Implement as Axum middleware layer: on each request, check rate limit. If exceeded, return 429 Too Many Requests with `Retry-After` header and JSON error body.
6. Add `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset` response headers on all requests.
7. Exempt health and metrics endpoints from rate limiting.

## Validation
Integration test: (1) Send 100 requests from same IP in rapid succession to a public endpoint, verify all return 200. (2) Send request 101, verify 429 status with Retry-After header. (3) Verify authenticated requests get the higher 1000 req/min limit. (4) Verify /health/live and /metrics are not rate limited. (5) Verify X-RateLimit-Remaining header decrements correctly.