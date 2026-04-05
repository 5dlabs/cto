Implement subtask 10001: Define Kubernetes RBAC roles and role bindings for all namespaces and services

## Objective
Create fine-grained RBAC Role, ClusterRole, RoleBinding, and ClusterRoleBinding resources for all namespaces, ensuring each service's ServiceAccount has least-privilege access to only the Kubernetes resources it needs.

## Steps
1. Inventory all ServiceAccounts across all namespaces (one per service).
2. For each service, identify what Kubernetes API access it needs (e.g., read ConfigMaps, read Secrets in own namespace only).
3. Create namespace-scoped Roles with minimal permissions (e.g., `get`, `list` on specific resource types).
4. Create RoleBindings linking each ServiceAccount to its Role.
5. Create ClusterRoles only for cross-namespace needs (e.g., cloudflared reading tunnel secrets).
6. Remove any overly permissive default bindings (e.g., `default` SA auto-mounted tokens).
7. Set `automountServiceAccountToken: false` on all pods that don't need Kubernetes API access.
8. Store all RBAC manifests in `rbac/` directory in the infra repo, organized by namespace.

## Validation
For each service, exec into a pod and attempt to list resources outside its granted scope (e.g., `kubectl auth can-i list pods --namespace=other-ns`) — verify denied. Verify services can still access their required ConfigMaps and Secrets. Run `kubectl auth can-i --list` for each ServiceAccount and confirm minimal permissions.