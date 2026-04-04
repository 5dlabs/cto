Implement subtask 2005: Implement GET /api/pipeline/status endpoint

## Objective
Add a REST endpoint GET /api/pipeline/status to the Elysia server that returns pipeline stage, task counts (total, assigned, pending), and stage transition timestamps.

## Steps
1. Add a new Elysia route: `GET /api/pipeline/status`.
2. Query the task store and pipeline state to compute:
   - `stage` — the current pipeline stage (string)
   - `task_count` — total number of tasks
   - `assigned_count` — number of tasks with delegation_status = 'assigned'
   - `pending_count` — number of tasks with delegation_status = 'pending'
   - `failed_count` — number of tasks with delegation_status = 'failed'
   - `stage_transitions` — array of objects with `stage` and `timestamp` for each transition
3. Return as a JSON object with the above fields.
4. Set appropriate Content-Type header (application/json).
5. Handle empty state: return zero counts and current stage even if no tasks exist.
6. Add structured logging for requests to this endpoint.

## Validation
Integration test: call GET /api/pipeline/status and verify the response is valid JSON containing 'stage' (string), 'task_count' (number), 'assigned_count' (number), and 'pending_count' (number) fields. Verify counts are consistent with the actual state. Test that stage_transitions is an array. Test empty pipeline state returns zeros with 200 status.