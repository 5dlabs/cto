Implement subtask 8006: Implement AC-2 and AC-3 tests: Linear session with delegated issues

## Objective
Write E2E tests verifying that after pipeline completion, the delegation status endpoint returns >= 5 tasks and >= 5 tasks have non-null delegate_id with 'assigned' status. Optionally verify against live Linear API.

## Steps
Step-by-step:
1. In `tests/e2e/pipeline.test.ts`, create test: `test('AC-2: Delegation status returns >= 5 tasks', async () => { ... })`.
2. Using the run ID from AC-1 test, call `GET /api/delegation/status?runId={runId}`.
3. Parse response and assert: `response.tasks.length >= 5`.
4. Create test: `test('AC-3: At least 5 tasks have delegate_id assigned', async () => { ... })`.
5. Filter tasks where `delegation_status === 'assigned'` AND `delegate_id !== null && delegate_id !== undefined`.
6. Assert: filtered count >= 5.
7. Assert: `delegate_id` values match expected Linear user ID format (string, non-empty).
8. Assert: no tasks that should be assigned are stuck in 'pending' status.
9. If using LiveLinearAdapter: for each assigned task, call `linearAdapter.verifyAssignee(issueId)` to confirm the Linear issue exists and has an assignee set.
10. Collect issue IDs for cleanup phase.

## Validation
AC-2 passes when delegation status response contains >= 5 tasks. AC-3 passes when >= 5 tasks have non-null delegate_id and status 'assigned'. If live Linear mode: each issue's assignee is confirmed via Linear API. No tasks expected to be assigned are stuck in 'pending'.