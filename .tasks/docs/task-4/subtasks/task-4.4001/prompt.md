Implement subtask 4001: Scaffold finance crate in Cargo workspace with shared dependencies

## Objective
Add the `finance` crate to the existing Rex Cargo workspace at `services/rust/finance`, configure Cargo.toml with dependencies (axum, sqlx, serde, utoipa, tokio, reqwest), and wire up the shared crate for health checks, metrics, error types, DB pool, and API key auth middleware. Set up the main.rs entrypoint with Axum router skeleton listening on port 8082.

## Steps
1. Create `services/rust/finance/` directory with `Cargo.toml` and `src/main.rs`.
2. Add `finance` to the workspace `members` in the root `Cargo.toml`.
3. Add dependencies: `axum = "0.7"`, `sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "uuid", "chrono", "rust_decimal"] }`, `serde`, `serde_json`, `utoipa`, `tokio`, `reqwest`, `tracing`, `tracing-subscriber`.
4. Add workspace-internal dependency on the shared crate (e.g., `common = { path = "../common" }`).
5. In `main.rs`, initialize tracing, create DB pool via shared crate, build Axum router with health check from shared crate, bind to `0.0.0.0:8082`.
6. Create module stubs: `mod routes;`, `mod models;`, `mod db;`, `mod services;`, `mod stripe;`, `mod background;`.
7. Verify `cargo build` succeeds and `cargo test` runs (even if no tests yet).

## Validation
Verify `cargo build --workspace` succeeds. Verify `cargo run -p finance` starts and responds to GET /health with 200 OK. Verify shared crate health check and metrics middleware are active.