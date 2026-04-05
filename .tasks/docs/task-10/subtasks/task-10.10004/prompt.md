Implement subtask 10004: Enable Kubernetes API audit logging

## Objective
Configure the Kubernetes API server audit policy and audit log backend to capture all security-relevant API operations.

## Steps
1. Create an audit policy YAML file defining rules at appropriate levels: a) `RequestResponse` level for write operations on Secrets, RBAC resources, and ServiceAccounts. b) `Request` level for all write operations on core resources (pods, deployments, etc.). c) `Metadata` level for read operations on sensitive resources. d) `None` level for health checks and other noise (e.g., `/healthz`, `/readyz`, system:nodes watch). 2. Configure the API server to use the audit policy: a) For managed Kubernetes (EKS/GKE/AKS), enable audit logging via the cloud provider's cluster configuration. b) For self-managed clusters, add `--audit-policy-file` and `--audit-log-path` flags to the kube-apiserver manifest. 3. Configure audit log shipping to the chosen storage backend (local file with rotation, or webhook to logging stack). 4. If using a centralized logging stack, deploy a log shipper (e.g., Fluent Bit DaemonSet) configured to tail audit log files and forward to Loki/Elasticsearch. 5. Verify audit logs are being generated and captured for test operations.

## Validation
Perform a sensitive operation (e.g., `kubectl create secret generic test-audit --from-literal=key=value`) and verify the operation appears in the audit log within 60 seconds. Verify audit log entries contain the expected fields: user, verb, resource, timestamp, response code. Confirm that noisy endpoints (`/healthz`) are excluded from logs. If using centralized logging, query the logging backend and confirm audit events are searchable.