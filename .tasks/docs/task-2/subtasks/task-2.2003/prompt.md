Implement subtask 2003: Implement idempotent issue creation with duplicate detection

## Objective
Add idempotency logic to the issue creation flow: before creating a Linear issue, compute a deterministic identifier from PRD hash + task ID, query Linear for existing issues with that identifier, and skip creation if a duplicate is found.

## Steps
1. Create `src/pipeline/idempotency.ts`.
2. Implement `computeIdempotencyKey(prdContent: string, taskId: string | number): string` — hash the PRD content (e.g., SHA-256 truncated to 12 chars) and concatenate with taskId to form a deterministic key like `prd:abc123def456:task:3`.
3. Implement `findExistingIssue(linearClient, teamId: string, idempotencyKey: string): Promise<string | null>` — query Linear's `issues` endpoint filtering by the team and searching for the idempotency key string in the issue description. Return the issue ID if found, null otherwise.
4. In the main issue creation orchestrator, before each `issueCreate` call:
   a. Compute the idempotency key.
   b. Call `findExistingIssue()`.
   c. If found, log `{ level: 'info', stage: 'issue_creation', action: 'skipped_duplicate', idempotencyKey }` and continue to the next task.
   d. If not found, embed the idempotency key in the issue description (e.g., as a footer line `<!-- idempotency:prd:abc123:task:3 -->`) and proceed with creation.
5. Ensure the idempotency key is never visible to end users (use HTML comment syntax in Linear markdown).

## Validation
Unit test: computeIdempotencyKey with identical inputs returns identical keys; different inputs return different keys. Integration test: mock Linear's issues query to return an existing issue matching the key — assert issueCreate is NOT called and a 'skipped_duplicate' log is emitted. Mock returning no match — assert issueCreate IS called with the key embedded in the description.