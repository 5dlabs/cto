Implement subtask 4001: Scaffold Rust/Axum project with dependencies and infrastructure config

## Objective
Initialize a Rust 1.75+ project with Axum 0.7 web framework, sqlx for PostgreSQL, redis-rs for caching, and configuration loading from the sigma1-infra-endpoints ConfigMap.

## Steps
1. Run `cargo init` for the finance service crate.
2. Add dependencies to Cargo.toml: axum 0.7, tokio (full features), sqlx (postgres, runtime-tokio, tls-rustls, migrate, uuid, chrono, rust_decimal), redis-rs (tokio-comp), serde/serde_json, tower-http (cors, trace), tracing/tracing-subscriber, dotenvy for local dev.
3. Create project structure: src/main.rs, src/config.rs, src/routes/ (mod.rs), src/models/, src/db/, src/services/, src/error.rs.
4. Implement src/config.rs: read POSTGRES_URL, REDIS_URL, STRIPE_SECRET_KEY, STRIPE_WEBHOOK_SECRET from environment variables (injected via envFrom from sigma1-infra-endpoints ConfigMap and secrets).
5. Implement src/main.rs: initialize tracing, load config, create sqlx PgPool, create Redis connection, build Axum router with health check at GET /healthz, start server.
6. Implement shared error types in src/error.rs with proper Axum IntoResponse implementations for 400, 404, 409, 500 responses.
7. Add sqlx migrations directory at migrations/.
8. Verify the project compiles and the server starts with `cargo run`.

## Validation
Project compiles with zero warnings; server starts and GET /healthz returns 200 with {"status":"ok"}; config correctly reads all expected environment variables and panics with clear messages when required vars are missing.