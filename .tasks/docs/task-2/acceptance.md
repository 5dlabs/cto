## Acceptance Criteria

- [ ] 1. Unit test: `resolve_agent_delegates(['nova', 'bolt', 'blaze', 'tess', 'unknown_agent'])` returns valid Linear user IDs for known agents and `undefined` for unknown agents. 2. Integration test: call the issue creation flow with a mock PRD containing 5 tasks; assert 5 Linear API `issueCreate` mutations were made, at least 4 with a non-null `assigneeId`. 3. Idempotency test: run the same PRD twice; assert the second run creates zero new issues (all skipped). 4. Invalid delegate test: inject an invalid delegate ID; assert the issue is created without an assignee and an error log entry with `stage: 'delegate_resolution'` is emitted. 5. `POST /api/pipeline/delegate-status` returns 200 with a JSON object mapping agent hints to user IDs.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.