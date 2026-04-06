Implement subtask 2001: Scaffold Rust/Axum project with dependencies and configuration

## Objective
Initialize the Rust project with Cargo, configure Axum 0.7, sqlx (PostgreSQL), redis-rs, and set up the application configuration layer to consume environment variables from the sigma1-infra-endpoints ConfigMap.

## Steps
1. Run `cargo init equipment-catalog` and set up the workspace.
2. Add dependencies to Cargo.toml:
   - axum = "0.7"
   - tokio = { version = "1", features = ["full"] }
   - sqlx = { version = "0.7", features = ["runtime-tokio", "tls-rustls", "postgres", "uuid", "chrono", "migrate"] }
   - redis = { version = "0.24", features = ["tokio-comp"] }
   - serde = { version = "1", features = ["derive"] }
   - serde_json = "1"
   - uuid = { version = "1", features = ["v4", "serde"] }
   - chrono = { version = "0.4", features = ["serde"] }
   - tracing, tracing-subscriber for structured logging
   - prometheus-client or metrics + metrics-exporter-prometheus for metrics
3. Create a config module (src/config.rs) that reads from environment variables: POSTGRES_URL, REDIS_URL, S3_ENDPOINT_URL, S3_PRODUCT_IMAGES_BUCKET, S3_ACCESS_KEY_ID, S3_SECRET_ACCESS_KEY, LISTEN_ADDR (default 0.0.0.0:8080).
4. Create an AppState struct holding: PgPool, Redis ConnectionManager, S3 config, and any shared state.
5. Create main.rs that initializes AppState, builds the Axum Router (empty routes for now), and starts the server.
6. Create a Dockerfile (multi-stage: rust builder → distroless/scratch runtime).
7. Create a Kubernetes Deployment manifest in the sigma1 namespace with envFrom referencing sigma1-infra-endpoints ConfigMap and secretRef for sigma1-s3-credentials.

## Validation
Project compiles with `cargo build`. The binary starts and listens on the configured port. Docker image builds successfully. Environment variables from ConfigMap are correctly parsed into the config struct (unit test for config parsing).