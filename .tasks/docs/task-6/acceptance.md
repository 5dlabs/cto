## Acceptance Criteria

- [ ] All 7 test cases must pass in a single test run against the deployed dev environment. Specific pass criteria: 1. Pipeline returns `status: 'complete'` with a valid `pipelineRunId`. 2. Task count >= 5 with all required fields populated. 3. >= 80% of tasks have non-null `delegate_id`. 4. Linear session exists with >= 5 issues. 5. PR URL matches GitHub sigma-1 repo pattern. 6. Execution time < 300s. 7. Zero fatal log entries. Test report is generated as a JUnit XML artifact for CI integration.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.