## Acceptance Criteria

- [ ] 1. Unit test: Given a PipelineOutput with 5 tasks, the module generates exactly 5 task scaffold files with correct naming convention `tasks/task-<id>-<slug>.md`. 2. Unit test: PR body contains task count, list of agent assignments, and research memo availability summary. 3. Unit test: With GITHUB_TOKEN unset, `createSnapshotPR()` logs an error and returns a result indicating PR was skipped, without throwing. 4. Unit test: With a mocked GitHub API returning 500 on first call and 201 on retry, the module succeeds on retry. 5. Integration test: With a mocked GitHub API, run `createSnapshotPR()` with a full PipelineOutput and verify the sequence of API calls: create branch, commit files, create PR.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.