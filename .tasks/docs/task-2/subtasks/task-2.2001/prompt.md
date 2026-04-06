Implement subtask 2001: Scaffold Rust/Axum project with infrastructure client setup and health endpoints

## Objective
Initialize the Equipment Catalog Rust project with Axum 0.7 scaffolding, PostgreSQL (sqlx) and Redis client pools configured from the sigma1-infra-endpoints ConfigMap, and implement /health/live, /health/ready, and /metrics endpoints.

## Steps
1. Run `cargo init equipment-catalog` and add dependencies: axum 0.7, tokio, sqlx (postgres, runtime-tokio), redis (or deadpool-redis), serde, serde_json, tracing, tracing-subscriber, prometheus (or metrics crate).
2. Create a `config.rs` module that reads environment variables from the sigma1-infra-endpoints ConfigMap (POSTGRES_HOST, POSTGRES_PORT, REDIS_HOST, REDIS_PORT, S3_ENDPOINT, etc.).
3. Create an `AppState` struct holding a sqlx::PgPool and a Redis connection pool.
4. Initialize both pools in main.rs using the config values.
5. Implement `GET /health/live` (returns 200 if server is up), `GET /health/ready` (checks DB and Redis connectivity), and `GET /metrics` (Prometheus text format with basic request counters and latency histograms).
6. Set up tracing/logging with structured JSON output.
7. Add a Dockerfile for the service.
8. Verify the service starts and all three endpoints respond correctly.

## Validation
Compile and run the service; curl /health/live returns 200; /health/ready returns 200 when DB and Redis are reachable (503 otherwise); /metrics returns valid Prometheus text format; logs are structured JSON.