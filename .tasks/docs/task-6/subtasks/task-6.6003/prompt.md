Implement subtask 6003: Implement task generation and agent assignment validation tests

## Objective
Implement test case 2 (task generation — at least 5 tasks with required fields) and test case 3 (agent assignments — >= 80% delegate coverage).

## Steps
1. Test case 2 — `it('should generate at least 5 tasks with required fields')`:
   - GET `${PM_SERVER_URL}/api/pipeline/runs/${pipelineRunId}/tasks`.
   - Assert response status 200.
   - Parse the response body as an array of task objects.
   - Assert `tasks.length >= 5`.
   - For each task, assert:
     - `task.title` is a non-empty string.
     - `task.agent` is a non-empty string.
     - `task.stack` is a non-empty string.
   - Store tasks array in suite-level variable for test case 3.
2. Test case 3 — `it('should assign agents to at least 80% of tasks')`:
   - Iterate over the tasks array.
   - For each task, check if `task.delegate_id` OR `task.assigneeId` is present and non-null.
   - Count assigned vs total.
   - Assert `(assignedCount / totalCount) >= 0.8`.
   - Include actual ratio in assertion message (e.g., `Expected >= 80% assigned, got 3/7 (42.8%)`).
3. Handle field name ambiguity: check for both `delegate_id` and `assigneeId` since the field name may vary. Consider a task assigned if either field is present and non-null.

## Validation
Test case 2 passes: GET returns 200 with an array of >= 5 tasks, each having non-empty `title`, `agent`, and `stack`. Test case 3 passes: >= 80% of tasks have a non-null `delegate_id` or `assigneeId`. Assertion messages include actual counts for debugging.