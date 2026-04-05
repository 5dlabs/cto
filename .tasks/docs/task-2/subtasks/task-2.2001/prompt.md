Implement subtask 2001: Initialize Cargo workspace and shared crate with DB pool, error types, and ConfigMap env parsing

## Objective
Create the Cargo workspace at services/rust/ with members catalog, finance, vetting, and shared. Implement the shared crate foundation: PgPool setup reading POSTGRES_URL from env (sigma1-infra-endpoints ConfigMap), standard JSON error response type, and env/config parsing utilities.

## Steps
1. Create `services/rust/Cargo.toml` as a workspace with members `shared`, `catalog`, `finance`, `vetting`.
2. In `shared/Cargo.toml`, add dependencies: `sqlx` (with postgres, runtime-tokio, tls-rustls features), `serde`, `serde_json`, `axum 0.7`, `tokio`, `thiserror`.
3. Implement `shared::db` module: `pub async fn create_pool() -> Result<PgPool>` that reads `POSTGRES_URL` from env, configures max connections (default 10), connect timeout (5s), and returns the pool.
4. Implement `shared::error` module: define `AppError` enum with variants (NotFound, BadRequest, Unauthorized, Internal, Conflict) that implements `IntoResponse` returning JSON `{"error": "...", "code": N}` with appropriate HTTP status codes.
5. Implement `shared::config` module: struct `InfraConfig` that deserializes from env vars matching the sigma1-infra-endpoints ConfigMap keys (POSTGRES_URL, VALKEY_URL, R2_ENDPOINT, etc.).
6. Create stub `catalog/Cargo.toml` depending on `shared` via path reference.
7. Verify workspace compiles with `cargo check --workspace`.

## Validation
Run `cargo check --workspace` succeeds. Unit test that `AppError::NotFound` produces a 404 JSON response. Unit test that `InfraConfig` parses from env vars correctly. Test `create_pool` returns error with invalid POSTGRES_URL.