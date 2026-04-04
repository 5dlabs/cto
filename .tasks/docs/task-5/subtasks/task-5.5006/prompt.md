Implement subtask 5006: Write unit tests for notification-dispatch module

## Objective
Write comprehensive unit tests covering payload formatting, facade contract, and error handling for the notification-dispatch module.

## Steps
1. Create `src/notification-dispatch/__tests__/notify.test.ts`.
2. Test facade contract: `notify()` with a mock transport verifies send() is called for both 'discord' and 'linear' targets.
3. Test payload formatting for pipeline.start: verify JSON body has event='pipeline.start', pipeline_id, and timestamp.
4. Test payload formatting for pipeline.complete: verify JSON body has task_count >= 5, assigned_count, pr_url, and linear_session_url.
5. Test error handling: mock fetch to throw for discord, verify warning logged and no rejection.
6. Test error handling: mock fetch to return 500 for linear, verify warning logged and no rejection.
7. Test concurrent dispatch: verify both bridges are notified even when one fails.
8. Use Bun's built-in test runner (`bun:test`).

## Validation
All 7+ unit test cases pass. Each test case is isolated with fresh mocks. Coverage of the notification-dispatch module's notify(), HTTP transport send(), and error handling paths is verified.