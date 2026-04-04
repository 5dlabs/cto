Implement task 8: End-to-End Pipeline Integration Test (Tess - Test frameworks)

## Goal
Comprehensive E2E integration test that validates all PRD acceptance criteria: pipeline completes without fatal errors, Linear session created with issues having delegate_id set, PR created in sigma-1, and Discord notifications fired. This test does NOT depend on Task 7 (dashboard) per the scope decision D-SCOPE.

## Task Context
- Agent owner: tess
- Stack: Test frameworks
- Priority: high
- Dependencies: 2, 3, 4, 5, 6

## Implementation Plan
Step-by-step implementation:

1. Test framework setup:
   - Use Bun test runner (consistent with cto-pm stack) or Vitest
   - Configure test environment with access to in-cluster services or appropriate mocks
   - Test timeout: 5 minutes (pipeline runs take minutes per D7 discussion)

2. Mocking strategy (Open Question #5):
   - **Linear API**: Use live API with a test project/team if available, OR mock with recorded responses. If live, ensure idempotency by using unique run IDs and cleaning up after tests.
   - **Discord bridge**: Use live in-cluster bridge (preferred — it's in-cluster and low-risk) OR capture outbound requests via an HTTP interceptor.
   - **GitHub API**: Use live API to create a real PR (validates the full flow) OR mock if rate limits are a concern. If live, create PRs on a test branch prefix.
   - **Hermes/NOUS**: Accept whatever the environment provides (live Hermes, NOUS fallback, or skip) — validate the outcome matches the three-tier logic.

3. Test: Pipeline completion (AC-1)
   - Trigger a full pipeline run via cto-pm's intake endpoint
   - Assert pipeline reaches 'complete' status without 'fatal_error' status
   - Timeout: 5 minutes

4. Test: Linear session with issues (AC-2 + AC-3)
   - After pipeline completion, query `GET /api/delegation/status`
   - Assert: response contains >= 5 tasks
   - Assert: >= 5 tasks have `delegation_status: 'assigned'` and non-null `delegate_id`
   - If using live Linear: query Linear API directly to confirm issues exist and have assignees
   - Assert: no tasks that should have been assigned are stuck in 'pending' (cross-reference against known agent mappings)

5. Test: PR creation (AC-4)
   - After pipeline completion, query pipeline state for PR URL
   - Assert: PR URL is non-null and matches pattern `https://github.com/5dlabs/sigma-1/pull/*`
   - If using live GitHub: fetch the PR via API and verify it contains `tasks/` directory with >= 5 files and a valid `pipeline-meta.json`

6. Test: Discord notifications (AC-5)
   - Verify pipeline state or notification logs show at least 2 notification events (start + complete)
   - If using live Discord bridge: verify bridge returned 2xx for both calls
   - Assert: start notification was sent before complete notification (timestamp ordering)

7. Test: Research integration
   - Check validation report for `research_included` field
   - If true: verify deliberation output contains non-empty research memo files
   - If false: verify skip reason is logged and pipeline still completed successfully

8. Test: Graceful degradation
   - If environment allows: temporarily disable Hermes and NOUS_API_KEY
   - Run pipeline and assert it completes with `research_included: false` and no fatal errors

9. Cleanup:
   - If live Linear: close/archive test issues
   - If live GitHub: close test PR, delete test branch
   - Log full test results with pass/fail for each acceptance criterion

## Acceptance Criteria
1. AC-1: Pipeline run completes with status 'complete' within 5 minutes — no 'fatal_error' status at any stage. 2. AC-2: `GET /api/delegation/status` returns >= 5 tasks in the response array. 3. AC-3: At least 5 tasks in the response have `delegate_id` set to a non-null string matching Linear user ID format. Direct Linear API query (if live) confirms each issue's `assignee.id` is non-null. 4. AC-4: Pipeline state contains a PR URL matching `https://github.com/5dlabs/sigma-1/pull/\d+`. PR API response (if live) shows state 'open' and contains files in `tasks/` directory. 5. AC-5: Notification log or bridge response records show >= 2 events with HTTP 2xx responses, including both 'pipeline_start' and 'pipeline_complete' event types. 6. Research: validation report `research_included` is boolean; if true, at least one research memo file exists with content length > 0. 7. All 5 core acceptance criteria pass in a single test run.

## Subtasks
- Configure Bun test runner and E2E environment bootstrap: Set up the Bun test runner for the E2E test suite, configure environment variables for all external service endpoints (cto-pm intake, Linear, GitHub, Discord bridge, Hermes/NOUS), set the global test timeout to 5 minutes, and create shared test utilities (unique run ID generator, HTTP client wrappers, assertion helpers).
- Implement mock/live adapters for Linear API: Create an adapter layer for Linear API interactions that supports both live API calls and recorded mock responses. The adapter must support: creating test issues, querying issues by assignee/delegate_id, and cleanup (close/archive). Implement the mock variant with realistic recorded payloads.
- Implement mock/live adapters for GitHub API: Create an adapter layer for GitHub API interactions supporting both live PR verification and mock responses. The adapter must support: fetching PR details by URL, verifying PR contains expected files, and cleanup (close PR, delete branch).
- Implement Discord notification verification adapter: Create an adapter for verifying Discord notification delivery. Supports verifying that at least 2 notifications (pipeline_start and pipeline_complete) were sent with 2xx responses, with correct timestamp ordering.
- Implement AC-1 test: Pipeline completion without fatal errors: Write the E2E test that triggers a full pipeline run via cto-pm's intake endpoint and asserts it reaches 'complete' status without any 'fatal_error' status within the 5-minute timeout.
- Implement AC-2 and AC-3 tests: Linear session with delegated issues: Write E2E tests verifying that after pipeline completion, the delegation status endpoint returns >= 5 tasks and >= 5 tasks have non-null delegate_id with 'assigned' status. Optionally verify against live Linear API.
- Implement AC-4 test: PR creation in sigma-1: Write E2E test verifying that the pipeline created a PR in the sigma-1 repository with the correct URL pattern, containing tasks/ directory files and pipeline-meta.json.
- Implement AC-5 test: Discord notification delivery: Write E2E test verifying that at least 2 Discord notifications were sent (pipeline_start and pipeline_complete) with successful HTTP responses and correct timestamp ordering.
- Implement research integration verification test: Write E2E test that verifies the research_included field in the validation report and conditionally checks for research memo content or a valid skip reason.
- Implement graceful degradation test (Hermes/NOUS disabled): Write E2E test that verifies the pipeline completes successfully even when Hermes and NOUS are unavailable, with research_included set to false and no fatal errors.
- Implement test cleanup and result aggregation: Implement afterAll cleanup logic that closes/archives test Linear issues, closes test PRs, deletes test branches, and produces a structured test result summary with pass/fail per acceptance criterion.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.