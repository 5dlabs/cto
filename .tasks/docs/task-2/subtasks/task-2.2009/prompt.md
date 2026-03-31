Implement subtask 2009: Implement rate limiting middleware using Redis

## Objective
Add an Axum middleware layer that enforces per-tenant or per-IP rate limiting using Redis as the backing store with a sliding window or token bucket algorithm.

## Steps
1. Create src/middleware/rate_limit.rs. 2. Implement a sliding window rate limiter using Redis MULTI/EXEC or Lua script: key pattern `ratelimit:{identifier}:{window}`, INCR + EXPIRE approach or sorted set approach. 3. Identifier extraction: check X-Tenant-ID header first, fall back to X-Forwarded-For or peer IP. 4. Configure rate limits via environment: RATE_LIMIT_REQUESTS_PER_MINUTE (default 60 for public, 300 for agent API). 5. Add the middleware to the Axum router. Apply different limits to public /api/v1/catalog/ routes vs agent /api/v1/equipment-api/ routes. 6. Return HTTP 429 Too Many Requests with Retry-After header when limit is exceeded. Include rate limit headers on all responses: X-RateLimit-Limit, X-RateLimit-Remaining, X-RateLimit-Reset. 7. Ensure the middleware is non-blocking and handles Redis connection failures gracefully (fail open with a warning log).

## Validation
Sending requests above the configured rate limit returns HTTP 429 with Retry-After header. Responses include X-RateLimit-* headers with correct values. Rate limits reset after the window expires. If Redis is unavailable, requests still succeed (fail-open behavior verified by stopping Redis temporarily).