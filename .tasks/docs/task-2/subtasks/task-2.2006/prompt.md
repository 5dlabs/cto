Implement subtask 2006: Implement Redis-based rate limiting per tenant

## Objective
Implement rate limiting middleware using Redis sliding window counters, keyed by tenant ID, to protect all API endpoints.

## Steps
1. Create src/middleware/rate_limit.rs:
   - Implement an Axum middleware (using axum::middleware::from_fn_with_state or tower layer).
   - Extract tenant_id from request (from JWT claims, API key header, or X-Tenant-Id header depending on dp-4 decision).
   - Use Redis INCR + EXPIRE pattern or sliding window log algorithm:
     Key: `rate_limit:{tenant_id}:{window_timestamp}`
     Window: 60 seconds
     Limit: 100 requests per tenant per minute (configurable via env var RATE_LIMIT_PER_MINUTE)
   - If limit exceeded, return 429 Too Many Requests with Retry-After header.
   - Add rate limit headers to all responses: X-RateLimit-Limit, X-RateLimit-Remaining, X-RateLimit-Reset.
2. Use redis-rs async commands.
3. Add the middleware to the router for /api/v1/ routes (not for /health/ or /metrics).
4. Handle Redis connection failures gracefully: if Redis is unavailable, allow the request through (fail-open) and log a warning.

## Validation
Integration test: Send 100 requests within a minute from the same tenant and verify all succeed. Send request 101 and verify 429 response with correct Retry-After header. Verify rate limit headers are present on all API responses. Test fail-open behavior when Redis is down (mock or stop Redis, verify requests still pass).