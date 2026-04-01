## Acceptance Criteria

- [ ] 1. Unit test with mocked Hermes API: verify research memo contains 'Hermes Research Findings' header and at least one formatted entry when API returns valid results. 2. Unit test with NOUS_API_KEY unset: verify deliberation completes without error and research memo does not contain Hermes section. 3. Unit test: verify results with relevance_score < 0.5 are excluded. 4. Integration test: run deliberation with live NOUS_API_KEY; confirm research memo file in deliberation artifacts directory contains Hermes-sourced content with valid URLs. 5. Verify timeout handling: mock a 35-second delay and confirm the call times out gracefully with a logged warning.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.