Implement subtask 1001: Create Kubernetes namespaces and RBAC foundations

## Objective
Create all required Kubernetes namespaces (databases, sigma1, openclaw, social, web) and apply baseline RBAC roles and service accounts for each namespace to enable subsequent deployments.

## Steps
1. Define namespace manifests for: databases, sigma1, openclaw, social, web.
2. Apply each namespace via kubectl/Helm.
3. Create a ServiceAccount in each namespace for workloads to use.
4. Apply baseline RBAC RoleBindings granting the service accounts read access to ConfigMaps and Secrets within their namespace.
5. Label all namespaces with project=sigma1 for easy identification.

## Validation
Run `kubectl get namespaces` and verify all five namespaces exist with correct labels. Verify ServiceAccounts exist in each namespace. Verify RoleBindings are applied and functional by attempting a permitted and a denied operation from a test pod.