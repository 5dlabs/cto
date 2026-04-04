## Acceptance Criteria

- [ ] 1. >= 80% of created issues (at minimum 4 of 5) have a non-null `assignee` when queried directly via Linear GraphQL API. 2. Each assigned issue's `assignee.id` matches the expected user ID from the delegate mapping. 3. Zero issues carry an `agent:pending` label. 4. Issues without assignees are accounted for by corresponding error log entries. 5. All issues belong to the expected Linear project and have non-empty titles and descriptions.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.