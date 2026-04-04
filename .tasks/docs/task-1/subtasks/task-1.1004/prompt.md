Implement subtask 1004: Create ServiceAccount sigma-1-pm-server with RBAC Role and RoleBinding

## Objective
Create a ServiceAccount, Role, and RoleBinding in sigma-1-dev namespace granting minimal get/list permissions on configmaps and secrets.

## Steps
1. Create a ServiceAccount manifest for `sigma-1-pm-server` in `sigma-1-dev` namespace.
2. Create a Role `sigma-1-pm-server-role` in `sigma-1-dev` with rules:
   - apiGroups: [""], resources: ["configmaps", "secrets"], verbs: ["get", "list"]
3. Create a RoleBinding `sigma-1-pm-server-binding` binding the role to the service account.
4. Apply all three manifests.
5. Verify the service account exists and the RBAC permissions are correct via `kubectl auth can-i` checks.

## Validation
`kubectl get serviceaccount sigma-1-pm-server -n sigma-1-dev` exists. `kubectl auth can-i get configmaps -n sigma-1-dev --as=system:serviceaccount:sigma-1-dev:sigma-1-pm-server` returns 'yes'. `kubectl auth can-i get secrets -n sigma-1-dev --as=system:serviceaccount:sigma-1-dev:sigma-1-pm-server` returns 'yes'. `kubectl auth can-i create pods -n sigma-1-dev --as=system:serviceaccount:sigma-1-dev:sigma-1-pm-server` returns 'no'.