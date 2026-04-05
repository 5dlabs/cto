Implement subtask 3015: Implement Valkey (Redis) integration for caching and session support

## Objective
Add go-redis/v9 client initialization for Valkey connection pooling, reading connection details from the sigma1-infra-endpoints ConfigMap.

## Steps
1. Add `github.com/redis/go-redis/v9` to go.mod.
2. Create `internal/cache/valkey.go`:
   - NewValkeyClient function that reads VALKEY_URL (or REDIS_URL) from environment (populated by sigma1-infra-endpoints ConfigMap)
   - Configure connection pool: PoolSize, MinIdleConns, DialTimeout, ReadTimeout, WriteTimeout
   - Implement a health check function that runs PING command
3. Create `internal/cache/stock_cache.go`:
   - Cache stock levels with TTL (e.g., 30 seconds) to avoid repeated aggregation queries
   - Cache key format: `rms:stock:{inventory_item_id}`
   - Invalidate cache on RecordTransaction
4. Integrate Valkey health check into the /health/ready endpoint (service is ready only if both DB and Valkey are connected).
5. Make Valkey optional: if VALKEY_URL is not set, skip cache initialization and fall through to direct DB queries.

## Validation
Verify Valkey client connects successfully when URL is provided. Verify stock level cache hit returns same result as DB query. Verify cache is invalidated after RecordTransaction. Verify service starts correctly when VALKEY_URL is not set (graceful degradation). Verify /health/ready includes Valkey connectivity check.