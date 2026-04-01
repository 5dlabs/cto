Implement subtask 7002: Implement branch creation from latest main SHA via GitHub Refs API

## Objective
Implement a function that fetches the latest commit SHA from the default branch of 5dlabs/sigma-1 and creates a new branch `pipeline/{runId}` pointing to that SHA using the GitHub Git Refs API.

## Steps
1. Create `src/services/branch-creator.ts`.
2. Implement `createPipelineBranch(client: GitHubClient, runId: string): Promise<{ branchRef: string, baseSha: string }>`.
3. Step 1: `GET /repos/5dlabs/sigma-1/git/ref/heads/main` — extract `object.sha` as `baseSha`.
4. Step 2: `POST /repos/5dlabs/sigma-1/git/refs` with body `{ ref: 'refs/heads/pipeline/{runId}', sha: baseSha }`.
5. Return both the branch ref string and the baseSha (needed later for tree creation).
6. If the branch already exists (422 response), handle gracefully — either delete and recreate, or skip creation and fetch existing ref SHA.
7. Propagate GitHubApiError for 404 (repo not found) so the caller can handle it.

## Validation
Unit test: mock GET ref returning a SHA, mock POST refs returning success; verify both calls are made with correct paths and payloads. Unit test: mock 422 on branch creation (already exists); verify graceful handling. Unit test: mock 404 on GET ref; verify GitHubApiError propagates.