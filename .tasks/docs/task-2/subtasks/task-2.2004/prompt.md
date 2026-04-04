Implement subtask 2004: Implement GET /api/delegation/status endpoint

## Objective
Add a REST endpoint GET /api/delegation/status to the Elysia server that returns a JSON array of tasks with their delegate_id, delegation_status, agent hint, and Linear issue URL.

## Steps
1. Add a new Elysia route: `GET /api/delegation/status`.
2. Query the task store for all tasks that have been processed through the delegation flow.
3. Return a JSON array where each entry contains:
   - `task_id` — the task identifier
   - `agent_hint` — the original agent hint string
   - `delegate_id` — the resolved Linear user ID (string | null)
   - `delegation_status` — 'assigned' | 'pending' | 'failed'
   - `linear_issue_url` — the URL of the created Linear issue (string | null)
4. Set appropriate Content-Type header (application/json).
5. Handle empty state gracefully: return an empty array `[]` if no tasks exist.
6. Add input validation: this is a simple GET with no parameters, but ensure proper error handling for internal failures (return 500 with error message).
7. Add structured logging for requests to this endpoint.

## Validation
Integration test: after creating at least one issue through the delegation flow, call GET /api/delegation/status and verify the response is a JSON array with at least one entry containing delegate_id, delegation_status, agent_hint, and linear_issue_url fields. Test empty state: call the endpoint before any issues are created and verify an empty array is returned with 200 status. Test response Content-Type is application/json.