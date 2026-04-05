Implement subtask 10001: Define Kubernetes RBAC roles and bindings for all service accounts

## Objective
Create least-privilege Role, ClusterRole, RoleBinding, and ClusterRoleBinding resources for every service account across all namespaces. Ensure no service account has more permissions than required for its function.

## Steps
1. Inventory all ServiceAccounts across all namespaces (application services, operators, cloudflared, monitoring).
2. For each ServiceAccount, document the minimum required permissions (e.g., which API resources, verbs, and namespaces it needs).
3. Create namespace-scoped Roles for services that only need access within their namespace (e.g., app services reading ConfigMaps and Secrets in their namespace).
4. Create ClusterRoles only for cross-namespace or cluster-wide needs (e.g., operators managing CRDs).
5. Create corresponding RoleBindings and ClusterRoleBindings linking each ServiceAccount to the appropriate Role/ClusterRole.
6. Remove or restrict the `default` ServiceAccount in each namespace (set `automountServiceAccountToken: false`).
7. Ensure no ServiceAccount has `cluster-admin` or wildcard `*` permissions.
8. Apply all RBAC manifests and organize them in the infra repo under `rbac/` directory.

## Validation
For each ServiceAccount, use `kubectl auth can-i --list --as=system:serviceaccount:<ns>:<sa>` to verify it has only the expected permissions. Attempt unauthorized operations (e.g., a web service SA trying to delete pods) and confirm they are denied with 403. Verify the default SA in each namespace cannot access any resources.