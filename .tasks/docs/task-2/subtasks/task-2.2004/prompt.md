Implement subtask 2004: Integrate Redis for rate limiting and response caching

## Objective
Add Redis integration to the Equipment Catalog service for API rate limiting and caching of catalog read responses to meet performance requirements.

## Steps
1. Add `redis` (or `fred`) crate to Cargo.toml.
2. Initialize a Redis connection pool from REDIS_URL in AppState.
3. Implement a rate-limiting middleware for Axum:
   - Use a sliding window or token bucket algorithm stored in Redis.
   - Key by client IP (or API key if present).
   - Default limit: 100 requests/minute for public endpoints.
   - Return HTTP 429 with Retry-After header when exceeded.
4. Implement response caching for read endpoints:
   - Cache `GET /catalog/categories` with a 5-minute TTL.
   - Cache `GET /catalog/products` keyed by query params with a 2-minute TTL.
   - Cache `GET /catalog/products/:id` with a 5-minute TTL.
   - Do NOT cache availability (it changes frequently).
5. Add cache invalidation: when admin creates/updates a product or category, delete relevant cache keys.
6. Implement as Axum middleware layers or per-handler logic.
7. Add Redis health check to the readiness probe.

## Validation
Verify rate limiting triggers HTTP 429 after exceeding 100 requests/minute from the same IP. Verify cached responses are served from Redis (check response headers or Redis key existence). Verify cache is invalidated after an admin mutation. Verify Redis connection failure degrades gracefully (service still works without cache).