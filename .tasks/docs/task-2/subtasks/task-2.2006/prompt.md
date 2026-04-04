Implement subtask 2006: Add structured logging for delegation resolution across the flow

## Objective
Ensure all delegation-related operations (resolution, issue creation, fallback) produce structured log entries with agent hint, resolved user ID, and action taken.

## Steps
1. Review all code paths touched in subtasks 2001 and 2003 for logging completeness.
2. Ensure structured JSON logging (not console.log with string interpolation) is used consistently.
3. Each delegation resolution log entry should include: `{ event: 'delegation_resolution', agent_hint: string, resolved_user_id: string | null, action: 'assigned' | 'label_fallback' | 'error_fallback', timestamp: ISO string }`.
4. Each issue creation log entry should include: `{ event: 'issue_created', linear_issue_id: string, assignee_id: string | null, labels: string[], timestamp: ISO string }`.
5. Ensure error cases log the error message and stack without exposing sensitive data (no Linear API tokens in logs).
6. Verify log output format is consistent and parseable by standard log aggregation tools.

## Validation
Unit test: trigger a successful delegation and capture log output; verify it contains a JSON object with event='delegation_resolution', agent_hint, and resolved_user_id. Trigger a fallback delegation and verify action='label_fallback' appears in log. Verify no Linear API tokens appear in any log output by searching captured logs for known token patterns.