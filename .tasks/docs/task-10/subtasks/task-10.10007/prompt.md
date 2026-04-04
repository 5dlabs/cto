Implement subtask 10007: Add audit logging for namespace security events

## Objective
Enable audit logging for the sigma-1-dev namespace to capture secret access events and security-relevant API calls, either via cluster audit policy or a structured logging sidecar.

## Steps
1. Determine if cluster-level audit policy can be modified (requires cluster-admin access to the API server configuration).
2. If cluster audit policy is accessible:
   a. Add a namespace-scoped audit rule to the cluster audit policy YAML:
      ```yaml
      - level: RequestResponse
        resources:
        - group: ""
          resources: ["secrets"]
        namespaces: ["sigma-1-dev"]
      ```
   b. Restart the API server or wait for the policy to be picked up.
3. If cluster audit policy is NOT accessible:
   a. Deploy a structured logging sidecar (e.g., Falco or a custom audit logger) as a DaemonSet or sidecar in the namespace.
   b. Configure it to watch for secret access, RBAC denials, and pod exec events.
   c. Example: Use `kube-audit-rest` as a webhook backend to capture audit events for the namespace.
4. Ensure audit logs are shipped to the cluster's logging backend (e.g., stdout for collection by Fluentd/Promtail, or directly to a log aggregator).
5. Verify audit events appear for secret read operations in sigma-1-dev.

## Validation
Perform a secret access in the namespace: `kubectl get secret sigma-1-secrets -n sigma-1-dev`. Then check the audit log output (API server audit log file, or sidecar logs via `kubectl logs`) for a corresponding entry showing the secret access event with request metadata. Assert at least one audit log entry is captured for the operation.