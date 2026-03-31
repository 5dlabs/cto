Implement subtask 3003: Configure Redis connectivity

## Objective
Establish Redis client connection using ConfigMap endpoints for use in conflict detection caching and session-related lookups.

## Steps
1. Read Redis connection details from environment variables sourced via `envFrom` referencing the infra-endpoints ConfigMap.
2. Use `go-redis/redis/v9` client with configurable pool size and timeouts.
3. Create `/internal/cache/redis.go` exposing a Redis client wrapper with a health check method.
4. Implement basic Get/Set/Delete helpers with context and TTL support.
5. Ensure the client gracefully handles connection failures with retries and logging.

## Validation
Redis client connects and responds to PING. Get/Set/Delete operations work correctly. Health check returns healthy when connected and unhealthy when Redis is unavailable.