Implement subtask 5004: Implement retry policy with exponential backoff and structured logging

## Objective
Create a generic retry wrapper that applies 3 attempts with exponential backoff (1s, 2s, 4s) to any async function, with structured logging for each attempt and outcome. Wire it into the Discord and Linear notifiers within the dispatcher.

## Steps
1. Create `src/notifications/retry.ts`.
2. Export `withRetry<T>(fn: () => Promise<T>, options: { maxAttempts: number, baseDelayMs: number, label: string }): Promise<T>`.
3. Implement: attempt the function up to `maxAttempts` times. On failure, wait `baseDelayMs * 2^(attempt-1)` ms before retrying (1000, 2000, 4000 for defaults).
4. Log each attempt: `{ level: 'info', stage: 'notification', bridge: options.label, attempt: N, status: 'retrying', error: message }` on failure before retry.
5. Log final failure: `{ level: 'warn', stage: 'notification', bridge: options.label, status: 'failed', attempts: maxAttempts, error: message }`.
6. On final failure, throw the last error (the dispatcher's graceful degradation in 5001 will catch it).
7. Update the dispatcher module to wrap each bridge notifier call with `withRetry(fn, { maxAttempts: 3, baseDelayMs: 1000, label: 'discord' | 'linear' })`.
8. Use a sleepFn parameter (defaulting to `Bun.sleep` or `setTimeout` promise) so tests can inject a no-op sleep.

## Validation
Retry success test: mock function failing twice then succeeding; assert 3 calls total, final result is success, 2 retry log entries emitted. Retry exhaustion test: mock function failing 3 times; assert the function throws after 3 attempts, warn-level log emitted. Backoff test: inject a mock sleep function; assert it was called with 1000ms and 2000ms delays. No-delay test: confirm injectable sleep allows instant test execution.