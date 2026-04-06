Implement subtask 10001: Define and apply Kubernetes RBAC Roles and RoleBindings for all service accounts

## Objective
Create least-privilege RBAC Roles (or ClusterRoles where necessary) and bind them to dedicated ServiceAccounts for each service, ensuring no service has more permissions than it needs.

## Steps
1) Inventory all services and their Kubernetes API needs (e.g., most application pods need zero Kubernetes API access; operators may need specific resource access). 2) Create a dedicated ServiceAccount for each service if not already present. 3) For services that need no Kubernetes API access, ensure they use a ServiceAccount with `automountServiceAccountToken: false`. 4) For services that need Kubernetes API access (e.g., cloudflared reading ConfigMaps, any operator), create a Role scoped to the exact resources and verbs needed (e.g., `get`, `list` on `configmaps` in their namespace). 5) Create RoleBindings linking each Role to its ServiceAccount. 6) If any cross-namespace access is needed, use ClusterRole + ClusterRoleBinding but scope as tightly as possible. 7) Remove any default or overly broad bindings (e.g., `default` ServiceAccount should have no extra permissions). 8) Apply all RBAC manifests and store them in version control under `k8s/rbac/`.

## Validation
For each service, exec into a pod and attempt `kubectl auth can-i --list` (using the mounted service account token) to verify the exact permissions granted. Verify that application pods with `automountServiceAccountToken: false` cannot reach the Kubernetes API at all. Attempt unauthorized actions (e.g., a backend pod trying to list secrets or delete pods) and confirm they are denied with 403. Run `kubectl auth can-i` checks from a CI script covering all expected allow/deny combinations.