## Acceptance Criteria

- [ ] 1. Unit test: mock Hermes API returning valid research content; assert the deliberation output contains Hermes-sourced content with `source: 'hermes'`. 2. Timeout test: mock Hermes API with 35s delay; assert circuit breaker opens after timeout and fallback memo is generated within 31s. 3. Circuit breaker test: trigger 3 consecutive failures; assert subsequent calls return fallback immediately without HTTP calls for 60s, then half-open allows one probe. 4. No-API-key test: unset `NOUS_API_KEY`; assert deliberation completes with skip log entry and no HTTP calls to Hermes. 5. Integration test: run deliberation with a real or stubbed Hermes endpoint; assert the returned research memo has non-empty `content` field and is persisted in the deliberation artifacts.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.