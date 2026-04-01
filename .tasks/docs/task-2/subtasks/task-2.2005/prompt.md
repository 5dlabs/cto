Implement subtask 2005: Add structured logging for agent delegation in issue creation

## Objective
Add structured log entries at each issue creation that include the issue ID, issue title, agent hint, and resolved delegate_id (or 'unassigned').

## Steps
1. After each successful `issueCreate` call, emit a structured log object: `{ event: 'issue_created', issueId, title, agentHint, delegateId: resolvedId ?? 'unassigned' }`.
2. Use the project's existing structured logger (e.g., pino, consola, or Bun's console with JSON formatting).
3. At the start of agent resolution, log the full list of agent hints being resolved.
4. At the end of the pipeline run's issue creation phase, log a summary: total issues created, count assigned, count unassigned.
5. Ensure log level is 'info' for normal operations and 'warn' for unresolvable hints.

## Validation
Capture log output during a test pipeline run. Verify each created issue produces a structured log entry with all required fields. Verify the summary log shows correct counts.