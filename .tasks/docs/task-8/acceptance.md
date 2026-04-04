## Acceptance Criteria

- [ ] 1. When NOUS_API_KEY is set: deliberation artifacts contain at least one memo with `source: 'hermes'` and `content.length > 100`. Circuit breaker reports `closed` state. No `skipped` log entries for hermes_research stage. 2. When NOUS_API_KEY is not set: deliberation artifacts contain a fallback memo with `source: 'fallback'` and `reason: 'no_api_key'`. Pipeline still completes successfully. 3. At least one of the two paths (with key or without) must execute and pass in the test run, depending on environment configuration.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.