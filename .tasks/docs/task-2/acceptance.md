## Acceptance Criteria

- [ ] 1. `cargo test` passes all unit and integration tests (minimum 15 test cases covering: valid notification creation, creation with empty title returns 422, get by valid ID returns 200, get by unknown ID returns 404, list with default pagination returns correct structure, list with status filter returns only matching, list with per_page > 100 clamps to 100, cancel pending notification returns 200 with status=cancelled, cancel non-pending returns 409, cancel unknown returns 404, health check returns 200 with database=connected, enum serialization to lowercase JSON, pagination offset calculation, graceful shutdown signal handling). 2. `docker build .` completes successfully and image size < 100MB. 3. Running the container with valid DATABASE_URL against a test Postgres instance: POST /api/v1/notifications returns 201 with a valid UUID within 50ms. 4. GET /health returns `{"status": "healthy"}` with HTTP 200. 5. `cargo clippy -- -D warnings` passes with zero warnings.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.