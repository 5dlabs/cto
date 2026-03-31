Implement subtask 10001: Create Kubernetes RBAC: ServiceAccounts, Roles, and RoleBindings for Hermes services

## Objective
Create dedicated ServiceAccounts for backend and frontend services, create Roles with least-privilege access, bind them with RoleBindings, and disable default ServiceAccount token automount on all pods.

## Steps
1. Create `ServiceAccount` `hermes-backend-sa` in `hermes-production` namespace.
2. Create `ServiceAccount` `hermes-frontend-sa` in `hermes-production` namespace.
3. Create `Role` `hermes-backend-role` with rules: get/list ConfigMaps, get Secrets matching `hermes-*` resource names only. No cluster-level access.
4. Create `Role` `hermes-frontend-role` with rules: get/list ConfigMaps only. No Secret access.
5. Create `RoleBinding` binding `hermes-backend-role` to `hermes-backend-sa`.
6. Create `RoleBinding` binding `hermes-frontend-role` to `hermes-frontend-sa`.
7. Update backend Deployment to use `serviceAccountName: hermes-backend-sa` and set `automountServiceAccountToken: false` on the pod spec (mount only if needed via projected volume).
8. Update frontend Deployment similarly with `hermes-frontend-sa`.
9. Verify no pod uses the `default` ServiceAccount.

## Validation
Verify `kubectl auth can-i get configmaps --as=system:serviceaccount:hermes-production:hermes-backend-sa -n hermes-production` returns `yes`. Verify `kubectl auth can-i list secrets --as=system:serviceaccount:hermes-production:hermes-backend-sa -n kube-system` returns `no`. Verify `kubectl auth can-i get secrets --as=system:serviceaccount:hermes-production:hermes-frontend-sa -n hermes-production` returns `no`.