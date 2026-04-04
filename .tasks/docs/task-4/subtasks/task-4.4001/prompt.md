Implement subtask 4001: Create design-snapshot module interface, types, and GitHub API client setup

## Objective
Set up the design-snapshot module with the createSnapshotPR function signature, define PipelineOutput and PRResult types, and configure the GitHub API client with GITHUB_TOKEN from environment.

## Steps
1. Create `src/design-snapshot/types.ts` defining: `PipelineOutput` (containing tasks array with id, title, slug, description, agent, dependencies, acceptance_criteria, research_memo), `PRResult` (containing prUrl: string | null, skipped: boolean, error?: string), and `TaskScaffold` (the shape of a generated task file).
2. Create `src/design-snapshot/index.ts` exporting `async function createSnapshotPR(pipelineOutput: PipelineOutput): Promise<PRResult>`.
3. Read `GITHUB_TOKEN` from `process.env.GITHUB_TOKEN` (or `Bun.env`). If missing, log an error and return `{ prUrl: null, skipped: true, error: 'GITHUB_TOKEN not configured' }` immediately.
4. Set up the GitHub API client (Octokit instance or a configured fetch wrapper) with the token in the Authorization header and base URL pointing to `https://api.github.com`.
5. Define constants: `REPO_OWNER = '5dlabs'`, `REPO_NAME = 'sigma-1'`, `BASE_BRANCH = 'main'`.

## Validation
TypeScript compilation passes. Unit test: with GITHUB_TOKEN unset, createSnapshotPR returns PRResult with skipped=true and prUrl=null without throwing. Verify the error log message is emitted.