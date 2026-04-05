Implement subtask 5009: Write comprehensive integration and end-to-end tests for vetting service

## Objective
Create a full test suite covering the vetting service end-to-end, including integration tests against a test PostgreSQL database with mocked external APIs, and ensure at least 80% code path coverage.

## Steps
1. Create tests/ directory with integration test files.
2. Set up test fixtures: a test database with migrations applied, mock HTTP servers (using wiremock-rs or similar) for all four external APIs.
3. Test scenarios:
   a. Full vetting pipeline with all APIs returning positive data -> expect Hot lead.
   b. Full pipeline with mixed data -> expect Warm/Cold lead.
   c. Pipeline with one API failing -> expect partial results with reduced confidence.
   d. Pipeline with all APIs failing -> expect Disqualified or error.
   e. Duplicate vetting run for same org -> expect updated results.
   f. GET endpoints return correct data after POST run.
   g. GET endpoints return 404 for unknown org.
4. Verify database state after each test (correct rows in vetting_results and lead_scores tables).
5. Run `cargo tarpaulin` or similar to verify >= 80% code coverage.
6. Add CI-friendly test configuration.

## Validation
All integration tests pass; mock external API servers simulate realistic responses; code coverage report shows >= 80% of code paths exercised; tests are deterministic and can run in CI.