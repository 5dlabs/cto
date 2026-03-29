Implement subtask 2006: Implement health check endpoint and optional Redis caching layer

## Objective
Implement the GET /health endpoint with database connectivity check, and the optional Redis caching layer for GET by ID with silent fallthrough on Redis failure.

## Steps
1. **GET /health**:
   - Query `sqlx::query("SELECT 1").fetch_one(&pool).await`.
   - If successful, return `(StatusCode::OK, Json({"status": "healthy", "database": "connected"}))`.
   - If failed, return `(StatusCode::SERVICE_UNAVAILABLE, Json({"status": "degraded", "database": "disconnected"}))`.
   - Register route: `.route("/health", get(health_check))`.
2. **Redis caching** (behind `cache` feature flag or always compiled with optional runtime behavior):
   - Create `src/cache.rs` module.
   - On GET /api/v1/notifications/:id: before DB query, attempt `redis.get("notification:{id}")`. If hit, deserialize and return. If miss or error, proceed to DB. On DB hit, attempt `redis.set_ex("notification:{id}", serialized, 300)` (5 min TTL). Catch and log any Redis errors, never propagate.
   - On POST (create): no cache action needed (new ID).
   - On DELETE (cancel): invalidate `redis.del("notification:{id}")`. Catch errors silently.
   - If `AppState.redis` is `None`, skip all cache operations.
3. Update the GET by ID handler in 2004 to call through the cache layer.
4. Ensure all Redis failures are logged at `warn` level but never affect HTTP responses.

## Validation
Health check returns 200 with `{"status": "healthy", "database": "connected"}` when DB is reachable. Health check returns 503 when DB is unreachable (test by using invalid DATABASE_URL). Redis cache: with Redis available, second GET by ID is served from cache (verify via Redis GET command). With Redis unavailable, GET by ID still returns correct data from Postgres (no error in HTTP response, warning in logs).