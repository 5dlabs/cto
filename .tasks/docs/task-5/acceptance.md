## Acceptance Criteria

- [ ] 1. Unit test: `notify('pipeline.start')` sends an HTTP POST to DISCORD_BRIDGE_URL with a JSON body containing event='pipeline.start', pipeline_id, and timestamp. 2. Unit test: `notify('pipeline.complete')` payload includes task_count >= 5, assigned_count, pr_url, and linear_session_url. 3. Unit test: When DISCORD_BRIDGE_URL is unreachable (connection refused), `notify()` logs a warning and resolves without throwing. 4. Unit test: When LINEAR_BRIDGE_URL returns 500, `notify()` logs a warning and resolves without throwing. 5. Integration test: With mocked bridge HTTP endpoints, run the full pipeline and verify exactly 2 notification calls per bridge (start + complete), with correct payloads.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.