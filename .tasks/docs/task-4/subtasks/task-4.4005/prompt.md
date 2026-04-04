Implement subtask 4005: Implement error handling with retry logic for GitHub API failures

## Objective
Add retry logic with exponential backoff for GitHub API calls and ensure the pipeline never fails due to GitHub API unavailability.

## Steps
1. Create a reusable `retryWithBackoff(fn, maxRetries=1, baseDelay=1000)` utility in `src/design-snapshot/retry.ts`.
2. Wrap the key GitHub API operations (branch creation, file commit, PR creation) with the retry utility.
3. On first failure (non-2xx response or network error), wait `baseDelay` ms then retry once.
4. If the retry also fails, log the error with details (HTTP status, error message, operation name) and return a PRResult with `{ prUrl: null, skipped: false, error: '<descriptive error>' }`.
5. Ensure no unhandled exceptions escape createSnapshotPR — all errors must be caught and converted to PRResult responses.
6. Integrate the retry logic into the main createSnapshotPR orchestration flow.

## Validation
Unit test: With a mocked GitHub API returning 500 on first call and 201 on retry, createSnapshotPR succeeds and returns a valid PR URL. Unit test: With a mocked GitHub API returning 500 on both attempts, createSnapshotPR returns PRResult with prUrl=null and a descriptive error, without throwing. Verify the backoff delay is applied between attempts.