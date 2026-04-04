Implement subtask 10008: Enable Kubernetes audit logging for sigma-1-dev namespace

## Objective
Configure Kubernetes audit logging to capture all create, update, and delete operations on secrets, configmaps, and deployments within the sigma-1-dev namespace.

## Steps
1. Create or update the Kubernetes audit policy file (typically at `/etc/kubernetes/audit-policy.yaml` on the control plane, or via the cluster provider's audit configuration).
2. Add an audit rule:
   - level: RequestResponse
   - resources: [{group: "", resources: ["secrets", "configmaps"]}, {group: "apps", resources: ["deployments"]}]
   - namespaces: ["sigma-1-dev"]
   - verbs: ["create", "update", "patch", "delete"]
3. If the cluster is managed (EKS, GKE, AKS), use the provider's audit log configuration (CloudWatch, Cloud Logging, Azure Monitor) and document the configuration steps.
4. If using a self-managed cluster, ensure the API server is configured with `--audit-policy-file` and `--audit-log-path` flags.
5. Document the audit log location and how to query it in `docs/production/audit-logging.md`.
6. Verify that a test operation (e.g., `kubectl annotate configmap sigma-1-infra-endpoints test=true -n sigma-1-dev && kubectl annotate configmap sigma-1-infra-endpoints test- -n sigma-1-dev`) appears in the audit log.

## Validation
Perform a test annotation on a configmap in sigma-1-dev, then query the audit log (via kubectl logs, CloudWatch, or the configured sink) and confirm the create/update event is recorded with the correct namespace, resource, and verb.