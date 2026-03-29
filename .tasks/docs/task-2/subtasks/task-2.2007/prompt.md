Implement subtask 2007: Unit tests for validation logic, enum serialization, and pagination math

## Objective
Write unit tests within src/ modules covering validation logic, enum serialization/deserialization to lowercase JSON, and pagination offset calculation.

## Steps
1. In `src/models.rs` (or a `#[cfg(test)]` module):
   - Test `Channel` enum serializes to lowercase: `serde_json::to_string(&Channel::Email)` == `"email"`.
   - Test `Priority` enum: all variants serialize correctly.
   - Test `NotificationStatus` enum: all variants serialize correctly.
   - Test deserialization of `CreateNotificationRequest` from valid JSON.
   - Test deserialization rejects unknown channel values.
2. In `src/handlers.rs` (or separate test module):
   - Test pagination offset calculation: page=1, per_page=20 → offset=0. page=3, per_page=10 → offset=20.
   - Test per_page clamping: per_page=200 → clamped to 100. per_page=0 → default to 20 (or minimum 1).
   - Test page minimum: page=0 → clamped to 1.
3. In `src/errors.rs`:
   - Test `AppError::NotFound` produces StatusCode 404.
   - Test `AppError::Validation` produces StatusCode 422.
   - Test `AppError::Conflict` produces StatusCode 409.
   - Test error JSON body shape is `{"error": "message"}`.
4. Aim for at least 6 unit test cases across these modules.

## Validation
`cargo test --lib` passes all unit tests. Tests cover: enum serialization (3 enum types × at least 1 variant each), pagination math (offset calculation, per_page clamping, page minimum), error response codes and body shapes. Minimum 6 passing unit test functions.