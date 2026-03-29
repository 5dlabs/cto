Implement subtask 2003: Custom AppError enum and error handling middleware

## Objective
Implement the custom `AppError` enum with variants NotFound, Validation, Conflict, and Internal, implementing `IntoResponse` to return appropriate HTTP status codes and JSON error bodies.

## Steps
1. Create `src/errors.rs`:
   - Define `AppError` enum with variants:
     - `NotFound(String)` → 404 `{"error": "..."}`
     - `Validation(String)` → 422 `{"error": "..."}`
     - `Conflict(String)` → 409 `{"error": "..."}`
     - `Internal(String)` → 500 `{"error": "internal server error"}`
   - Implement `IntoResponse` for `AppError` that returns `(StatusCode, Json<serde_json::Value>)`.
   - Implement `From<sqlx::Error>` for `AppError` — map `RowNotFound` to `NotFound`, others to `Internal` (logging the actual error via tracing::error!).
   - Optionally implement `From<redis::RedisError>` for `AppError` mapping to `Internal`.
2. All endpoint handlers will return `Result<impl IntoResponse, AppError>`.
3. Ensure error responses always have a consistent `{"error": "message"}` JSON shape.

## Validation
Unit tests verify: `AppError::NotFound` produces 404 with correct JSON body, `AppError::Validation` produces 422, `AppError::Conflict` produces 409, `AppError::Internal` produces 500 with generic message (not leaking internal details). `From<sqlx::Error>` maps RowNotFound correctly.