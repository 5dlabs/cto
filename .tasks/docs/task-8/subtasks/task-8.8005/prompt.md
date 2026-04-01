Implement subtask 8005: Implement Test Case 2: Task Generation with Agent Assignments

## Objective
Write the E2E test that fetches generated tasks for the pipeline run and asserts correct task count, agent assignment, and agent diversity.

## Steps
1. Test Case 2 (`it('generates >= 5 tasks with diverse agent assignments')`):
   a. Call `GET ${PM_SERVER_URL}/api/pipeline/${runId}/tasks`.
   b. Parse response as JSON array of task objects.
   c. Assert: `tasks.length >= 5`.
   d. Assert: every task has a non-empty string `agent` field (`task.agent` is truthy and `typeof task.agent === 'string'`).
   e. Collect unique agent values: `new Set(tasks.map(t => t.agent))`.
   f. Assert: `uniqueAgents.size >= 3`.
   g. Log the task count and agent distribution for debugging (e.g., `{ bolt: 2, rex: 1, nova: 3, ... }`).
2. If the endpoint returns an error, fail with the full response body for debugging.

## Validation
Test passes when at least 5 tasks are returned, each has a non-empty agent field, and at least 3 distinct agent types are represented. Console output shows agent distribution for manual verification.