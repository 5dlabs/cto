Implement subtask 2008: Integration tests for all five endpoints including error cases

## Objective
Write integration tests in the `tests/` directory using testcontainers-rs or sqlx test fixtures to test all five endpoints end-to-end, including error paths (404, 409, 422).

## Steps
1. Create `tests/api_tests.rs`.
2. Setup: Use testcontainers-rs to spin up a Postgres container, or use sqlx's `#[sqlx::test]` macro with a test database. Build the Axum app with test configuration. Use `axum::test::TestClient` or `tower::ServiceExt` with `oneshot` for in-process HTTP testing.
3. Test cases (minimum 9 integration tests):
   a. **POST valid notification** — 201, body contains UUID, status=pending, correct channel/priority/title/body.
   b. **POST with empty title** — 422, body contains error message.
   c. **POST with empty body** — 422.
   d. **GET by valid ID** — create one first, then GET returns 200 with matching data.
   e. **GET by unknown UUID** — 404 with `{"error": "not found"}`.
   f. **GET list default pagination** — create 3 notifications, list returns page=1, per_page=20, total=3, data has 3 items.
   g. **GET list with status filter** — create pending + cancel one, filter by status=cancelled returns only 1.
   h. **DELETE pending notification** — 200, returned notification has status=cancelled.
   i. **DELETE non-pending notification** — cancel one, try to cancel again, returns 409.
   j. **DELETE unknown ID** — 404.
   k. **GET /health** — 200 with `{"status": "healthy", "database": "connected"}`.
4. Ensure test isolation: each test should clean up or use unique data.
5. Run with `cargo test --test api_tests`.

## Validation
`cargo test --test api_tests` passes all 11 integration test cases. Each test asserts both HTTP status code and response body structure. Tests complete within 60 seconds (testcontainers startup included). No test pollution between cases.