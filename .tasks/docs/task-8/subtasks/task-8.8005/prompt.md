Implement subtask 8005: Implement AC-1 test: Pipeline completion without fatal errors

## Objective
Write the E2E test that triggers a full pipeline run via cto-pm's intake endpoint and asserts it reaches 'complete' status without any 'fatal_error' status within the 5-minute timeout.

## Steps
Step-by-step:
1. In `tests/e2e/pipeline.test.ts`, create test: `test('AC-1: Pipeline completes without fatal errors', async () => { ... })`.
2. Generate a unique run ID using `generateRunId()`.
3. Send `POST /api/pipeline/intake` to `CTO_PM_URL` with the run ID and a valid PRD payload (create a minimal test PRD fixture in `tests/e2e/fixtures/test-prd.md`).
4. Assert the POST returns 2xx with a pipeline run ID.
5. Use `waitForPipelineStatus(runId, 'complete', 300_000)` helper to poll `GET /api/pipeline/:runId/status` every 5 seconds.
6. During polling, also check for `fatal_error` status — if encountered, fail immediately with the error details from the response.
7. Assert final status is 'complete'.
8. Store the run ID in a shared test context (module-level variable or `beforeAll` setup) so subsequent tests can reference the same pipeline run.
9. Export the pipeline response for use by downstream tests.

## Validation
Test passes when pipeline reaches 'complete' status. Test fails immediately if 'fatal_error' is encountered. Test fails on timeout after 5 minutes. Polling interval is 5 seconds. Run ID is unique per execution.