Implement subtask 8010: Implement graceful degradation test (Hermes/NOUS disabled)

## Objective
Write E2E test that verifies the pipeline completes successfully even when Hermes and NOUS are unavailable, with research_included set to false and no fatal errors.

## Steps
Step-by-step:
1. In `tests/e2e/pipeline.test.ts`, create test: `test('Graceful degradation: pipeline completes without Hermes/NOUS', async () => { ... })`.
2. Check if environment manipulation is possible (e.g., `E2E_ALLOW_ENV_MANIPULATION` env var). If not, skip this test with `test.skip()` and a descriptive message.
3. If allowed:
   a. Save current values of `HERMES_URL` and `NOUS_API_KEY`.
   b. Temporarily unset or set them to invalid values (e.g., `http://localhost:1` for HERMES_URL, empty string for NOUS_API_KEY).
   c. Trigger a new pipeline run with a unique run ID via `POST /api/pipeline/intake`.
   d. Poll for completion using `waitForPipelineStatus()` with 5-minute timeout.
   e. Assert: pipeline status is 'complete' (NOT 'fatal_error').
   f. Assert: validation report shows `research_included: false`.
   g. Restore original env var values.
4. Wrap the entire test in a try/finally to guarantee env var restoration.
5. Note: this is a separate pipeline run from the main AC tests, so it won't interfere with other assertions.

## Validation
Test passes when pipeline completes with status 'complete' and research_included is false despite Hermes/NOUS being unavailable. Test is skipped (not failed) if environment manipulation is not permitted. Environment variables are always restored.