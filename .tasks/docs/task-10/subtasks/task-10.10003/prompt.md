Implement subtask 10003: Enable Kubernetes API server audit logging with policy configuration

## Objective
Configure the Kubernetes API server audit logging with an appropriate audit policy that captures security-relevant events (authentication, authorization, secret access, RBAC changes) while filtering noise.

## Steps
1. Create an audit policy YAML (`audit-policy.yaml`) with tiered rules:
   - `RequestResponse` level for: secrets access, RBAC resource changes (roles, rolebindings), serviceaccount token creation, pod exec/attach.
   - `Request` level for: deployment/statefulset create/update/delete, configmap changes.
   - `Metadata` level for: all other resource mutations.
   - `None` level for: health checks, node status updates, and other high-frequency read-only noise.
2. Configure the API server flags: `--audit-policy-file`, `--audit-log-path`, `--audit-log-maxage=30`, `--audit-log-maxbackup=10`, `--audit-log-maxsize=100`.
3. If using a managed Kubernetes service (EKS/GKE/AKS), enable the cloud-native audit log integration instead of file-based logging.
4. Store the audit policy manifest under `infra/audit/`.

## Validation
Perform security-relevant actions (create a secret, modify an RBAC binding, exec into a pod) and verify corresponding audit log entries are generated with correct level, user, resource, and verb fields. Verify that filtered events (health checks) do NOT appear in audit logs.