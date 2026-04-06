Implement subtask 3010: Implement Redis session cache integration

## Objective
Add Redis client initialization and session caching logic to the RMS service for caching frequently accessed data and session state.

## Steps
1. Create `internal/cache/redis.go` with a Redis client wrapper using go-redis/redis. 2. Initialize Redis connection from ConfigMap environment variables (REDIS_HOST, REDIS_PORT). 3. Implement a generic cache layer: Get, Set, Delete with TTL support. 4. Cache frequently accessed entities (e.g., crew availability, inventory counts) with appropriate TTLs. 5. Implement session storage helpers if session-based auth is chosen (store/retrieve/delete session by token). 6. Add connection health check for Redis to the service health endpoint. 7. Ensure graceful degradation: if Redis is unavailable, fall back to direct DB queries (cache-aside pattern).

## Validation
Redis client connects successfully using ConfigMap values; cache set/get/delete operations work correctly; cache miss falls through to DB; Redis health check reports status accurately; service starts and operates even if Redis is temporarily unavailable.