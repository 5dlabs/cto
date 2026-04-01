Implement subtask 7005: Implement PR creation via GitHub Pulls API and store PR metadata

## Objective
Implement the function that creates a pull request from the pipeline branch to main, formats the PR body with run context and task summary, and stores the resulting PR URL and number in the pipeline run metadata.

## Steps
1. Create `src/services/pr-creator.ts`.
2. Implement `createPullRequest(client: GitHubClient, opts: { runId: string, taskCount: number, linearSessionUrl?: string }): Promise<{ prUrl: string, prNumber: number }>`.
3. Format the PR title as `[Pipeline {runId}] Task Scaffolds`.
4. Format the PR body with: task count, agents involved, link to Linear session (if provided), and a note that these are auto-generated scaffolds.
5. `POST /repos/5dlabs/sigma-1/pulls` with `{ title, body, head: 'pipeline/{runId}', base: 'main' }`.
6. Extract `html_url` and `number` from the response.
7. Store the PR metadata in the pipeline run context — call the existing pipeline metadata store (or accept a callback/store interface) to persist `{ prUrl, prNumber }` keyed by runId.
8. Return the PR URL and number.

## Validation
Unit test: mock POST pulls; verify request body has correct title format, body contains runId and task count, and head/base are correct. Unit test: verify returned prUrl and prNumber match the mock response. Unit test: verify metadata store is called with correct prUrl and prNumber. Unit test: verify PR body includes Linear session URL when provided, and omits it gracefully when not provided.