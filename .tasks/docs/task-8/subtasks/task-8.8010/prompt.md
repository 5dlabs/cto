Implement subtask 8010: Implement test suite orchestration: lifecycle hooks, timeout handling, cleanup, and reporting

## Objective
Wire up the full test suite with proper lifecycle management — ordered execution, shared state between test cases, global timeout, post-run cleanup of created resources, and CI-friendly reporting.

## Steps
1. In `sigma1-e2e.test.ts`, ensure test execution order: Test 1 runs first (pipeline trigger + completion), then Tests 2-5 can run in any order, then Test 6 runs last (needs pipeline to have fully completed including notifications).
2. Implement `afterAll` cleanup:
   a. Close the PR created during the test run (via GitHub API) to avoid PR clutter, or add a `[E2E-TEST]` label so it can be filtered.
   b. Stop the Discord collector process.
   c. Log a summary: total tests passed/failed, pipeline runId, elapsed time.
3. Configure the test framework's global timeout to 10 minutes.
4. Add a JUnit XML reporter (or equivalent) so CI can parse and display test results.
5. Add a retry strategy: if Test Case 1 (pipeline completion) fails due to timeout, retry once with a fresh PRD submission.
6. Create a `tests/e2e/README.md` documenting: required env vars, how to run locally, how to run in CI, and how to interpret failures.
7. Verify the entire suite runs end-to-end locally (with real credentials) and produces the expected report output.

## Validation
Full suite runs in under 10 minutes. JUnit XML report is generated and parseable. Cleanup closes or labels the test PR. Discord collector is stopped. README accurately documents the setup process. CI job shows pass/fail status with individual test case results.