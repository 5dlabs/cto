Implement subtask 4001: Scaffold Rust/Axum project with dependency configuration

## Objective
Initialize the Finance service Cargo project with Axum 0.7, configure crate dependencies for PostgreSQL (sqlx), Redis, Stripe SDK, serde, tokio, and establish the application entrypoint with Axum router skeleton, graceful shutdown, and configuration loading from environment variables via the infra-endpoints ConfigMap (envFrom).

## Steps
1. Run `cargo init finance-service`. 2. Add Cargo.toml dependencies: axum 0.7, tokio (full features), serde/serde_json, sqlx (postgres, runtime-tokio, tls-rustls), redis (tokio-comp), stripe-rust, chrono, uuid, dotenvy, tracing, tracing-subscriber. 3. Create `src/main.rs` with Axum app skeleton: build Router, bind to 0.0.0.0:8080, add graceful shutdown signal handler. 4. Create `src/config.rs` to load DATABASE_URL, REDIS_URL, STRIPE_SECRET_KEY, STRIPE_WEBHOOK_SECRET, CURRENCY_API_URL from env vars (populated via envFrom referencing the `{project}-infra-endpoints` ConfigMap). 5. Create `src/state.rs` with AppState struct holding PgPool, Redis connection manager, and Stripe client. 6. Wire up tracing-subscriber for structured JSON logging. 7. Create module stubs: `src/routes/mod.rs`, `src/models/mod.rs`, `src/handlers/mod.rs`, `src/services/mod.rs`, `src/errors.rs`.

## Validation
Cargo build succeeds with no errors. Application starts and binds to port 8080. Configuration loads from environment variables without panic. Tracing output appears in structured JSON format.