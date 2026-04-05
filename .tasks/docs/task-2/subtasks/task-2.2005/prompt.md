Implement subtask 2005: Implement rate limiting middleware using Redis

## Objective
Add per-tenant/per-IP rate limiting to all API endpoints using Redis as the backing store, with configurable limits for public vs. machine-readable endpoints.

## Steps
1. Create src/middleware/rate_limit.rs.
2. Implement a Tower middleware/layer that extracts a rate limit key from the request: use X-Tenant-ID header if present, otherwise fall back to client IP (from X-Forwarded-For or socket addr).
3. Use Redis INCR + EXPIRE pattern (sliding window counter) for rate limiting: key format 'ratelimit:{endpoint_group}:{tenant_or_ip}', TTL of 60 seconds.
4. Configure different limits: public catalog endpoints at 60 req/min per IP, equipment-api endpoints at 30 req/min per tenant.
5. Return HTTP 429 Too Many Requests with a JSON error body and Retry-After header when limit is exceeded.
6. Include X-RateLimit-Limit, X-RateLimit-Remaining, X-RateLimit-Reset response headers on all requests.
7. Make limits configurable via environment variables (RATE_LIMIT_CATALOG, RATE_LIMIT_EQUIPMENT_API).
8. Apply the middleware layer to the appropriate route groups in the Axum router.
9. Handle Redis connection failures gracefully — if Redis is unavailable, allow the request through (fail-open) and log a warning.

## Validation
Send 61 requests to a catalog endpoint within 60 seconds from the same IP and verify the 61st returns 429; verify rate limit headers are present on all responses; verify equipment-api has its own separate limit; verify fail-open behavior when Redis is stopped; verify X-Tenant-ID header scoping works.