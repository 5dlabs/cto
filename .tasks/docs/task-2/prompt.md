Implement task 2: Extend PM Server for Agent Delegation in Linear Issues (Nova - Bun/Elysia)

## Goal
Extend the existing cto/cto-pm Bun/Elysia service to resolve agent hints to Linear user IDs via resolve_agent_delegates() and set delegate_id on Linear issues at creation time. Ensure backward compatibility: if agent mapping is unavailable for a task, fall back to agent:pending label rather than failing.

## Task Context
- Agent owner: nova
- Stack: Bun/Elysia
- Priority: high
- Dependencies: 1

## Implementation Plan
Step-by-step implementation:

1. Audit the existing `resolve_agent_delegates()` function in cto/cto-pm:
   - Identify the current mapping source (hardcoded map, config file, or API lookup — Open Question #3)
   - Verify it covers at least 5 distinct agent → Linear user ID mappings (Bolt, Nova, Blaze, Tess, plus at least one more)
   - If fewer than 5 mappings exist, extend the mapping source with additional agent entries

2. Modify the Linear issue creation flow in cto-pm:
   - Before creating each issue, call `resolve_agent_delegates()` with the task's agent hint
   - If a valid Linear user ID is returned, set `assigneeId` (delegate_id) on the Linear API createIssue mutation
   - If resolution fails or returns null, apply the label `agent:pending` to the issue instead of assigning — do NOT throw

3. Extend the task schema with:
   - `delegate_id: string | null` — the resolved Linear user ID
   - `delegation_status: 'assigned' | 'pending' | 'failed'` — tracks resolution outcome

4. Add a REST endpoint `GET /api/delegation/status` that returns:
   - List of tasks with their `delegate_id`, `delegation_status`, agent hint, and Linear issue URL
   - This endpoint will be consumed by Task 7 (dashboard) if implemented, and by Task 8 (E2E test)

5. Add a REST endpoint `GET /api/pipeline/status` that returns:
   - Current pipeline stage, task count, assigned count, pending count
   - Timestamps for stage transitions

6. Ensure all Linear API calls use the token from ExternalSecret `sigma-1-linear-token` injected via `sigma-1-infra-endpoints` ConfigMap envFrom.

7. Add structured logging for delegation resolution: log agent hint, resolved user ID (or null), and fallback action taken.

## Acceptance Criteria
1. Unit test: `resolve_agent_delegates()` returns valid Linear user IDs for at least 5 known agent hints (bolt, nova, blaze, tess, and one additional). 2. Unit test: when `resolve_agent_delegates()` returns null, the issue creation applies `agent:pending` label instead of assigneeId. 3. Integration test: create a test Linear issue with a known agent hint and verify the response includes a non-null `assigneeId` matching the expected Linear user ID. 4. `GET /api/delegation/status` returns JSON array with at least one entry containing `delegate_id`, `delegation_status`, and `linear_issue_url` fields. 5. `GET /api/pipeline/status` returns valid JSON with `stage`, `task_count`, `assigned_count` fields. 6. Backward compatibility: pipeline does not throw when an unknown agent hint is provided — confirmed by test with hint 'unknown-agent' returning `delegation_status: 'pending'`.

## Subtasks
- Audit and extend resolve_agent_delegates() mapping to cover 5+ agents: Audit the existing resolve_agent_delegates() function in cto/cto-pm to determine the current mapping source and coverage. Extend it to map at least 5 distinct agent hints (bolt, nova, blaze, tess, and at least one more) to their corresponding Linear user IDs.
- Extend task schema with delegate_id and delegation_status fields: Add delegate_id (string | null) and delegation_status ('assigned' | 'pending' | 'failed') fields to the task schema/type definitions used in cto-pm.
- Modify Linear issue creation flow to use delegate_id with agent:pending fallback: Modify the existing Linear issue creation flow in cto-pm to call resolve_agent_delegates() before creating each issue. If a valid user ID is returned, set assigneeId on the createIssue mutation. If resolution fails or returns null, apply the 'agent:pending' label and do NOT throw.
- Implement GET /api/delegation/status endpoint: Add a REST endpoint GET /api/delegation/status to the Elysia server that returns a JSON array of tasks with their delegate_id, delegation_status, agent hint, and Linear issue URL.
- Implement GET /api/pipeline/status endpoint: Add a REST endpoint GET /api/pipeline/status to the Elysia server that returns pipeline stage, task counts (total, assigned, pending), and stage transition timestamps.
- Add structured logging for delegation resolution across the flow: Ensure all delegation-related operations (resolution, issue creation, fallback) produce structured log entries with agent hint, resolved user ID, and action taken.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.