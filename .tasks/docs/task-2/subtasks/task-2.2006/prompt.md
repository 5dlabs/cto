Implement subtask 2006: Integrate Redis client for caching and rate limiting infrastructure

## Objective
Set up the Redis connection pool and create reusable caching and rate limiting utility modules that other handlers will consume.

## Steps
1. Add the redis crate (with tokio-comp feature) or deadpool-redis to Cargo.toml. 2. Create src/services/redis.rs module. 3. Initialize a Redis connection pool from REDIS_URL environment variable, add to AppState. 4. Implement a generic cache helper: async fn cache_get<T: DeserializeOwned>(pool, key) -> Option<T> and async fn cache_set<T: Serialize>(pool, key, value, ttl_seconds). 5. Implement a rate limiting module src/middleware/rate_limit.rs: use a sliding window counter pattern with Redis INCR + EXPIRE. Key pattern: 'ratelimit:{tenant_id}:{endpoint}:{window}'. 6. Create an Axum middleware layer that extracts tenant_id from headers or API key and applies per-tenant rate limits (configurable: e.g., 100 req/min default). 7. Return 429 Too Many Requests with Retry-After header when limit exceeded.

## Validation
Redis pool connects successfully; cache_set followed by cache_get returns the stored value; cache entries expire after TTL; rate limiter returns 429 after exceeding the configured limit; rate limit counters reset after the window expires.