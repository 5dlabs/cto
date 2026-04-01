Implement subtask 8008: Implement Test Case 5: PR Creation Verification via GitHub API

## Objective
Write the E2E test that fetches PR metadata from the pipeline and verifies the PR exists on GitHub with the correct repo, status, and scaffold file count.

## Steps
1. Create `tests/e2e/helpers/github-client.ts` — a minimal GitHub API client using fetch:
   a. Constructor takes `GITHUB_TOKEN`.
   b. Method `getPR(owner: string, repo: string, prNumber: number)` calls GitHub REST API.
   c. Method `getPRFiles(owner: string, repo: string, prNumber: number)` returns the list of files changed in the PR.
2. Test Case 5 (`it('creates a PR in 5dlabs/sigma-1 with >= 5 scaffold files')`):
   a. Call `GET ${PM_SERVER_URL}/api/pipeline/${runId}/pr`.
   b. Parse response — expect `{ prUrl, prNumber, prStatus }` or similar.
   c. Assert: `prUrl` is non-null and contains '5dlabs/sigma-1'.
   d. Extract `owner`, `repo`, `prNumber` from the URL.
   e. Call `githubClient.getPR(owner, repo, prNumber)`.
   f. Assert: PR state is 'open'.
   g. Call `githubClient.getPRFiles(owner, repo, prNumber)`.
   h. Assert: `files.length >= 5`.
   i. Log the list of files for debugging.
3. If GITHUB_TOKEN is not set, skip this test with a clear message.

## Validation
Test passes when pipeline returns a valid PR URL pointing to 5dlabs/sigma-1, GitHub API confirms the PR is 'open', and the PR contains >= 5 changed files. Test skips gracefully without GITHUB_TOKEN.