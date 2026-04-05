Implement subtask 3008: Integrate Redis for session cache and ephemeral state

## Objective
Add Redis (Valkey) integration for session caching, barcode scan deduplication, and ephemeral operational state in the RMS service.

## Steps
1. Create /internal/cache/redis.go with Redis client initialization using go-redis, reading REDIS_URL from config (sigma1-infra-endpoints ConfigMap).
2. Implement session cache helpers: SetSession(sessionID, data, TTL), GetSession(sessionID), DeleteSession(sessionID).
3. Implement barcode scan deduplication: cache recent scan results with short TTL (e.g., 5 seconds) to prevent duplicate scans from triggering multiple state changes.
4. Implement crew schedule cache: cache computed schedules with configurable TTL, invalidate on assignment changes.
5. Export a Cache interface that services can use for their caching needs.
6. Add connection health check to the /healthz endpoint.

## Validation
Redis client connects successfully using REDIS_URL; session set/get/delete round-trips correctly; barcode dedup prevents duplicate processing within TTL window; health check reports Redis status; graceful handling when Redis is unavailable (fallback to no-cache).