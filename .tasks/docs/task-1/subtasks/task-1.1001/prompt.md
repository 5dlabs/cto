Implement subtask 1001: Create Kubernetes namespaces and RBAC foundation

## Objective
Create all required Kubernetes namespaces for the Sigma-1 platform (databases, sigma1, openclaw, social, web) and configure baseline RBAC ServiceAccounts and RoleBindings so that each namespace's workloads can access their designated secrets and ConfigMaps.

## Steps
1. Author a Helm chart or kustomize overlay that declaratively creates namespaces: databases, sigma1, openclaw, social, web.
2. In each namespace create a default ServiceAccount (e.g., sigma1-sa, databases-sa) with labels for future RBAC scoping.
3. Create RoleBindings that grant each ServiceAccount read access to ConfigMaps and Secrets within its own namespace.
4. Create a ClusterRole 'sigma1-infra-reader' that allows reading the 'sigma1-infra-endpoints' ConfigMap from the sigma1 namespace, and bind it to ServiceAccounts in other namespaces that need cross-namespace access.
5. Apply resource quotas on each namespace to prevent runaway resource usage during dev (generous limits, not restrictive).
6. Commit all manifests under infra/namespaces/.

## Validation
Run 'kubectl get ns' and verify all five namespaces exist; run 'kubectl auth can-i get configmaps --as=system:serviceaccount:sigma1:sigma1-sa -n sigma1' and confirm allowed; verify ServiceAccounts exist in each namespace.