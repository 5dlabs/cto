Implement subtask 7003: Implement scaffold file content generators for per-task README and SUMMARY.md

## Objective
Create pure functions that generate markdown content for each task's scaffold README.md and the root SUMMARY.md table. These are pure content generators with no API dependencies.

## Steps
1. Create `src/services/scaffold-generator.ts`.
2. Define a `TaskMeta` type: `{ id: number, title: string, agent: string, stack: string, description: string, details: string, testStrategy: string, priority: string, dependencies: number[] }`.
3. Implement `generateTaskReadme(task: TaskMeta): string` that produces markdown with sections: `# {title}`, `**Agent:** {agent}`, `**Stack:** {stack}`, `## Description`, `## Implementation Details`, `## Test Strategy`.
4. Implement `generateSlug(task: TaskMeta): string` that produces a kebab-case slug from the title (max 50 chars, alphanumeric and hyphens only).
5. Implement `generateSummaryMd(tasks: TaskMeta[], runId: string): string` that produces a markdown document with:
   - Header: `# Pipeline {runId} — Task Scaffolds`
   - A markdown table with columns: ID, Title, Agent, Stack, Priority, Dependencies.
   - A dependency graph section showing which tasks depend on which (simple textual representation like `Task 3 → Task 1, Task 2`).
6. All functions are pure — no side effects, no API calls.

## Validation
Unit test: call generateTaskReadme with sample TaskMeta; verify output contains all required sections and field values. Unit test: call generateSlug with various titles including special characters; verify output is valid kebab-case. Unit test: call generateSummaryMd with 3 sample tasks; verify markdown table has 3 rows, correct headers, and dependency graph lists all edges. Unit test: verify SUMMARY.md with 0 tasks produces a valid document with empty table.