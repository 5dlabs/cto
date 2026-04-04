Implement subtask 2005: Write unit tests for resolve_agent_delegates and delegate_id propagation

## Objective
Create comprehensive unit tests covering resolve_agent_delegates mapping, fallback on unknown agents, and delegate_id propagation to the Linear API call layer.

## Steps
1. Create a test file (e.g., `resolve-agent-delegates.test.ts`) using Bun's test runner.
2. Unit tests for resolve_agent_delegates():
   a. Known agents ('bolt', 'nova', 'blaze', 'rex', 'grizz', 'cipher') all resolve to non-empty string Linear user IDs.
   b. Unknown agent returns null in the mapping.
   c. Empty array input returns empty object.
   d. Duplicate agent hints return same ID for each occurrence.
3. Unit tests for delegate_id propagation:
   a. Mock the task generation output with agent hints.
   b. Verify after resolution, each task has the correct delegate_id.
   c. Mock the Linear API client and verify `createIssue` is called with `assigneeId` matching delegate_id for resolved agents.
   d. Verify `createIssue` is called without `assigneeId` for unresolved agents.
4. Use Bun's built-in test and mock utilities. Avoid external test dependencies.

## Validation
All unit tests pass via `bun test`. Test coverage includes: resolve_agent_delegates with known agents, unknown agents, empty input, and duplicates. Delegate_id propagation tests verify the Linear API client receives correct assigneeId values. At least 8 distinct test cases pass.