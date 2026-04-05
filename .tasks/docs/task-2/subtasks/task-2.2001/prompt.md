Implement subtask 2001: Scaffold Rust project with Axum 0.7, sqlx, redis-rs, and configuration layer

## Objective
Initialize the equipment-catalog Rust project with all required dependencies, project structure, configuration loading from environment variables (sigma1-infra-endpoints ConfigMap via envFrom), database connection pool, Redis connection, and application entrypoint.

## Steps
1. Run 'cargo init equipment-catalog' with Rust 1.75+ edition 2021.
2. Add Cargo.toml dependencies: axum 0.7, tokio (full features), sqlx (postgres, runtime-tokio, tls-rustls, migrate), redis (aio, tokio-comp), serde/serde_json, tower-http (cors, trace), tracing, tracing-subscriber, dotenvy.
3. Create a config module (src/config.rs) that reads env vars: POSTGRES_URL, REDIS_URL, S3_CDN_BASE_URL, S3_ENDPOINT, S3_BUCKET_IMAGES, LISTEN_ADDR (default 0.0.0.0:8080).
4. Create a db module (src/db.rs) that initializes a sqlx::PgPool with the POSTGRES_URL, setting max_connections=10, and runs embedded migrations on startup.
5. Create a cache module (src/cache.rs) that initializes a redis::Client from REDIS_URL.
6. Create an AppState struct holding PgPool, Redis connection manager, and config values; implement Clone.
7. In main.rs, initialize tracing, load config, create AppState, build the Axum router (empty for now), and bind to LISTEN_ADDR.
8. Create a Dockerfile (multi-stage: rust:1.75-slim for build, debian:bookworm-slim for runtime) and a basic Makefile with build/run/test targets.
9. Add a .sqlx/ directory and configure sqlx offline mode for CI builds.

## Validation
cargo build succeeds with no errors; cargo run starts the server and binds to the configured port; the application connects to PostgreSQL and Redis (or fails gracefully with clear error messages when infra is unavailable); Docker image builds successfully.