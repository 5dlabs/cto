Implement subtask 2003: Modify Linear issue creation flow to use delegate_id with agent:pending fallback

## Objective
Modify the existing Linear issue creation flow in cto-pm to call resolve_agent_delegates() before creating each issue. If a valid user ID is returned, set assigneeId on the createIssue mutation. If resolution fails or returns null, apply the 'agent:pending' label and do NOT throw.

## Steps
1. Locate the Linear issue creation flow in cto-pm (the function/handler that calls the Linear GraphQL API createIssue mutation).
2. Before creating each issue, call resolve_agent_delegates() with the task's agent hint.
3. If a valid Linear user ID is returned:
   - Set `assigneeId` on the createIssue mutation payload.
   - Set the task's `delegate_id` to the resolved user ID.
   - Set `delegation_status` to 'assigned'.
4. If resolution returns null or throws:
   - Do NOT propagate the error — catch it gracefully.
   - Apply the label 'agent:pending' to the issue (via the labels field on createIssue or a subsequent addLabel call).
   - Set `delegate_id` to null.
   - Set `delegation_status` to 'pending' (or 'failed' if an error occurred vs. simply unknown agent).
5. Add structured logging: log agent hint, resolved user ID (or null), and the fallback action taken (label applied vs. assigned).
6. Ensure the Linear API token is sourced from the environment variable injected via sigma-1-infra-endpoints ConfigMap envFrom.
7. Verify no existing tests break — run the full test suite after changes.

## Validation
Unit test: mock resolve_agent_delegates() to return a valid user ID and verify createIssue is called with assigneeId set and delegation_status is 'assigned'. Unit test: mock resolve_agent_delegates() to return null and verify createIssue is called with 'agent:pending' label and no assigneeId, delegation_status is 'pending'. Unit test: mock resolve_agent_delegates() to throw and verify the error is caught, 'agent:pending' label is applied, delegation_status is 'failed', and no exception propagates. Backward compatibility test: call with agent hint 'unknown-agent' and confirm no throw, delegation_status is 'pending'.