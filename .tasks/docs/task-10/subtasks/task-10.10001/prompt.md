Implement subtask 10001: Define and apply Kubernetes RBAC policies for all namespaces and service accounts

## Objective
Create Role, ClusterRole, RoleBinding, and ClusterRoleBinding resources to enforce least-privilege access for all service accounts, operators, and human administrators across all namespaces.

## Steps
1. Inventory all service accounts across namespaces (application services, operators, cloudflared, monitoring agents).
2. Create namespace-scoped Roles for each service account granting only the permissions they need (e.g., app services: get/list configmaps, secrets in their namespace; operators: manage their CRDs).
3. Create RoleBindings binding each service account to its Role.
4. Create ClusterRoles for cluster-wide needs (e.g., monitoring agent needs read access to node metrics).
5. Create a read-only ClusterRole for developer access (no secret read, no exec).
6. Create an admin ClusterRole for ops with broader but still scoped permissions.
7. Apply a default deny posture: ensure no service account uses the `cluster-admin` ClusterRole.
8. Remove any auto-mounted default service account tokens where not needed (`automountServiceAccountToken: false`).
9. Apply all RBAC manifests and verify with `kubectl auth can-i --list --as=system:serviceaccount:<ns>:<sa>`.

## Validation
For each service account, run `kubectl auth can-i` to confirm it can only perform its intended operations; verify a test service account cannot read secrets in other namespaces; verify developer role cannot exec into pods or read secrets; confirm no service account has cluster-admin binding.