Implement subtask 10001: Define Kubernetes RBAC Roles and RoleBindings for all service accounts

## Objective
Create least-privilege RBAC Roles, ClusterRoles, RoleBindings, and ClusterRoleBindings for every service account in the cluster. Each service should only have access to the namespaced resources it needs.

## Steps
1. Inventory all existing ServiceAccounts across all namespaces (application services, operators, monitoring agents).
2. For each ServiceAccount, define a Role (namespaced) or ClusterRole (cluster-wide) with the minimum set of API groups, resources, and verbs required.
3. Create corresponding RoleBinding or ClusterRoleBinding manifests binding each ServiceAccount to its Role/ClusterRole.
4. Remove any default or overly-permissive bindings (e.g., default ServiceAccount auto-mount, broad cluster-admin bindings).
5. Set `automountServiceAccountToken: false` on pods that don't need API access.
6. Organize all RBAC manifests under `infra/rbac/` with one file per namespace or logical grouping.
7. Apply via Helm chart or Kustomize overlay so RBAC is version-controlled and reproducible.

## Validation
For each ServiceAccount: (a) verify it can perform its required operations (e.g., `kubectl auth can-i --as=system:serviceaccount:<ns>:<sa> <verb> <resource>`), (b) verify it CANNOT perform operations outside its scope (e.g., cannot list secrets in other namespaces, cannot delete deployments). Run `kubectl auth reconcile --dry-run` to confirm no drift from manifests.