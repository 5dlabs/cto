## Acceptance Criteria

- [ ] 1. Unit test: emit issue.created event; verify DiscordNotifier receives a POST with purple embed containing correct issue title and assignee. 2. Unit test: emit 5 events within 1 second; verify only 1 batched Discord message is sent containing all 5 issues. 3. Unit test: set ENABLE_ISSUE_DISCORD_BRIDGE=false; verify no Discord call made. 4. Unit test: simulate Discord API failure; verify issue creation flow is not blocked and error is logged. 5. Integration test: run pipeline creating 5+ issues; verify Discord channel receives batched issue notification(s) with correct assignee names.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.