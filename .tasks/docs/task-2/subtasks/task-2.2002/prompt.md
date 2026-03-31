Implement subtask 2002: Initialize Redis client for caching and rate limiting

## Objective
Add Redis/Valkey client to the Axum application, reading connection parameters from the infra ConfigMap, and make the connection pool available in Axum shared state.

## Steps
1. Add `redis` crate (with tokio-comp feature) or `fred` crate to Cargo.toml. 2. In config.rs, read REDIS_URL from environment variables (provided by infra ConfigMap). 3. Create a Redis connection manager/pool and store it in the shared Axum state alongside the PgPool. 4. Add a simple Redis PING health check utility function. 5. Verify connectivity at startup with a PING command, logging success or failure.

## Validation
Application starts and logs successful Redis PING. A unit test confirms the Redis client can set and get a test key.