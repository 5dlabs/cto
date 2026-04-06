Implement subtask 10004: Enable Kubernetes API server audit logging

## Objective
Configure the Kubernetes API server audit policy to log all administrative and sensitive actions, and ensure audit logs are written to a persistent location for collection.

## Steps
1. Create an audit-policy.yaml file defining audit levels: a) `RequestResponse` for secrets, RBAC resources, pod exec/attach. b) `Request` for all write operations (create, update, delete, patch). c) `Metadata` for read operations on sensitive resources. d) `None` for health checks and high-volume read-only endpoints (to reduce noise). 2. If using a managed Kubernetes provider, enable audit logging through the provider's console/API (e.g., GKE audit logs, EKS CloudTrail). 3. If self-managed, configure kube-apiserver flags: `--audit-policy-file`, `--audit-log-path`, `--audit-log-maxage=30`, `--audit-log-maxbackup=10`, `--audit-log-maxsize=100`. 4. Verify audit logs are being written by performing a test action (create a ConfigMap) and checking the audit log output.

## Validation
Perform a series of test actions: create a ConfigMap, read a Secret, exec into a pod. Verify each action appears in the audit log with the correct audit level, user identity, resource, and verb. Confirm that health check endpoints are NOT logged (noise reduction). Verify logs are persisted and not lost on pod restart.