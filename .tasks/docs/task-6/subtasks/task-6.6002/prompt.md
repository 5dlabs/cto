Implement subtask 6002: Implement pipeline trigger test and timing SLA assertion

## Objective
Implement test case 1 (full pipeline execution — POST PRD, assert completion) and test case 6 (pipeline timing < 300s). These are the foundational tests that must pass before any downstream artifact checks.

## Steps
1. In the `describe('Pipeline E2E Completion')` block, add a `beforeAll` that:
   - Reads env vars and validates they are set.
   - Records `startTime = Date.now()`.
   - POSTs the sample PRD fixture to `${PM_SERVER_URL}/api/pipeline/trigger` (or the correct endpoint — check PM server routes).
   - Stores the response body in a shared suite-level variable (`pipelineResult`) for use by subsequent tests.
   - Records `endTime = Date.now()` after the POST resolves.
   - Stores `elapsedMs = endTime - startTime`.
2. If the pipeline endpoint is async (returns immediately with a run ID and requires polling), implement a polling loop:
   - Poll `GET /api/pipeline/runs/{runId}/status` every 5 seconds.
   - Timeout after 300 seconds.
   - Store the final status response.
3. Test case 1 — `it('should complete the full pipeline')`: Assert response status is 200. Assert response body contains `pipelineRunId` (non-null string). Assert `status` field equals `'complete'`.
4. Test case 6 — `it('should complete within 300 second SLA')`: Assert `elapsedMs < 300000`. Include the actual elapsed time in the assertion message for debugging.
5. Export the `pipelineRunId` to a suite-level variable so downstream test cases can reference it.

## Validation
Test case 1 passes: POST returns 200 with `pipelineRunId` and `status: 'complete'`. Test case 6 passes: elapsed time is under 300 seconds. Both tests fail with descriptive messages if assertions fail (e.g., actual status, actual elapsed time).