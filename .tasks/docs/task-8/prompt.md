Implement task 8: End-to-End Pipeline Integration Test (Tess - Test frameworks)

## Goal
Develop and execute a comprehensive E2E integration test that validates the full Sigma-1 pipeline from PRD intake through deliberation, task generation, issue creation with agent delegation, research memo inclusion, PR surfacing, and Discord/Linear notifications.

## Task Context
- Agent owner: tess
- Stack: Test frameworks
- Priority: high
- Dependencies: 2, 3, 4, 5, 6, 7

## Implementation Plan
1. Create a test suite `sigma1-e2e.test.ts` using a test framework compatible with Bun (e.g., bun:test or vitest).
2. Test Case 1 — Full Pipeline Completion:
   a. Submit a sample PRD to the PM server intake endpoint.
   b. Poll the pipeline status endpoint until status is 'completed' or timeout after 5 minutes.
   c. Assert: pipeline status is 'completed', no fatal errors in logs.
3. Test Case 2 — Task Generation with Agent Assignments:
   a. Fetch tasks from `GET /api/pipeline/:runId/tasks`.
   b. Assert: at least 5 tasks returned.
   c. Assert: each task has a non-empty `agent` field.
   d. Assert: at least 3 distinct agent types are present.
4. Test Case 3 — Linear Issues with Delegate IDs:
   a. Fetch issues from `GET /api/pipeline/:runId/issues`.
   b. Assert: issue count >= 5.
   c. Assert: each issue has a non-null `assigneeId` (delegate_id).
   d. Cross-verify with Linear API: query each issue by ID and confirm assignee matches.
5. Test Case 4 — Hermes Research Integration:
   a. Fetch deliberation artifacts from `GET /api/pipeline/:runId/deliberation`.
   b. If NOUS_API_KEY is set: assert research memo contains 'Hermes Research Findings' section with at least one entry.
   c. If NOUS_API_KEY is not set: assert research memo does not contain Hermes section and no errors.
6. Test Case 5 — PR Creation:
   a. Fetch PR metadata from `GET /api/pipeline/:runId/pr`.
   b. Assert: PR URL is non-null and points to 5dlabs/sigma-1.
   c. Assert: PR status is 'open'.
   d. Verify PR contains at least 5 task scaffold files via GitHub API.
7. Test Case 6 — Discord Notifications:
   a. Query a Discord audit log or use a test webhook collector.
   b. Assert: at least 2 messages received (pipeline start + complete).
   c. Assert: start message contains run ID; complete message contains task count.
8. Configure CI to run this suite with real credentials from the sigma1-dev namespace secrets.

## Acceptance Criteria
1. All 6 test cases pass in CI with real infrastructure. 2. Test Case 1: pipeline completes within 5-minute timeout with status 'completed'. 3. Test Case 2: >= 5 tasks with non-empty agent fields and >= 3 distinct agents. 4. Test Case 3: >= 5 Linear issues each with non-null assigneeId verified against Linear API. 5. Test Case 4: Hermes section present in research memo when NOUS_API_KEY is configured. 6. Test Case 5: PR exists in 5dlabs/sigma-1 with >= 5 scaffold files. 7. Test Case 6: Discord webhook collector received >= 2 messages with correct content. 8. Suite completes in under 10 minutes total.

## Subtasks
- Set up E2E test framework and project scaffolding: Initialize the sigma1-e2e test project with the chosen test framework (vitest or bun:test), configure TypeScript, create the `sigma1-e2e.test.ts` entry file, and establish shared utility modules for HTTP requests, polling, and assertions.
- Configure CI credential injection from sigma1-dev namespace secrets: Set up the CI pipeline job to inject real credentials (LINEAR_API_KEY, GITHUB_TOKEN, NOUS_API_KEY, PM_SERVER_URL, DISCORD_COLLECTOR_URL) from sigma1-dev Kubernetes secrets into the E2E test runner environment.
- Implement Discord webhook collector service for notification verification: Build a lightweight HTTP server that acts as a Discord webhook collector — receives POST payloads, stores them in memory, and exposes a GET endpoint for the test suite to query received messages.
- Implement Test Case 1: Full Pipeline Completion: Write the E2E test that submits a sample PRD to the PM server intake endpoint, polls for pipeline completion, and asserts the pipeline reaches 'completed' status without fatal errors.
- Implement Test Case 2: Task Generation with Agent Assignments: Write the E2E test that fetches generated tasks for the pipeline run and asserts correct task count, agent assignment, and agent diversity.
- Implement Test Case 3: Linear Issue Delegation Verification with Cross-API Validation: Write the E2E test that fetches Linear issues created by the pipeline, asserts correct count and delegate assignment, and cross-verifies each issue's assignee against the Linear API.
- Implement Test Case 4: Hermes Research Integration with Conditional Assertions: Write the E2E test that fetches deliberation artifacts and conditionally asserts the presence or absence of Hermes research findings based on NOUS_API_KEY availability.
- Implement Test Case 5: PR Creation Verification via GitHub API: Write the E2E test that fetches PR metadata from the pipeline and verifies the PR exists on GitHub with the correct repo, status, and scaffold file count.
- Implement Test Case 6: Discord Notification Assertions: Write the E2E test that queries the Discord webhook collector and asserts the correct number of messages with expected content (pipeline start and complete notifications).
- Implement test suite orchestration: lifecycle hooks, timeout handling, cleanup, and reporting: Wire up the full test suite with proper lifecycle management — ordered execution, shared state between test cases, global timeout, post-run cleanup of created resources, and CI-friendly reporting.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.