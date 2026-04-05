Implement subtask 5012: Write comprehensive unit and integration tests

## Objective
Create the full test suite covering scoring algorithm test vectors, pipeline integration with mocked APIs, cache behavior, GDPR deletion, and circuit breaker verification.

## Steps
1. Scoring unit tests (in `src/scoring/mod.rs` #[cfg(test)]):
   - Test vector 1: all positive → GREEN, 100pts, empty risk_flags
   - Test vector 2: all negative → RED, 0pts, all flags present
   - Test vector 3: verified only → 30pts, RED
   - Test vector 4: verified + linkedin(200 followers) + reviews(4.5, 15) + no credit → verify partial credit logic
   - Test vector 5: edge at 70 → GREEN
   - Test vector 6: edge at 40 → YELLOW
   - Test vector 7: 39 → RED
   - Verify risk_flags contents for each vector
2. Pipeline integration test (in `tests/` directory):
   - Use wiremock to mock all 4 external APIs
   - POST /vetting/run → poll GET until completed → verify GREEN score
   - All APIs return errors → verify RED score with all risk_flags
3. Cache integration test:
   - Run vetting, GET returns result, verify second GET doesn't trigger external calls (check mock call counts)
4. GDPR test:
   - Run vetting, DELETE org, GET returns 404, verify DB rows deleted
5. Circuit breaker test:
   - Configure wiremock to return 500 for OpenCorporates 5 times
   - Run 5 vettings expecting failures
   - 6th call returns immediately without hitting the mock (verify mock received exactly 5 requests)
6. All tests use test database with migrations applied via sqlx test utilities.

## Validation
All tests pass. Scoring tests cover at least 7 vectors. Integration tests verify full request lifecycle. Cache test confirms mock call count is 0 on cache hit. GDPR test confirms data deletion. Circuit breaker test confirms exactly 5 mock calls before short-circuit.