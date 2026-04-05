Implement subtask 10004: Enable Kubernetes API audit logging

## Objective
Configure the Kubernetes API server audit policy to log all authentication, authorization, and resource mutation events, and ship audit logs to the existing log aggregation stack (Loki).

## Steps
1. Create an audit policy file (`audit-policy.yaml`) with rules:
   - Log all `create`, `update`, `patch`, `delete` at `RequestResponse` level.
   - Log all `authentication` events at `Metadata` level.
   - Log `get`, `list`, `watch` on Secrets at `Metadata` level.
   - Omit high-volume, low-value events (e.g., health checks, leader election).
2. Configure the API server to use the audit policy with a log backend (file or webhook).
3. If using a managed Kubernetes provider, enable the provider's audit log feature and configure export.
4. Deploy a log shipping agent (e.g., Promtail sidecar or Fluentd) to forward audit logs to Loki.
5. Verify audit logs appear in Loki with appropriate labels (source=k8s-audit).

## Validation
Create a test Secret in a test namespace and verify the creation event appears in Loki audit logs within 60 seconds. Verify failed authentication attempts are logged. Query Loki for `{source="k8s-audit"}` and confirm structured log entries with user, verb, resource, and timestamp fields.