Implement subtask 8003: Implement mock/live adapters for GitHub API

## Objective
Create an adapter layer for GitHub API interactions supporting both live PR verification and mock responses. The adapter must support: fetching PR details by URL, verifying PR contains expected files, and cleanup (close PR, delete branch).

## Steps
Step-by-step:
1. Create `tests/e2e/adapters/github.ts` with interface `GitHubTestAdapter` containing methods: `verifyPR(prUrl: string): Promise<PRVerification>`, `listPRFiles(prUrl: string): Promise<string[]>`, `cleanup(prUrl: string): Promise<void>`.
2. Define `PRVerification` type: `{ state: string, url: string, files: string[], hasPipelineMeta: boolean, taskFileCount: number }`.
3. Implement `LiveGitHubAdapter`: uses `GITHUB_TOKEN`, calls GitHub REST API (`GET /repos/5dlabs/sigma-1/pulls/:number`, `GET /repos/5dlabs/sigma-1/pulls/:number/files`). Cleanup closes PR and deletes the source branch.
4. Implement `MockGitHubAdapter`: returns recorded payloads from `tests/e2e/fixtures/github/`. Include fixture with PR state 'open', >= 5 files in `tasks/` directory, and a `pipeline-meta.json`.
5. Factory function `createGitHubAdapter()` selects based on `E2E_GITHUB_MODE` env var (default: 'mock').

## Validation
MockGitHubAdapter returns a PR verification with state 'open', taskFileCount >= 5, and hasPipelineMeta true. LiveGitHubAdapter (if token available) can fetch a known existing PR without errors. Factory correctly switches on env var.