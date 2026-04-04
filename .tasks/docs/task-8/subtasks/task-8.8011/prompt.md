Implement subtask 8011: Implement test cleanup and result aggregation

## Objective
Implement afterAll cleanup logic that closes/archives test Linear issues, closes test PRs, deletes test branches, and produces a structured test result summary with pass/fail per acceptance criterion.

## Steps
Step-by-step:
1. In `tests/e2e/pipeline.test.ts`, implement `afterAll(async () => { ... })` block.
2. Linear cleanup: if LiveLinearAdapter was used and issue IDs were collected, call `linearAdapter.cleanup(issueIds)` to archive/close all test issues. Log success/failure of each cleanup action. Do not throw — cleanup failures should warn, not fail the test suite.
3. GitHub cleanup: if LiveGitHubAdapter was used and PR URL was collected, call `githubAdapter.cleanup(prUrl)` to close the PR and delete the test branch. Log success/failure.
4. Graceful degradation cleanup: if a second pipeline run was triggered, ensure its resources are also cleaned up.
5. Result aggregation: create a structured summary object: `{ ac1: pass/fail, ac2: pass/fail, ac3: pass/fail, ac4: pass/fail, ac5: pass/fail, research: pass/fail, gracefulDegradation: pass/fail/skipped }`.
6. Log the summary to stdout in a human-readable format.
7. Write the summary to `tests/e2e/results/last-run.json` for CI artifact collection.
8. Ensure all cleanup is wrapped in try/catch so one cleanup failure doesn't prevent others.

## Validation
Cleanup runs even if tests fail (afterAll guarantee). Linear issues are archived if live mode was used. GitHub PRs are closed if live mode was used. Result summary JSON is written to disk. No cleanup failure causes the test suite to throw.