Implement subtask 10007: Implement application-level audit logging middleware with Loki integration

## Objective
Create audit logging middleware in the Hermes backend that logs all critical resource access with structured fields (audit_action, audit_actor, audit_resource, audit_result, audit_ip) and ships logs to Loki with a dedicated 'audit=true' label.

## Steps
1. Create `src/modules/hermes/audit-middleware.ts` that extends the existing auth middleware from Task 2.
2. Define the audit log structure as a TypeScript interface:
   ```typescript
   interface AuditLogEntry {
     audit_action: 'create_deliberation' | 'read_artifact' | 'trigger_migration' | 'access_admin_endpoint' | 'delete_deliberation';
     audit_actor: string; // user ID from session
     audit_resource: string; // resource type and ID, e.g., 'deliberation:abc-123'
     audit_result: 'success' | 'denied' | 'error';
     audit_ip: string; // client IP from request headers
     timestamp: string; // ISO 8601
   }
   ```
3. Implement the middleware to:
   a. Extract user ID from session.
   b. Determine audit_action from the route and method.
   c. Log the entry as structured JSON to stdout.
   d. Add `audit: 'true'` as a log label for Loki filtering.
4. Wire the middleware into all Hermes routes: POST /deliberations, GET /deliberations/:id, POST /admin/migrate-artifacts, etc.
5. Log `denied` result when RBAC check fails (before returning 403).
6. Ensure audit logs are separate from application logs by using a dedicated logger instance with the `audit=true` label.
7. Configure Loki (via Promtail/Grafana Agent) to parse the `audit` label and apply a longer retention policy for audit logs.

## Validation
Create a deliberation via the API. Within 30 seconds, query Loki: `{app="hermes", audit="true"} | json | audit_action="create_deliberation"` and verify the log entry contains correct `audit_actor` and `audit_resource`. Attempt an unauthorized action and verify an audit log with `audit_result="denied"` appears. Verify audit logs have the `audit=true` label in Loki.