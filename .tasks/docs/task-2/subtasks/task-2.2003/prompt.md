Implement subtask 2003: Integrate PostgreSQL and Redis for data and caching

## Objective
Connect the service to PostgreSQL using `sqlx` and integrate Redis for caching and rate limiting, utilizing the `sigma1-infra-endpoints` ConfigMap.

## Steps
1. Configure `sqlx` to connect to PostgreSQL using the URL from `sigma1-infra-endpoints`.2. Implement connection pooling for PostgreSQL.3. Integrate `redis` crate for caching product data and implementing tenant-based rate limiting, using the Redis URL from `sigma1-infra-endpoints`.

## Validation
1. Deploy the service and verify successful connection to PostgreSQL and Redis.2. Test API endpoints and observe Redis cache hits/misses.3. Verify rate limiting functionality by sending excessive requests from a simulated tenant.