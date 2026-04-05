Implement subtask 5009: End-to-end vetting pipeline integration tests

## Objective
Write comprehensive integration tests that validate the full vetting pipeline from API request through external integrations (mocked), scoring, persistence, and retrieval.

## Steps
1. Set up a test harness using `tokio::test` with a real PostgreSQL test database (via testcontainers or a test-specific database URL).
2. Use wiremock to create mock servers for OpenCorporates, LinkedIn, Google Reviews, and credit APIs.
3. Configure the application to use mock server URLs.
4. Test scenarios:
   a. Happy path: all external APIs return valid data → verify GREEN/YELLOW/RED classification is correct.
   b. Partial data: one or more external APIs return errors → verify graceful degradation, scoring still works with available data.
   c. All external APIs fail → verify appropriate error response.
   d. Duplicate vetting run for same org_id → verify new result is created, GET returns latest.
   e. Credit-specific endpoint returns correct subset of data.
5. Verify database state after each test scenario.
6. Verify metrics are recorded correctly during test runs.
7. Clean up test data between tests.

## Validation
All test scenarios pass. Database contains expected records. Metrics reflect the test activity. Tests are repeatable and isolated from each other.