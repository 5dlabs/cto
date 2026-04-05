Implement subtask 2001: Initialize Rust/Axum project with PostgreSQL and Redis connection pools

## Objective
Scaffold the Rust project with Axum 0.7, configure connection pools for PostgreSQL (via sqlx) and Redis, reading endpoints from environment variables sourced from the sigma1-infra-endpoints ConfigMap.

## Steps
1. Create a new Rust project with `cargo init equipment-catalog`.
2. Add dependencies: axum 0.7, tokio, sqlx (with postgres and runtime-tokio features), redis (or deadpool-redis), serde, serde_json, dotenvy.
3. Create a config module that reads POSTGRES_URL, REDIS_URL, S3_ENDPOINT, S3_PRODUCT_IMAGES_BUCKET from environment variables (populated via envFrom: sigma1-infra-endpoints ConfigMap).
4. Initialize a sqlx::PgPool with the POSTGRES_URL.
5. Initialize a Redis connection pool with the REDIS_URL.
6. Create an AppState struct holding both pools and config, and pass it into Axum's Router as state.
7. Create a basic main.rs that starts the Axum server on 0.0.0.0:8080.
8. Add a Dockerfile for the service (multi-stage build with cargo-chef for caching).
9. Add a Kubernetes Deployment manifest referencing the sigma1-infra-endpoints ConfigMap via envFrom.

## Validation
Cargo build succeeds with no errors. The application starts and binds to port 8080. With valid POSTGRES_URL and REDIS_URL env vars, connection pools initialize without error. A curl to the root path returns a response (even if 404).