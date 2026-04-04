## Validate End-to-End Pipeline Completion (Tess - Test frameworks)

### Objective
Build and execute an end-to-end test suite that validates the full intake pipeline completes through all stages — deliberation, task generation, issue creation with agent assignments, and notifications — for a single PRD. This is the primary acceptance gate for the Sigma-1 E2E validation run.

### Ownership
- Agent: tess
- Stack: Test frameworks
- Priority: high
- Status: pending
- Dependencies: 2, 3, 4, 5

### Implementation Details
1. Determine the existing test framework in the repository (Jest, Vitest, or other). Use what's configured; do not introduce a new framework.
2. Create a test file `e2e/pipeline-completion.test.ts` (or matching existing convention).
3. Test case 1 — Full pipeline execution: POST a sample PRD to the PM server's pipeline trigger endpoint. Assert: response status 200, response body contains `pipelineRunId` and `status: 'complete'`.
4. Test case 2 — Task generation: query the pipeline results (GET `/api/pipeline/runs/{runId}/tasks`). Assert: at least 5 tasks returned, each with a non-empty `title`, `agent`, and `stack` field.
5. Test case 3 — Agent assignments: for each generated task, assert the `delegate_id` or `assigneeId` field is present and non-null for at least 80% of tasks (allowing for unmapped agents).
6. Test case 4 — Linear session: query pipeline results for Linear session metadata. Assert: `linearSessionId` is non-null, `issueCount >= 5`.
7. Test case 5 — PR creation: query pipeline results for PR metadata. Assert: `prUrl` is non-null and matches `https://github.com/5dlabs/sigma-1/pull/\d+` pattern.
8. Test case 6 — Pipeline timing: assert total pipeline execution time is less than 300 seconds (5 minute SLA).
9. Test case 7 — No fatal errors: assert PM server logs for the run contain zero entries with `level: 'fatal'` or `level: 'error'` with `fatal: true`.
10. Set test timeout to 360 seconds to accommodate external API latency.
11. Use environment variables for PM server URL and auth credentials. Document required env vars in the test file header.

### Subtasks
- [ ] Detect existing test framework and configure E2E test scaffold: Inspect the repository for the configured test framework (Jest, Vitest, or other), identify the existing test directory convention, and create the foundational test file `e2e/pipeline-completion.test.ts` with proper imports, describe blocks, global timeout configuration (360s), and environment variable documentation.
- [ ] Implement pipeline trigger test and timing SLA assertion: Implement test case 1 (full pipeline execution — POST PRD, assert completion) and test case 6 (pipeline timing < 300s). These are the foundational tests that must pass before any downstream artifact checks.
- [ ] Implement task generation and agent assignment validation tests: Implement test case 2 (task generation — at least 5 tasks with required fields) and test case 3 (agent assignments — >= 80% delegate coverage).
- [ ] Implement Linear session and PR creation validation tests: Implement test case 4 (Linear session — linearSessionId non-null, issueCount >= 5) and test case 5 (PR creation — prUrl matches GitHub pattern).
- [ ] Implement no-fatal-errors log validation test: Implement test case 7 — assert PM server logs for the pipeline run contain zero fatal-level entries.
- [ ] Execute full E2E test suite and generate JUnit XML report: Run all 7 test cases in a single test execution against the deployed dev environment, verify all pass, and confirm JUnit XML artifact is generated.