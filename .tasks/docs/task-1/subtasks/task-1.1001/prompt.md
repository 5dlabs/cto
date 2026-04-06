Implement subtask 1001: Create Kubernetes namespaces and RBAC foundations

## Objective
Create all required Kubernetes namespaces (databases, sigma1, openclaw, etc.) and set up basic RBAC ServiceAccounts for each namespace so downstream deployments have appropriate permissions.

## Steps
1. Create namespace manifests for: databases, sigma1, openclaw (and any others referenced in the PRD).
2. Apply namespace labels for organization (e.g., app.kubernetes.io/part-of: sigma1).
3. Create ServiceAccount resources in each namespace for pod identity.
4. Apply ResourceQuota and LimitRange on the sigma1 namespace to prevent runaway resource usage in dev.
5. Use `kubectl apply -f` or Helm to deploy all namespace manifests.
6. Verify namespaces exist with `kubectl get ns`.

## Validation
Run `kubectl get ns` and confirm all expected namespaces exist; verify ServiceAccounts are present in each namespace with `kubectl get sa -n <ns>`; confirm ResourceQuota is applied.