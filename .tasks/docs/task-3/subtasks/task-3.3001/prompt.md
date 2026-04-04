Implement subtask 3001: Define ResearchMemo type and extend task entity type

## Objective
Create the ResearchMemo TypeScript type definition and extend the existing task entity/type to include the research_memo field as ResearchMemo | null.

## Steps
1. In a new file (e.g., `src/hermes-research/types.ts`), define: `export type ResearchMemo = { content: string; source: string; timestamp: Date };`
2. Define `TaskContext` type that captures the task description, dependencies, agent, and any other context needed for the Hermes query.
3. Locate the existing task entity/type definition in the PM server codebase and add `research_memo: ResearchMemo | null` as a field, defaulting to null.
4. Ensure all existing code that creates or serializes task objects handles the new nullable field without breaking.

## Validation
Verify TypeScript compilation passes with the new types. Confirm existing task creation/serialization still works by running any existing tests. Manually inspect that ResearchMemo has exactly three fields (content: string, source: string, timestamp: Date).