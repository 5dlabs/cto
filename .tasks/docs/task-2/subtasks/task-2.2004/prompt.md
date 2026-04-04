Implement subtask 2004: Integrate delegate resolution into the issue creation flow with assigneeId

## Objective
Wire resolve_agent_delegates() into the issue creation pipeline so that each task's agent hint is resolved to a Linear user ID and passed as assigneeId in the issueCreate mutation. Handle unmapped agents gracefully.

## Steps
1. In the issue creation orchestrator (e.g., `src/pipeline/create-issues.ts`), after task generation and before Linear API calls:
   a. Collect all unique agent hints from the generated tasks.
   b. Call `resolve_agent_delegates(agentHints)` once to batch-resolve all delegates.
2. For each task, look up its agent hint in the returned Map:
   a. If a valid Linear user ID is returned, include `assigneeId` in the `issueCreate` mutation variables.
   b. If undefined, omit `assigneeId` from the mutation and log `{ level: 'error', stage: 'delegate_resolution', agent: hint, reason: 'unmapped' }`.
3. Construct the `issueCreate` GraphQL mutation payload including: `title`, `description` (with embedded idempotency key), `teamId`, `assigneeId` (optional), and any labels.
4. Ensure the pipeline continues creating remaining issues even if one delegate resolution fails — no short-circuiting.

## Validation
Integration test: provide a mock PRD generating 5 tasks with agents ['nova', 'bolt', 'blaze', 'grizz', 'unknown']. Assert 5 issueCreate calls were made. Assert 4 include a non-null assigneeId. Assert 1 (unknown) has no assigneeId and an error log with stage 'delegate_resolution' was emitted.