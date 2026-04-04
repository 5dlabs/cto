Implement subtask 2005: Add Linear API retry logic with exponential backoff

## Objective
Wrap all Linear API calls (issueCreate and issue queries) with retry logic: 3 attempts with exponential backoff (1s, 2s, 4s), structured logging on each retry attempt.

## Steps
1. Create `src/lib/linear-retry.ts`.
2. Implement `withRetry<T>(fn: () => Promise<T>, options: { maxRetries: 3, baseDelayMs: 1000, stage: string }): Promise<T>`.
3. On each failed attempt, log `{ level: 'warn', stage: options.stage, attempt: n, maxRetries: 3, error: err.message, nextRetryMs: delay }`.
4. Use exponential backoff: delay = baseDelayMs * 2^(attempt-1), so 1000ms, 2000ms, 4000ms.
5. After all retries exhausted, log `{ level: 'error', stage: options.stage, action: 'retries_exhausted', error: err.message }` and re-throw the error.
6. Wrap the `issueCreate` mutation call and the `findExistingIssue` query call with `withRetry`, using stage names `'issue_creation'` and `'issue_query'` respectively.
7. Handle Linear-specific rate limit responses (HTTP 429) — if detected, use the `Retry-After` header value instead of the exponential delay.

## Validation
Unit test: mock a function that fails twice then succeeds — assert withRetry returns the success value and logged 2 retry warnings. Mock a function that fails 4 times — assert withRetry throws after 3 retries and logs a 'retries_exhausted' error. Test that 429 responses use Retry-After header delay.