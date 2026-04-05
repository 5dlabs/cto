Implement subtask 5001: Scaffold customer-vetting service crate in Cargo workspace

## Objective
Create the `sigma1-services/services/customer-vetting/` crate with Cargo.toml, main.rs, and module structure. Wire up dependencies on shared-auth, shared-db, shared-error, shared-observability workspace crates. Configure Axum 0.7 application skeleton with health endpoint and tracing initialization.

## Steps
1. Create directory `sigma1-services/services/customer-vetting/` with `Cargo.toml`.
2. Add workspace member entry in root `Cargo.toml`.
3. Declare dependencies: axum 0.7, tokio (full), serde/serde_json, uuid, chrono, sqlx (postgres+runtime-tokio), reqwest (with rustls-tls), tokio-retry, and workspace crates (shared-auth, shared-db, shared-error, shared-observability).
4. Create `src/main.rs` with `#[tokio::main]`, initialize tracing from shared-observability, create Axum router with `GET /healthz` returning 200.
5. Create module stubs: `src/routes/mod.rs`, `src/models/mod.rs`, `src/clients/mod.rs`, `src/pipeline/mod.rs`, `src/scoring/mod.rs`, `src/cache/mod.rs`.
6. Define `AppState` struct holding: `PgPool`, `ValKeyPool` (redis connection manager), and config struct with API keys/URLs.
7. Verify `cargo check` passes and health endpoint responds.

## Validation
Run `cargo check` and `cargo build` for the crate with no errors. `GET /healthz` returns HTTP 200 with body `{"status":"ok"}`.