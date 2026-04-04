Implement subtask 2004: Implement fallback behavior for unresolvable agents and summary logging

## Objective
Add fallback logic for when resolve_agent_delegates() returns null for an agent hint: log a warning, create the issue unassigned, add an agent:unresolved label, and emit a summary log line after all issues are created.

## Steps
1. In the pipeline integration code (from subtask 2003), after populating delegate_id, check for tasks where `delegate_id === null`.
2. For each such task, log a structured warning: `{ level: 'warn', message: 'Unresolved agent delegate', agentHint: task.agent, taskId: task.id }`.
3. When creating the Linear issue for an unresolved task:
   a. Omit `assigneeId` (or pass undefined).
   b. Add a label `agent:unresolved` to the issue. Use the Linear API to find or create this label, then attach it.
4. After all issues are created, compute and log a summary line:
   - `Created N issues, M assigned, K unresolved`
   - Where N = total issues, M = issues with non-null delegate_id, K = issues with null delegate_id.
5. Use structured logging (JSON) consistent with the PM server's existing log format.
6. Ensure the `agent:unresolved` label creation is idempotent (check if it exists before creating).

## Validation
Unit test: Given a task with an unknown agent hint, the warning log is emitted with the correct agent hint and task ID. Integration test: Run pipeline with one known and one unknown agent; verify the unknown agent's issue is created without assigneeId and has the `agent:unresolved` label. Verify the summary log line shows correct counts (e.g., 'Created 2 issues, 1 assigned, 1 unresolved').