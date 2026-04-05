Implement subtask 1001: Create Kubernetes namespaces for all Sigma-1 service domains

## Objective
Create all required Kubernetes namespaces (databases, sigma1, openclaw, social, web, and any others referenced by the architecture) with appropriate labels and annotations for service discovery and RBAC scoping.

## Steps
1. Define a YAML manifest (namespaces.yaml) declaring each namespace: databases, sigma1, openclaw, social, web.
2. Add labels such as `app.kubernetes.io/part-of: sigma1` and `team: <owner>` for each namespace.
3. Apply the manifest via `kubectl apply -f namespaces.yaml`.
4. Verify all namespaces exist with `kubectl get namespaces` and confirm labels are correct.
5. Ensure no naming conflicts with existing cluster namespaces.

## Validation
Run `kubectl get namespaces` and confirm all expected namespaces (databases, sigma1, openclaw, social, web) exist with correct labels and annotations.