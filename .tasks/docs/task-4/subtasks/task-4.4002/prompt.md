Implement subtask 4002: Implement task scaffold file generation

## Objective
Create the logic that transforms PipelineOutput tasks into markdown scaffold files with the correct naming convention and content structure.

## Steps
1. Create `src/design-snapshot/scaffold-generator.ts` with function `generateTaskScaffolds(pipelineOutput: PipelineOutput): Array<{ path: string, content: string }>`.
2. For each task, generate a file with path `tasks/task-<id>-<slug>.md` where slug is derived from the task title (lowercased, spaces replaced with hyphens, special chars stripped).
3. File content should be structured markdown containing: task title as H1, description, agent assignment, dependencies list, acceptance criteria, and research_memo content (if non-null).
4. Also generate deliberation artifact files: for each task with a non-null research_memo, create `deliberation/research-memo-task-<id>.md` containing the raw research memo content, source, and timestamp.
5. Export a slug generation helper for testability.

## Validation
Unit test: Given a PipelineOutput with 5 tasks, generateTaskScaffolds returns exactly 5 task files plus the correct number of deliberation files. Verify naming convention matches `tasks/task-<id>-<slug>.md`. Verify each file's markdown content includes title, description, agent, dependencies, and acceptance criteria.