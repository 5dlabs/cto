## Acceptance Criteria

- [ ] 1. Unit test: PR creation function generates correct branch name format matching `pipeline/sigma-1-e2e-*`. 2. Unit test: scaffold files are generated for each task with required fields (title, agent, stack, priority, acceptance criteria). 3. Integration test: a PR is created in 5dlabs/sigma-1 with at least the `tasks/` directory containing 5+ scaffold files and a `pipeline-meta.json` with valid JSON. 4. Verify PR title contains the run ID and task count. 5. Verify PR body contains a summary table with agent assignments. 6. Error handling test: when GitHub API returns 403, the pipeline logs the error and continues without crashing — PR step is marked as failed in pipeline state.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.