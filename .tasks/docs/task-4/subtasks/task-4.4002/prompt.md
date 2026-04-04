Implement subtask 4002: Implement task scaffold markdown file generation

## Objective
Create a TypeScript/Bun module that takes an array of pipeline tasks and generates one markdown scaffold file per task, plus a pipeline-meta.json file, returning them as an in-memory file tree.

## Steps
1. Create `src/pr/scaffold-generator.ts`.
2. Define a `PipelineTask` type with fields: title, agent, stack, priority, details (summary), test_strategy (acceptance criteria).
3. Implement `generateTaskScaffold(task: PipelineTask): { path: string; content: string }` that produces a markdown file at `tasks/task-{id}-{slug}.md` with sections: Title, Agent, Stack, Priority, Implementation Details Summary, Acceptance Criteria.
4. Implement `generatePipelineMeta(runId: string, tasks: PipelineTask[]): { path: string; content: string }` that produces `pipeline-meta.json` with fields: runId, timestamp (ISO 8601), taskCount, agentAssignments (map of agent → task IDs).
5. Implement `generateDesignSnapshot(hasExisting: boolean, existingArtifacts?: unknown[]): { path: string; content: string }[]` — if `hasExisting` is true, format provided artifacts into `design-snapshots/` files; otherwise generate `design-snapshots/snapshot-summary.md` placeholder documenting the pipeline run.
6. Implement `generateDeliberationFiles(researchMemos: string[], decisions: string): { path: string; content: string }[]` for `deliberation/research-memos/` and `deliberation/decisions.md`.
7. Implement top-level `buildFileTree(...)` that combines all generators and returns a flat array of `{ path, content }` entries.
8. Export all functions for unit testing.

## Validation
Unit test: call `generateTaskScaffold` with a mock task and assert the returned markdown contains all required sections (title, agent, stack, priority, acceptance criteria). Unit test: call `generatePipelineMeta` and JSON.parse the output — assert runId, timestamp, taskCount, and agentAssignments are present and correctly typed. Unit test: `buildFileTree` with 5+ mock tasks returns at least 5 files in `tasks/` directory plus `pipeline-meta.json`.