## Acceptance Criteria

- [ ] 1. Unit test: `resolve_agent_delegates()` returns valid Linear user IDs for at least 5 known agent hints (bolt, nova, blaze, tess, and one additional). 2. Unit test: when `resolve_agent_delegates()` returns null, the issue creation applies `agent:pending` label instead of assigneeId. 3. Integration test: create a test Linear issue with a known agent hint and verify the response includes a non-null `assigneeId` matching the expected Linear user ID. 4. `GET /api/delegation/status` returns JSON array with at least one entry containing `delegate_id`, `delegation_status`, and `linear_issue_url` fields. 5. `GET /api/pipeline/status` returns valid JSON with `stage`, `task_count`, `assigned_count` fields. 6. Backward compatibility: pipeline does not throw when an unknown agent hint is provided — confirmed by test with hint 'unknown-agent' returning `delegation_status: 'pending'`.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.