Implement subtask 4001: Scaffold Rust/Axum service with PostgreSQL and Redis connectivity

## Objective
Initialize the Rust project with Axum 0.7, set up PostgreSQL connection pool via sqlx, Redis client, and health check endpoints, reading all connection details from the sigma1-infra-endpoints ConfigMap.

## Steps
1. cargo init with workspace structure if needed.
2. Add dependencies: axum 0.7, tokio, sqlx (with postgres and runtime-tokio features), redis (or deadpool-redis), serde, serde_json, tracing, tracing-subscriber, tower-http.
3. Create src/main.rs with Axum router setup, binding to configurable host:port.
4. Read DATABASE_URL and REDIS_URL from environment (injected via envFrom from sigma1-infra-endpoints ConfigMap).
5. Initialize sqlx::PgPool with connection pool settings and redis::Client.
6. Create AppState struct holding PgPool, Redis connection pool, and config.
7. Implement GET /healthz (liveness) and GET /readyz (readiness) that check DB and Redis connectivity.
8. Set up tracing with structured JSON logging.
9. Create Dockerfile with multi-stage build (builder with rust:1.75, runtime with debian-slim).
10. Add a Makefile or justfile with targets: build, test, lint (clippy), fmt-check.

## Validation
cargo build succeeds; cargo clippy reports no warnings; application starts and /healthz returns 200; /readyz returns 200 when DB and Redis are reachable, 503 otherwise; Docker image builds and runs correctly.