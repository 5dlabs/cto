Implement subtask 2001: Initialize Rust/Axum project with database connection pooling and configuration

## Objective
Scaffold the Rust 1.75+ Axum 0.7 project with Cargo workspace structure, configure environment-based settings loading from the sigma1-infra-endpoints ConfigMap, and set up PostgreSQL connection pooling using sqlx or deadpool-postgres.

## Steps
1. Initialize a new Rust project: 'cargo init equipment-catalog'. 2. Add dependencies in Cargo.toml: axum 0.7, tokio, serde, serde_json, sqlx (with postgres and runtime-tokio features), dotenvy or config crate for env-based config. 3. Create a config module that reads POSTGRES_URL, REDIS_URL, S3_ENDPOINT, S3_PRODUCT_BUCKET from environment variables (injected via sigma1-infra-endpoints ConfigMap). 4. Initialize a sqlx::PgPool with connection pooling (max 10 connections for dev). 5. Create the main.rs Axum app with a basic router and graceful shutdown. 6. Add a Dockerfile (multi-stage build: rust:1.75-slim for build, debian:bookworm-slim for runtime). 7. Create a Kubernetes Deployment manifest referencing envFrom: sigma1-infra-endpoints ConfigMap and relevant secrets.

## Validation
Project compiles with 'cargo build'; the application starts and connects to PostgreSQL successfully when POSTGRES_URL is set; Docker image builds without errors.