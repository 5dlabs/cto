Implement subtask 6005: Implement no-fatal-errors log validation test

## Objective
Implement test case 7 — assert PM server logs for the pipeline run contain zero fatal-level entries.

## Steps
1. Test case 7 — `it('should have zero fatal errors in PM server logs')`:
   - Query PM server for logs associated with the pipeline run. Try `GET ${PM_SERVER_URL}/api/pipeline/runs/${pipelineRunId}/logs` first.
   - If a dedicated log endpoint doesn't exist, check the pipeline run result object for an embedded `logs` or `errors` array.
   - Parse the log entries and filter for entries where `level === 'fatal'` OR (`level === 'error'` AND `fatal === true`).
   - Assert the filtered array length is 0.
   - On failure, include the first 3 fatal log entries in the assertion message for debugging.
2. If no log API endpoint exists, implement a fallback approach:
   - Check the pipeline run response for an `errors` array.
   - Assert it is empty or contains no fatal-level entries.
3. Add a descriptive skip message if the log endpoint returns 404, so the test is explicitly skipped rather than silently passing: `it.skip('Log endpoint not available — manual log review required')`.
4. This test can run in parallel with 6003 and 6004 since it only depends on the pipeline run ID from 6002.

## Validation
Test case 7 passes: zero log entries with `level: 'fatal'` or `level: 'error'` + `fatal: true` are found. If the log endpoint is unavailable, the test is explicitly skipped with a clear message rather than passing vacuously.