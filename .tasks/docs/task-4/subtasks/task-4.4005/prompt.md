Implement subtask 4005: Implement error handling and graceful degradation for GitHub API failures

## Objective
Wrap all GitHub API interactions with error handling that logs failures clearly, marks the PR step as failed in pipeline state, and ensures the pipeline continues without crashing.

## Steps
1. Create `src/pr/error-handler.ts`.
2. Define a `PRStepResult` type: `{ success: boolean; prUrl?: string; error?: string; errorCode?: number }`.
3. Implement `withGracefulDegradation<T>(fn: () => Promise<T>, stepName: string): Promise<{ ok: true; value: T } | { ok: false; error: string }>` wrapper that catches errors, logs them with context (step name, HTTP status if available, message about token permissions for 403/404), and returns a failure result.
4. In `github-client.ts`, wrap `createBranch` and `commitFileTree` calls with this handler.
5. In `pr-creator.ts`, wrap `createPullRequest` and `addLabels` with this handler.
6. For 403 errors, log: `GitHub API 403: Check that sigma-1-github-token has repo scope and access to 5dlabs/sigma-1`.
7. For 404 errors, log: `GitHub API 404: Repository 5dlabs/sigma-1 not found — verify token has access to this private repository`.
8. For branch-already-exists (422 on ref creation), implement retry with counter suffix (coordinate with logic in 4003).
9. Ensure that if any step fails, the overall pipeline does not throw — return `PRStepResult` with `success: false`.

## Validation
Unit test: `withGracefulDegradation` catches a thrown error and returns `{ ok: false, error: '...' }` without re-throwing. Unit test: when a mock fetch returns 403, the logged message contains 'token permissions'. Unit test: when a mock fetch returns 404, the logged message contains 'not found'. Integration test: simulate full PR flow where `createPullRequest` returns 500 — verify pipeline state shows `success: false` and no unhandled exception is thrown.