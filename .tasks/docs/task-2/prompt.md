Implement task 2: Extend PM Server for Agent Delegation in Linear Issues (Nova - Bun/Elysia)

## Goal
Update the PM server's issue creation flow so that agent hints from task decomposition are resolved to Linear user IDs via resolve_agent_delegates(), and the resulting delegate_id is set as the assignee on each Linear issue at creation time. Previously all issues were created unassigned.

## Task Context
- Agent owner: nova
- Stack: Bun/Elysia
- Priority: high
- Dependencies: 1

## Implementation Plan
1. In the PM server task-to-issue mapping module, locate the Linear issue creation call (GraphQL mutation `issueCreate`).
2. Before calling `issueCreate`, invoke `resolve_agent_delegates(agentHints)` which takes an array of agent hint strings (e.g., 'bolt', 'nova', 'blaze') and returns a map of `{ agentHint: linearUserId }`.
3. Implement `resolve_agent_delegates()` if not already present:
   a. Query Linear API `users` endpoint filtered by display name or custom metadata matching agent hints.
   b. Cache results for the duration of the pipeline run to avoid repeated API calls.
   c. If a hint cannot be resolved, log a warning and fall back to unassigned (do NOT set `agent:pending` label).
4. Pass the resolved `linearUserId` as the `assigneeId` field in the `issueCreate` mutation payload.
5. Remove any legacy code that sets `agent:pending` labels as a fallback for assignment.
6. Add structured logging: log each issue ID, title, agent hint, and resolved delegate_id.
7. Ensure the endpoint reads secrets from `sigma1-infra-endpoints` ConfigMap via `envFrom`.

## Acceptance Criteria
1. Unit test: mock Linear users API returning 3 known agents; verify resolve_agent_delegates returns correct mapping for all 3 and logs warning for unknown hint. 2. Integration test: create a test issue via the PM server with agent hint 'nova'; verify the Linear API response includes the correct assigneeId. 3. Verify at least 5 issues created in a pipeline run have non-null assigneeId fields by querying Linear API. 4. Confirm no issues carry the legacy 'agent:pending' label.

## Subtasks
- Implement resolve_agent_delegates() function with Linear Users API query: Create a new TypeScript module exporting resolve_agent_delegates(agentHints: string[]) that queries the Linear GraphQL users endpoint, matches agent hint strings to Linear user profiles, and returns a Map<string, string> of agentHint → linearUserId.
- Add per-pipeline-run caching layer to resolve_agent_delegates(): Wrap the Linear Users API call inside resolve_agent_delegates with an in-memory cache scoped to a single pipeline run so repeated calls within the same run reuse the first result.
- Integrate resolve_agent_delegates into issueCreate mutation flow: Modify the existing task-to-issue mapping module to call resolve_agent_delegates() before each issueCreate GraphQL mutation and pass the resolved linearUserId as the assigneeId field in the mutation payload.
- Remove legacy agent:pending label code: Find and remove all code paths that set or reference the 'agent:pending' label on Linear issues as a fallback assignment mechanism.
- Add structured logging for agent delegation in issue creation: Add structured log entries at each issue creation that include the issue ID, issue title, agent hint, and resolved delegate_id (or 'unassigned').
- Write unit and integration tests for agent delegation flow: Create comprehensive test coverage for resolve_agent_delegates, the updated issueCreate integration, cache behavior, and end-to-end pipeline delegation.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.