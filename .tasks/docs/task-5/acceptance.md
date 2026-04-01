## Acceptance Criteria

- [ ] 1. Unit test: mock fetch; call notifyPipelineStart and verify POST body matches expected embed structure with correct color, title, and fields. 2. Unit test: call notifyPipelineComplete with status='success' and verify green embed; call with status='failure' and verify red embed. 3. Unit test: unset DISCORD_WEBHOOK_URL; verify no fetch call made and warning logged. 4. Unit test: mock a 429 response then 200; verify retry fires and succeeds on second attempt. 5. Integration test: trigger a real pipeline run; verify two Discord messages appear in the test channel (start and complete) with correct run ID.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.