Implement subtask 8004: Implement Test Case 1: Full Pipeline Completion

## Objective
Write the E2E test that submits a sample PRD to the PM server intake endpoint, polls for pipeline completion, and asserts the pipeline reaches 'completed' status without fatal errors.

## Steps
1. In `sigma1-e2e.test.ts`, create a `describe('Full Pipeline E2E')` block.
2. In a `beforeAll` hook: start the Discord collector, clear its messages, then POST the sample PRD from `fixtures/sample-prd.json` to `POST ${PM_SERVER_URL}/api/pipeline/intake`. Capture the returned `runId`.
3. Test Case 1 (`it('completes pipeline within 5 minutes')`):
   a. Use the `poll` helper to call `GET /api/pipeline/${runId}/status` every 5 seconds.
   b. Predicate: `response.status === 'completed'` or `response.status === 'failed'`.
   c. Timeout: 5 minutes.
   d. Assert `response.status === 'completed'`.
   e. Assert `response.errors` is either absent or an empty array (no fatal errors).
4. Store the `runId` in a module-level variable so subsequent test cases (in other subtasks) can reference it.
5. Handle timeout gracefully: if poll times out, fail with a message including the last observed status and any partial error logs.

## Validation
Test passes when pipeline status reaches 'completed' within 5 minutes. On failure, the error message includes the last status and any error payload. Manually verify the test correctly times out by temporarily shortening the timeout to 10 seconds against a slow/non-existent server.