Implement subtask 4004: Implement PR creation with metadata formatting

## Objective
Create a TypeScript/Bun module that opens a pull request on 5dlabs/sigma-1 from the pipeline branch to the default branch, with a formatted title, summary body table, and labels.

## Steps
1. Create `src/pr/pr-creator.ts`.
2. Implement `buildPRTitle(runId: string, taskCount: number, assignedCount: number): string` returning `[Sigma-1 E2E] Pipeline Run {runId} — {taskCount} tasks, {assignedCount} delegated`.
3. Implement `buildPRBody(tasks: { id: number; title: string; agent: string; status: string }[]): string` that produces a markdown table with columns: Task ID, Title, Agent, Status.
4. Implement `createPullRequest(owner: string, repo: string, head: string, base: string, title: string, body: string): Promise<{ url: string; number: number }>` — POST `/repos/{owner}/{repo}/pulls`.
5. Implement `addLabels(owner: string, repo: string, prNumber: number, labels: string[]): Promise<void>` — POST `/repos/{owner}/{repo}/issues/{prNumber}/labels` with labels `['pipeline', 'e2e-validation']`.
6. Implement top-level `openPipelinePR(...)` that orchestrates: build title → build body → create PR → add labels → return PR URL.
7. Use the same authenticated fetch helper from `github-client.ts`.

## Validation
Unit test: `buildPRTitle('run-1', 7, 5)` returns exactly `[Sigma-1 E2E] Pipeline Run run-1 — 7 tasks, 5 delegated`. Unit test: `buildPRBody` with 3 mock tasks produces a markdown table with 3 data rows and correct columns. Integration test (mocked): `openPipelinePR` calls create PR endpoint and add labels endpoint in sequence, returns a PR URL string matching `https://github.com/5dlabs/sigma-1/pull/*`.