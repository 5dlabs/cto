## Acceptance Criteria

- [ ] 1. Unit test: mock both bridge endpoints returning 200; assert `notify('pipeline_start', payload)` makes POST requests to both URLs with correct Authorization header and payload shape. 2. Retry test: mock Discord bridge returning 500 twice then 200; assert 3 attempts made, final result is success, 2 retry log entries emitted. 3. Graceful degradation test: mock both bridges returning 500 for all attempts; assert `notify` resolves (does not throw), warn-level logs are emitted for both bridges, and the calling pipeline stage continues. 4. Integration test: trigger a full pipeline run; assert at least 2 notification POST requests were made (one start, one complete) by inspecting PM server logs for `stage: 'notification'` entries. 5. Auth test: assert all outbound requests include the `Authorization: Bearer` header with a non-empty value.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.