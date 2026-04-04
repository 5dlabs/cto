Implement task 2: Implement Agent Delegate Resolution in PM Server (Nova - Bun/Elysia)

## Goal
Extend the existing PM server to resolve agent hints to Linear user IDs at issue creation time using resolve_agent_delegates(), and set the resolved ID as the Linear assigneeId field on created issues. Invalid delegate IDs must result in unassigned issues with logged errors. Issue creation must be idempotent — re-running the pipeline with the same PRD must not create duplicate issues.

## Task Context
- Agent owner: nova
- Stack: Bun/Elysia
- Priority: high
- Dependencies: 1

## Implementation Plan
1. Mount `sigma-1-infra-endpoints` ConfigMap and external-secrets-managed secrets via `envFrom` in the PM server deployment manifest.
2. Implement or extend `resolve_agent_delegates()` function: accepts an array of agent hint strings, queries an agent-to-Linear-user-ID mapping (source: config file, DB table, or environment-based map), returns a Map<agentHint, linearUserId>.
3. In the issue creation flow, after task generation, look up each task's agent hint via `resolve_agent_delegates()`. If a valid Linear user ID is returned, pass it as `assigneeId` in the Linear API `issueCreate` mutation. If not found or invalid, log a structured error `{ level: 'error', stage: 'delegate_resolution', agent: hint, reason: 'unmapped' }` and create the issue without an assignee.
4. Implement idempotency for issue creation: before creating a Linear issue, query existing issues in the target project with a deterministic identifier (e.g., PRD hash + task ID in the issue description or a custom label). If found, skip creation and log `{ level: 'info', stage: 'issue_creation', action: 'skipped_duplicate' }`.
5. Add retry logic for Linear API calls: 3 retries with exponential backoff (1s, 2s, 4s). Log each retry attempt.
6. Add a new REST endpoint `POST /api/pipeline/delegate-status` that returns the current delegate mapping for observability.
7. Ensure all pipeline stages emit structured JSON logs with a `stage` field for filtering.

## Acceptance Criteria
1. Unit test: `resolve_agent_delegates(['nova', 'bolt', 'blaze', 'tess', 'unknown_agent'])` returns valid Linear user IDs for known agents and `undefined` for unknown agents. 2. Integration test: call the issue creation flow with a mock PRD containing 5 tasks; assert 5 Linear API `issueCreate` mutations were made, at least 4 with a non-null `assigneeId`. 3. Idempotency test: run the same PRD twice; assert the second run creates zero new issues (all skipped). 4. Invalid delegate test: inject an invalid delegate ID; assert the issue is created without an assignee and an error log entry with `stage: 'delegate_resolution'` is emitted. 5. `POST /api/pipeline/delegate-status` returns 200 with a JSON object mapping agent hints to user IDs.

## Subtasks
- Configure ConfigMap and secret mounting for PM server deployment: Update the PM server Kubernetes deployment manifest to mount the sigma-1-infra-endpoints ConfigMap and external-secrets-managed secrets via envFrom, making Linear API tokens and infra endpoints available as environment variables.
- Implement resolve_agent_delegates() function with agent-to-Linear-user-ID mapping: Create the resolve_agent_delegates() function that accepts an array of agent hint strings, looks up each against an agent-to-Linear-user-ID mapping, and returns a Map<string, string | undefined>. Unknown or invalid agents must return undefined.
- Implement idempotent issue creation with duplicate detection: Add idempotency logic to the issue creation flow: before creating a Linear issue, compute a deterministic identifier from PRD hash + task ID, query Linear for existing issues with that identifier, and skip creation if a duplicate is found.
- Integrate delegate resolution into the issue creation flow with assigneeId: Wire resolve_agent_delegates() into the issue creation pipeline so that each task's agent hint is resolved to a Linear user ID and passed as assigneeId in the issueCreate mutation. Handle unmapped agents gracefully.
- Add Linear API retry logic with exponential backoff: Wrap all Linear API calls (issueCreate and issue queries) with retry logic: 3 attempts with exponential backoff (1s, 2s, 4s), structured logging on each retry attempt.
- Implement POST /api/pipeline/delegate-status observability endpoint: Add a new Elysia route POST /api/pipeline/delegate-status that returns the current agent-to-Linear-user-ID delegate mapping as JSON for observability purposes.
- Standardize structured JSON logging across all pipeline stages: Ensure all pipeline stages (delegate resolution, idempotency checks, issue creation, retries, delegate-status endpoint) emit structured JSON logs with a consistent schema including the 'stage' field for filtering.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.