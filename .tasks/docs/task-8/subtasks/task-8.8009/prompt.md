Implement subtask 8009: Implement research integration verification test

## Objective
Write E2E test that verifies the research_included field in the validation report and conditionally checks for research memo content or a valid skip reason.

## Steps
Step-by-step:
1. In `tests/e2e/pipeline.test.ts`, create test: `test('Research integration: validates research_included field', async () => { ... })`.
2. Using the run ID, query the pipeline validation report (e.g., `GET /api/pipeline/:runId/validation` or extract from pipeline status response).
3. Assert: `research_included` field exists and is a boolean.
4. If `research_included === true`:
   a. Query deliberation output for research memo files.
   b. Assert: at least one research memo file exists.
   c. Assert: research memo file content length > 0.
5. If `research_included === false`:
   a. Assert: a skip reason is present in the validation report or pipeline logs.
   b. Assert: pipeline still completed successfully (status is 'complete', not degraded).
6. This test should pass regardless of whether Hermes/NOUS was available — it validates the three-tier fallback logic produced a coherent result.

## Validation
Test passes when research_included is a boolean. If true, at least one non-empty research memo exists. If false, a skip reason is logged and pipeline completed. Test never fails due to research unavailability — it validates the fallback logic.