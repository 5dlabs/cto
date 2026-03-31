Implement subtask 2001: Initialize Rust Axum project scaffold with PostgreSQL client

## Objective
Create the Rust workspace and Axum 0.7 application skeleton with PostgreSQL connection pool (sqlx or deadpool-postgres) configured from the infra ConfigMap environment variables. Set up the project structure with layered architecture (handlers, services, repositories, models).

## Steps
1. Run `cargo init equipment-catalog` and add dependencies: axum 0.7, tokio, serde, serde_json, sqlx (with postgres and runtime-tokio features), dotenvy, tracing, tracing-subscriber. 2. Create module structure: src/{main.rs, config.rs, routes/, handlers/, services/, repositories/, models/, errors.rs}. 3. In config.rs, read DATABASE_URL from environment (injected via `envFrom` referencing the infra ConfigMap). 4. Initialize sqlx::PgPool in main.rs and pass it via Axum state (Arc or Extension). 5. Configure tracing subscriber for structured JSON logging. 6. Set up a basic Axum router with a placeholder root route returning 200. 7. Add a Dockerfile with multi-stage build (rust:1.75-slim for build, debian:bookworm-slim for runtime). 8. Verify the app starts, connects to PostgreSQL, and responds on the root route.

## Validation
Application compiles without errors. `cargo run` starts the server and successfully creates a PgPool connection. A GET to / returns HTTP 200. Docker image builds successfully.