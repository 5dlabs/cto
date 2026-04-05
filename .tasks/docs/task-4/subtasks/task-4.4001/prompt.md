Implement subtask 4001: Scaffold finance service crate within Cargo workspace

## Objective
Create the finance service crate under sigma1-services/services/finance/ with proper Cargo.toml dependencies on shared-auth, shared-db, shared-error, shared-observability crates. Set up the main.rs with Axum 0.7 server bootstrap, router skeleton, graceful shutdown, and health/metrics endpoints via shared crates.

## Steps
1. Create directory `sigma1-services/services/finance/` with `Cargo.toml` and `src/main.rs`.
2. Add the crate to the workspace `Cargo.toml` members list.
3. Declare dependencies: axum 0.7, tokio, serde/serde_json, sqlx (postgres), uuid, chrono, rust_decimal, stripe-rust, reqwest, shared-auth, shared-db, shared-error, shared-observability.
4. In `main.rs`, initialize tracing via shared-observability, create a DB pool via shared-db, build an Axum Router with a `/healthz` and `/readyz` endpoint, bind to `0.0.0.0:PORT` from env.
5. Set up the module structure: `src/routes/`, `src/models/`, `src/services/`, `src/db/`, `src/stripe/`, `src/background/`, `src/tax/`, `src/currency/`.
6. Add an AppState struct holding the DB pool, Valkey connection, and Stripe client.
7. Ensure `cargo build` and `cargo clippy` pass with no errors.

## Validation
Verify `cargo build --workspace` succeeds. Verify `cargo test -p finance` runs (even if no tests yet). Verify the service starts and responds 200 on `/healthz`.