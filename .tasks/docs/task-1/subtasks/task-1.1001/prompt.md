Implement subtask 1001: Create Kubernetes namespaces and RBAC foundation

## Objective
Create all required Kubernetes namespaces (databases, sigma1, openclaw, social, web) and configure basic RBAC service accounts so that downstream resources can be deployed into the correct namespace with appropriate permissions.

## Steps
1. Create namespace manifests for: databases, sigma1, openclaw, social, web.
2. Apply each namespace via kubectl or Helm.
3. Create a ServiceAccount in each namespace for workloads to use (e.g., sigma1-sa, openclaw-sa).
4. Create RoleBindings granting each ServiceAccount read access to secrets and configmaps within its own namespace.
5. Create a ClusterRole that allows the sigma1 namespace SA to read the sigma1-infra-endpoints ConfigMap from the databases namespace (cross-namespace read).
6. Label all namespaces with project=sigma1 for easy identification.

## Validation
Run `kubectl get namespaces -l project=sigma1` and verify all 5 namespaces exist. Verify each ServiceAccount exists via `kubectl get sa -n <namespace>`. Verify RoleBindings are active and the cross-namespace read works by running `kubectl auth can-i get configmaps --as=system:serviceaccount:sigma1:sigma1-sa -n databases`.