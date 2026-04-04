Implement subtask 8002: Implement Hermes fallback path validation tests (no API key scenario)

## Objective
Add test cases that validate fallback behavior when NOUS_API_KEY is not set: assert deliberation artifacts contain a fallback memo with `source: 'fallback'` and `reason: 'no_api_key'`, and that the pipeline still completes successfully.

## Steps
1. Inside the same test file `e2e/hermes-research-validation.test.ts`, add a `describe.runIf(!hasApiKey())('Fallback path — NOUS_API_KEY absent')` block (or equivalent conditional skip).
2. **Test: Fallback memo structure** — Call `GET /api/pipeline/runs/{runId}/deliberation`. Assert at least one deliberation artifact has `source === 'fallback'` and `reason === 'no_api_key'`. Assert the memo has a non-empty `content` field (even if it is a placeholder explanation).
3. **Test: Pipeline completion** — Assert the pipeline run completed with a success status (not an error or timeout) despite the missing API key. Query `GET /api/pipeline/runs/{runId}` and check `status === 'completed'` or equivalent.
4. **Test: Skipped log entry present** — When running without the key, assert PM server logs contain at least one entry with `stage: 'hermes_research'` and `action: 'skipped'` (the inverse of the primary path assertion).
5. Ensure the `beforeAll` hook from subtask 8001 is shared or re-used so both describe blocks reference the same pipeline run ID.

## Validation
Run the test suite WITHOUT NOUS_API_KEY set. All fallback assertions (fallback memo present with correct source/reason, pipeline completed successfully, skipped log entry present) must pass. Confirm the primary path tests are correctly skipped (not failed) in this configuration.