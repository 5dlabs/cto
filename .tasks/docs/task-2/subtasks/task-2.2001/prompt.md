Implement subtask 2001: Initialize Cargo workspace root and shared-error crate

## Objective
Create the sigma1-services Cargo workspace root with workspace-level dependency management and implement the shared-error crate providing unified error types and Axum error response formatting used by all services.

## Steps
1. Create `sigma1-services/Cargo.toml` as a workspace root with `[workspace]` members listing `crates/shared-auth`, `crates/shared-db`, `crates/shared-error`, `crates/shared-observability`, and `services/equipment-catalog`. Define `[workspace.dependencies]` for shared deps: axum 0.7, sqlx 0.7 (features: runtime-tokio, tls-rustls, postgres, uuid, chrono, json), serde/serde_json, thiserror, tokio, tracing, uuid, chrono.
2. Create `crates/shared-error/Cargo.toml` and `src/lib.rs`.
3. Define `AppError` enum with variants: NotFound, Unauthorized, Forbidden, BadRequest(String), Conflict(String), Internal(String), Database(sqlx::Error), Validation(String).
4. Implement `IntoResponse` for `AppError` — map each variant to appropriate HTTP status code and JSON body `{ "error": { "code": "...", "message": "..." } }`.
5. Implement `From<sqlx::Error>` for `AppError` to auto-convert database errors.
6. Export a `Result<T> = std::result::Result<T, AppError>` type alias.
7. Ensure `cargo build --workspace` compiles successfully with this crate.

## Validation
Unit tests: construct each AppError variant and call `into_response()`, verify HTTP status codes (404, 401, 403, 400, 409, 500) and JSON body structure. Verify `From<sqlx::Error>` conversion maps to Internal variant. Run `cargo build --workspace` from workspace root to confirm workspace structure.