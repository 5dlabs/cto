Implement subtask 10005: Audit and tighten RBAC with dedicated Roles for all service accounts

## Objective
Audit all existing RoleBindings and ClusterRoleBindings in sigma-1-dev, remove overly permissive bindings, and create dedicated least-privilege Roles/RoleBindings for each service account.

## Steps
1. Audit existing RBAC: `kubectl get rolebindings,clusterrolebindings -n sigma-1-dev -o yaml` and identify any wildcard permissions or cluster-admin bindings.
2. Remove or replace any overly permissive bindings.
3. Create a dedicated Role for the PM server service account (`sigma-1-pm-sa`):
   ```yaml
   apiVersion: rbac.authorization.k8s.io/v1
   kind: Role
   metadata:
     name: sigma-1-pm-role
     namespace: sigma-1-dev
   rules:
   - apiGroups: [""]
     resources: ["configmaps", "secrets"]
     verbs: ["get", "list", "watch"]
   ```
4. Create RoleBinding binding `sigma-1-pm-role` to `sigma-1-pm-sa`.
5. For any other service accounts (bridge, workers, etc.), create similarly scoped Roles granting only the minimum required permissions.
6. Ensure no SA has `create pods`, `delete pods`, or any cluster-wide write permissions.
7. Apply all RBAC manifests.
8. Verify with `kubectl auth can-i` for each SA.

## Validation
Run `kubectl auth can-i --as=system:serviceaccount:sigma-1-dev:sigma-1-pm-sa create pods -n sigma-1-dev` and assert `no`. Run `kubectl auth can-i --as=system:serviceaccount:sigma-1-dev:sigma-1-pm-sa get configmaps -n sigma-1-dev` and assert `yes`. Run `kubectl auth can-i --as=system:serviceaccount:sigma-1-dev:sigma-1-pm-sa get secrets -n sigma-1-dev` and assert `yes`. Run `kubectl auth can-i --as=system:serviceaccount:sigma-1-dev:sigma-1-pm-sa '*' '*' -n sigma-1-dev` and assert `no`.