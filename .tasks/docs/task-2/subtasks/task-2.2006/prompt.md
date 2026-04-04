Implement subtask 2006: Write integration test with mock Linear API validating full delegation pipeline

## Objective
Create an integration test that runs the full task generation and issue creation pipeline against a mock Linear API, verifying at least 5 issues are created with non-null assigneeId and the summary log is correct.

## Steps
1. Create an integration test file (e.g., `delegation-pipeline.integration.test.ts`).
2. Set up a mock Linear API server (using Bun's built-in HTTP server or a mock library) that:
   a. Accepts `createIssue` mutation calls.
   b. Records all received payloads.
   c. Returns valid mock issue responses.
3. Configure the PM server to use the mock Linear API URL.
4. Provide a sample PRD input that generates at least 6 tasks with a mix of known and unknown agent hints.
5. Run the full pipeline: PRD → task generation → agent resolution → Linear issue creation.
6. Assertions:
   a. At least 5 issues were created with non-null `assigneeId`.
   b. Each `assigneeId` matches the expected Linear user ID for the agent hint.
   c. Any unresolved agents resulted in issues without `assigneeId` and with `agent:unresolved` label.
   d. The summary log line is present and shows correct N/M/K counts.
7. Clean up mock server after test completion.

## Validation
Integration test passes via `bun test`. Mock Linear API received at least 6 createIssue calls. At least 5 calls include a non-null `assigneeId`. Unresolved agent issues have the `agent:unresolved` label. Summary log output matches expected assigned/unresolved counts. Test completes in under 10 seconds.