## Acceptance Criteria

- [ ] 1. Unit test: `resolve_agent_delegates(['bolt', 'nova', 'blaze'])` returns an object mapping each to a valid Linear user ID string. 2. Unit test: `resolve_agent_delegates(['unknown_agent'])` returns null for the unknown agent and logs a warning. 3. Integration test: Run the task generation pipeline with a sample PRD; verify at least 5 task objects have non-null `delegate_id` values. 4. Integration test: Mock the Linear API create-issue endpoint; verify each call includes `assigneeId` matching the task's `delegate_id`. 5. Integration test: Verify the summary log line shows correct counts for assigned vs unresolved.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.