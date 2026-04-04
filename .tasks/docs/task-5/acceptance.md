## Acceptance Criteria

- [ ] 1. Unit test: `notifyPipelineStart` constructs a payload with runId, prdTitle, and taskCount fields and sends POST to DISCORD_BRIDGE_URL. 2. Unit test: `notifyPipelineComplete` includes task summary with assigned count and PR URL in the payload. 3. Unit test: when discord-bridge-http returns 5xx, the notification service retries once then logs a warning without throwing. 4. Integration test: trigger a pipeline start event and verify that discord-bridge-http received the notification (check bridge logs or response). 5. Integration test: trigger a pipeline complete event and verify linear-bridge received the notification with session reference. 6. End-to-end: a full pipeline run produces at least 2 Discord notifications (start + complete) — verified by checking Discord channel or bridge response logs.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.