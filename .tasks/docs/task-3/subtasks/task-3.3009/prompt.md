Implement subtask 3009: Integrate Redis session cache

## Objective
Set up Redis client connection using the ConfigMap URL and implement session caching for the RMS service.

## Steps
1. Add a Redis Go client dependency (go-redis/redis/v9). 2. Create a cache package in /internal/cache/ that initializes a Redis client using the Redis URL from the ConfigMap. 3. Implement session storage helpers: SetSession(key, data, ttl), GetSession(key), DeleteSession(key). 4. Optionally implement a caching layer for frequently accessed data (e.g., inventory item lookups) with appropriate TTLs. 5. Ensure the Redis connection is health-checked and included in the overall service health status.

## Validation
Session set/get/delete operations work correctly against Redis; expired sessions are not returned; Redis health check reports status accurately; service starts gracefully if Redis is temporarily unavailable.