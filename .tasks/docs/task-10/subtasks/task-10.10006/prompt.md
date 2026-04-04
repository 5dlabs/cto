Implement subtask 10006: Implement application-level audit logging for cto-pm pipeline operations

## Objective
Add structured audit log entries to the cto-pm application for all key pipeline events: delegation resolution, Linear issue creation, notification dispatch, and pipeline stage transitions. Each log entry must include a timestamp, event type, and relevant identifiers.

## Steps
Step-by-step:
1. Define an audit log schema/format (structured JSON recommended):
   - `timestamp`: ISO 8601
   - `event_type`: one of `delegation_resolved`, `issue_created`, `notification_sent`, `pipeline_stage_transition`
   - `details`: event-specific fields
2. For `delegation_resolved`: log agent_hint, resolved_user_id, resolution_method.
3. For `issue_created`: log linear_issue_id, assignee_id, team_id, title_hash.
4. For `notification_sent`: log bridge_type (discord/other), payload_hash (SHA-256 of payload), response_status_code.
5. For `pipeline_stage_transition`: log from_stage, to_stage, pipeline_run_id, duration_ms.
6. Emit these logs to stdout in JSON format so they are captured by the cluster's log collector.
7. Ensure logs do NOT contain secrets, tokens, or PII — use hashes or IDs only.
8. If the cto-pm codebase uses a logging library, add an `audit` log level or a dedicated audit logger.

## Validation
Trigger a full pipeline run and inspect `kubectl logs` for the cto-pm pod. Verify at least one log entry for each of the four event types. Each entry must be valid JSON with `timestamp`, `event_type`, and `details` fields. Confirm no secrets or tokens appear in any log entry (grep for known token prefixes). Verify timestamps are in ISO 8601 format.