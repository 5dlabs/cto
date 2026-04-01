Implement subtask 2003: Integrate resolve_agent_delegates into issueCreate mutation flow

## Objective
Modify the existing task-to-issue mapping module to call resolve_agent_delegates() before each issueCreate GraphQL mutation and pass the resolved linearUserId as the assigneeId field in the mutation payload.

## Steps
1. Locate the existing issue creation call site (the `issueCreate` GraphQL mutation in the PM server).
2. Before the loop/call that creates issues, invoke `resolve_agent_delegates()` with the collected array of agent hint strings from the decomposed tasks.
3. For each task/issue being created, look up the task's agent hint in the returned map.
4. If a userId is found, include `assigneeId: userId` in the `issueCreate` mutation variables.
5. If no userId is found (hint was unresolvable), omit `assigneeId` so the issue is created unassigned.
6. Ensure the mutation payload type/interface is updated to accept the optional `assigneeId` field.

## Validation
Integration test: with a mocked Linear API, submit a pipeline run with 3 tasks having hints 'nova', 'bolt', 'unknown'. Verify the issueCreate calls for 'nova' and 'bolt' include correct assigneeId values, and the 'unknown' task's call omits assigneeId.