Implement subtask 10004: Enable Kubernetes API server audit logging with retention policy

## Objective
Configure the Kubernetes API server audit policy to capture security-relevant events (authentication, authorization, secret access, RBAC changes) and ship logs to the chosen sink with defined retention.

## Steps
1. Create a Kubernetes audit policy file (`audit-policy.yaml`) with rules:
   - Log all `authentication` and `authorization` events at `Metadata` level.
   - Log `Secret`, `ConfigMap`, `Role`, `ClusterRole`, `RoleBinding`, `ClusterRoleBinding` operations at `RequestResponse` level.
   - Log pod `exec`, `attach`, `portforward` at `RequestResponse` level.
   - Log all `delete` operations at `Metadata` level.
   - Set a catch-all rule at `Metadata` level for everything else.
2. Configure the API server to use the audit policy with the chosen backend (log file, webhook, or dynamic — per dp-16 decision).
3. If using log files: configure log rotation (max size, max age, max backups).
4. If using a webhook backend: deploy the log collector (Fluentd/Fluent Bit/Vector) and configure it to forward to the chosen sink.
5. Set retention policy (e.g., 90 days hot, 1 year cold storage).
6. Apply the configuration and verify audit logs are being generated.

## Validation
Perform several auditable actions (create a Secret, delete a pod, exec into a pod, modify an RBAC role). Verify each action appears in the audit logs with correct metadata (user, verb, resource, timestamp). Verify log rotation is working by checking log file sizes. Confirm logs are forwarded to the configured sink.