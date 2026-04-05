## Acceptance Criteria

- [ ] 1. Unit tests for availability calculation logic covering overlapping date ranges, partial bookings, and full capacity — minimum 80% coverage on business logic modules. 2. Integration tests using `sqlx::test` with a real PostgreSQL instance: verify CRUD for products/categories, availability queries return correct counts after bookings, and checkout decrements availability atomically. 3. Availability endpoint benchmark: `cargo bench` or `wrk` test confirming p99 < 500ms for availability queries with 1000 products and 10,000 bookings. 4. Rate limiting test: send 101 requests in 60 seconds to a rate-limited endpoint, verify 429 status on request 101. 5. OpenAPI spec validates with `swagger-cli validate`. 6. GDPR deletion test: create booking, call DELETE endpoint, verify booking data is removed and confirmation JSON is returned. 7. Docker image builds successfully and `docker run` passes health checks.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.