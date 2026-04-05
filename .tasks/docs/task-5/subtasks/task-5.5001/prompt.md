Implement subtask 5001: Scaffold Rust/Axum project with dependencies and infrastructure wiring

## Objective
Initialize the Rust 1.75+ project with Axum 0.7, sqlx, reqwest (for external HTTP calls), serde, and tokio. Configure Cargo.toml with workspace structure. Set up the Axum router skeleton, health check endpoint, and environment configuration to read connection strings and API keys from the 'sigma1-infra-endpoints' ConfigMap via envFrom and Kubernetes secrets.

## Steps
1. `cargo init vetting-service` with Rust 1.75+ edition.
2. Add dependencies: axum 0.7, tokio (full features), sqlx (postgres, runtime-tokio-rustls), serde/serde_json, reqwest (rustls-tls), tracing, tracing-subscriber, dotenvy.
3. Create src/main.rs with Axum app skeleton: Router::new() with a GET /health endpoint.
4. Create src/config.rs to read env vars: DATABASE_URL, OPENCORPORATES_API_KEY, LINKEDIN_API_KEY, GOOGLE_REVIEWS_API_KEY, CREDIT_API_KEY, and any service URLs.
5. Create src/db.rs to initialize a sqlx::PgPool from DATABASE_URL.
6. Wire up tracing subscriber for structured logging.
7. Ensure the binary compiles and the health endpoint returns 200 OK.

## Validation
Project compiles without errors; `cargo build` succeeds; health endpoint at GET /health returns HTTP 200; config module correctly reads all expected environment variables (tested with .env file locally).