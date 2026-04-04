## Acceptance Criteria

- [ ] 1. Design PR section is reachable from the dashboard navigation. 2. After a pipeline run, at least one PR card renders with: title (non-empty string), status badge, repo name '5dlabs/sigma-1', and ISO-formatted date. 3. GitHub PR link matches expected URL pattern and has `target="_blank"`. 4. PR detail view lists >= 1 scaffold file. 5. Empty state renders correct message when no PRs exist. 6. axe-core accessibility scan returns zero critical/serious violations. 7. Displayed PRs are scoped to the specified pipelineRunId.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.