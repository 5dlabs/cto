Implement subtask 10002: Create ClusterRole for Hermes operator (human admin) access

## Objective
Create a ClusterRole and ClusterRoleBinding for human Hermes operators granting full access to the hermes-production namespace resources.

## Steps
1. Create `ClusterRole` `hermes-operator` with rules granting full access (`*` verbs) to all resources in the `hermes-production` namespace, scoped via a `ClusterRoleBinding` with namespace restriction or via a namespaced `Role` + `RoleBinding` if full cluster-level access is not needed.
2. Alternatively, create a namespaced `Role` `hermes-operator` in `hermes-production` with `*` verbs on `*` resources, and bind it to the operator group/users.
3. Document which users/groups should be bound to this role.
4. Do NOT grant cluster-admin or access to other namespaces.

## Validation
Verify an operator-bound user can `kubectl get all -n hermes-production` and `kubectl delete pod -n hermes-production <pod>` but cannot access resources in `kube-system` or other namespaces via the Hermes operator role.