Implement subtask 5009: Write comprehensive integration tests with mocked external APIs

## Objective
Create end-to-end integration tests that exercise the full vetting pipeline using mocked external API responses, verifying correct data flow from request through pipeline to database storage.

## Steps
1. Set up a test harness using axum::test and sqlx test fixtures with a test PostgreSQL database. 2. Create mock HTTP servers (using wiremock-rs or similar) for OpenCorporates, LinkedIn, Google Reviews, and credit API. 3. Write test scenarios: a) Happy path: all APIs return valid data, verify VettingResult stored correctly with GREEN score. b) Mixed signals: some APIs return poor data, verify YELLOW classification. c) Red flags: bad credit, no registration, verify RED classification. d) Partial failure: one API returns 500, verify pipeline completes with degraded result. e) All APIs fail: verify appropriate error response. 4. Verify database state after each test (VettingResult and LeadScore rows exist with correct data). 5. Test idempotency: running vetting twice for same org_id creates two results (audit trail).

## Validation
All 5+ test scenarios pass; database assertions confirm correct storage; mock servers receive expected requests; scoring classifications match expected values for each scenario; no test pollution between runs.