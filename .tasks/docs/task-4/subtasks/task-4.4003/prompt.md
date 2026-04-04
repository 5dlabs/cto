Implement subtask 4003: Implement GitHub API integration for branch creation and file commits

## Objective
Create a TypeScript/Bun module that authenticates with GitHub using the ExternalSecret token and provides functions to create a branch and commit a file tree to it on the 5dlabs/sigma-1 repository.

## Steps
1. Create `src/pr/github-client.ts`.
2. Read the GitHub token from the environment variable populated by `sigma-1-github-token` ExternalSecret (e.g., `process.env.SIGMA1_GITHUB_TOKEN`).
3. Implement `getDefaultBranchSha(owner: string, repo: string): Promise<string>` — GET `/repos/{owner}/{repo}/git/ref/heads/main` to get the latest commit SHA.
4. Implement `createBranch(owner: string, repo: string, branchName: string, sha: string): Promise<void>` — POST `/repos/{owner}/{repo}/git/refs` with `ref: refs/heads/{branchName}`.
5. Implement branch name generator: `buildBranchName(runId: string): string` returning `pipeline/sigma-1-e2e-{runId}`. If branch already exists (422 response), append `-{counter}`.
6. Implement `commitFileTree(owner: string, repo: string, branch: string, files: { path: string; content: string }[], message: string): Promise<string>` using the Git Trees API:
   a. Create blobs for each file.
   b. Create a tree referencing all blobs.
   c. Create a commit pointing to the tree.
   d. Update the branch ref to the new commit SHA.
7. Use Bun's native `fetch` for all HTTP calls. Set headers: `Authorization: Bearer {token}`, `Accept: application/vnd.github+json`, `X-GitHub-Api-Version: 2022-11-28`.
8. Export all functions.

## Validation
Unit test: `buildBranchName('abc123')` returns `pipeline/sigma-1-e2e-abc123`. Unit test: mock fetch to verify `createBranch` sends correct payload to `/repos/5dlabs/sigma-1/git/refs`. Unit test: mock fetch to verify `commitFileTree` creates blobs, tree, commit, and updates ref in correct sequence. Unit test: when `createBranch` receives a 422 response, the function retries with an appended counter suffix.