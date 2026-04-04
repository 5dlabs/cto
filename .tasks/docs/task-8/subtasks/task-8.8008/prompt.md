Implement subtask 8008: Implement AC-5 test: Discord notification delivery

## Objective
Write E2E test verifying that at least 2 Discord notifications were sent (pipeline_start and pipeline_complete) with successful HTTP responses and correct timestamp ordering.

## Steps
Step-by-step:
1. In `tests/e2e/pipeline.test.ts`, create test: `test('AC-5: Discord notifications sent', async () => { ... })`.
2. Using the run ID from AC-1 test, call `discordAdapter.getNotifications(runId)`.
3. Assert: returned array has >= 2 notification records.
4. Call `discordAdapter.verifyNotificationPair(records)`.
5. Assert: `result.hasBothEvents === true` (both 'pipeline_start' and 'pipeline_complete' present).
6. Assert: `result.bothSuccessful === true` (both HTTP status codes are 2xx).
7. Assert: `result.correctOrder === true` (start timestamp < complete timestamp).

## Validation
Test passes when >= 2 notifications are recorded, both pipeline_start and pipeline_complete events exist with 2xx HTTP status, and start timestamp precedes complete timestamp. Test fails if any event is missing or returned non-2xx.