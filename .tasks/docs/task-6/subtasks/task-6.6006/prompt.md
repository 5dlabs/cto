Implement subtask 6006: Execute full E2E test suite and generate JUnit XML report

## Objective
Run all 7 test cases in a single test execution against the deployed dev environment, verify all pass, and confirm JUnit XML artifact is generated.

## Steps
1. Ensure required environment variables are set: `PM_SERVER_URL`, `PM_AUTH_TOKEN` (or equivalent auth vars).
2. Run the test command with JUnit reporter enabled, e.g.:
   - Jest: `JEST_JUNIT_OUTPUT_DIR=test-results npx jest e2e/pipeline-completion --reporters=default --reporters=jest-junit`
   - Vitest: `npx vitest run e2e/pipeline-completion --reporter=default --reporter=junit --outputFile=test-results/e2e-pipeline.xml`
3. Verify all 7 test cases pass (exit code 0).
4. Verify the JUnit XML file exists at the expected path and contains 7 test case entries.
5. If any test fails, capture the failure output and categorize it:
   - Environment issue (server unreachable, auth failure) → document fix.
   - Assertion failure (wrong response shape) → adjust test to match actual API response.
   - Timeout → check if 360s is sufficient or if the pipeline is genuinely slow.
6. Document the final test run output (pass/fail counts, elapsed time) as the acceptance gate evidence for Sigma-1.

## Validation
All 7 test cases pass in a single run. Exit code is 0. JUnit XML file is generated at `test-results/e2e-pipeline.xml` and contains exactly 7 test case entries. The XML file is well-formed and parseable.