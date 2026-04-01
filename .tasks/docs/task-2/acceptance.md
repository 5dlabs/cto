## Acceptance Criteria

- [ ] 1. Unit test: mock Linear users API returning 3 known agents; verify resolve_agent_delegates returns correct mapping for all 3 and logs warning for unknown hint. 2. Integration test: create a test issue via the PM server with agent hint 'nova'; verify the Linear API response includes the correct assigneeId. 3. Verify at least 5 issues created in a pipeline run have non-null assigneeId fields by querying Linear API. 4. Confirm no issues carry the legacy 'agent:pending' label.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.