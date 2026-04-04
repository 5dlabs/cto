## Acceptance Criteria

- [ ] 1. Unit tests for scoring algorithm: parameterized tests covering all boundary conditions — all stages pass (GREEN), mixed results (YELLOW), all stages fail (RED), credit unavailable scenarios. Verify exact point calculations. 2. Unit tests for each pipeline stage with mocked HTTP responses: verify OpenCorporates parsing extracts company status and directors; verify Google Reviews parsing extracts rating and count; verify LinkedIn check correctly identifies presence. 3. Integration test: mock all external APIs using `wiremock-rs`, run full pipeline end-to-end, verify VettingResult stored in DB with correct score. 4. Circuit breaker test: simulate 5 consecutive failures on OpenCorporates mock, verify 6th call returns unavailable without attempting HTTP call. 5. Timeout test: mock slow API response (15s), verify pipeline stage times out at 10s and marks stage as unavailable. 6. GDPR test: create vetting result, call DELETE, verify all data removed. 7. `POST /api/v1/vetting/run` returns 202 and request ID; subsequent GET returns completed result. 8. Minimum 80% coverage.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.