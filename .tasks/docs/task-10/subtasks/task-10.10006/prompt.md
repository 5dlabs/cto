Implement subtask 10006: Configure Kubernetes audit policy for sigma1-prod namespace

## Objective
Create and apply a Kubernetes audit policy that logs all mutating operations in sigma1-prod and authentication failures at RequestResponse level, routing logs to persistent storage.

## Steps
1. Create `infra/audit/audit-policy.yaml` with the following rules:
   - Rule 1: level `RequestResponse` for all create/update/patch/delete verbs in namespace `sigma1-prod`.
   - Rule 2: level `RequestResponse` for authentication failures (group `authentication.k8s.io`).
   - Rule 3: level `Metadata` as catch-all for remaining requests in `sigma1-prod`.
   - Rule 4: level `None` for read-only requests to health endpoints to reduce noise.
2. Configure the API server to use this policy (if managed cluster, document required cluster-level config; if self-managed, add `--audit-policy-file` and `--audit-log-path` flags).
3. Create a PersistentVolumeClaim `audit-log-pvc` (5Gi) for storing audit logs if no external aggregator is configured.
4. Alternatively, configure a log forwarding sidecar or DaemonSet to ship audit logs to the chosen aggregator.
5. Apply all manifests and verify audit logging is active.

## Validation
Perform a mutating action in sigma1-prod (e.g., `kubectl create configmap test-audit --from-literal=key=value -n sigma1-prod`). Within 60 seconds, check the audit log output (file or aggregator) for a corresponding entry containing the create verb, the configmap resource, and the sigma1-prod namespace. Clean up the test configmap.