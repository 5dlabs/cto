Implement subtask 7006: Implement PRGenerator orchestrator service with error handling

## Objective
Create the top-level PRGenerator service that orchestrates the full flow: collect tasks → create branch → generate scaffolds → commit files → create PR, with error handling that logs failures and marks the PR step as failed without crashing the pipeline.

## Steps
1. Create `src/services/pr-generator.ts`.
2. Implement `PRGenerator` class with method `generatePR(runId: string, tasks: TaskMeta[]): Promise<{ success: boolean, prUrl?: string, prNumber?: number, error?: string }>`.
3. Orchestration flow:
   a. Instantiate GitHubClient.
   b. Call `createPipelineBranch(client, runId)` to get baseSha and branchRef.
   c. Generate file entries: for each task, call `generateTaskReadme` and map to `{ path: 'tasks/{taskId}-{slug}/README.md', content }`. Also generate SUMMARY.md at `tasks/SUMMARY.md`.
   d. Call `commitFiles(client, { baseSha, branchRef, files, message })` with commit message `chore: scaffold tasks for pipeline {runId}`.
   e. Call `createPullRequest(client, { runId, taskCount, linearSessionUrl })`.
   f. Return success with PR metadata.
4. Wrap entire flow in try/catch. On GitHubApiError (especially 404), log structured error with runId and endpoint, return `{ success: false, error: message }`. Pipeline must NOT crash.
5. Export this service for use by the pipeline orchestrator after task generation completes.

## Validation
Unit test: mock all sub-services; verify orchestration calls them in correct order with correct arguments. Unit test: mock branch creation throwing 404 GitHubApiError; verify function returns { success: false } with error message and does not throw. Unit test: mock commit step failing; verify PR creation is not attempted and error is returned. Unit test: with 3 tasks, verify file entries array has 4 items (3 READMEs + 1 SUMMARY). Integration test: run full PRGenerator with mocked GitHub API; verify all calls are sequenced correctly and PR metadata is stored.