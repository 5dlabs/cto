Implement subtask 10004: Create RoleBindings linking ServiceAccounts to their Roles

## Objective
Bind each dedicated ServiceAccount to its corresponding least-privilege Role via RoleBinding resources.

## Steps
1. Create `infra/rbac/rolebindings.yaml`.
2. Define RoleBinding `pm-server-rolebinding`:
   - roleRef: Role `pm-server-role`
   - subjects: ServiceAccount `sa-pm-server` in namespace `sigma1-prod`.
3. Define RoleBinding `frontend-rolebinding`:
   - roleRef: Role `frontend-role`
   - subjects: ServiceAccount `sa-frontend` in namespace `sigma1-prod`.
4. Apply: `kubectl apply -f infra/rbac/rolebindings.yaml`.
5. Verify: `kubectl describe rolebinding pm-server-rolebinding -n sigma1-prod`.

## Validation
Exec into a PM server pod and run `kubectl auth can-i list pods --as=system:serviceaccount:sigma1-prod:sa-pm-server -n sigma1-prod` — should return 'no'. Run `kubectl auth can-i get configmaps --as=system:serviceaccount:sigma1-prod:sa-pm-server -n sigma1-prod` — should return 'yes'. Exec into a frontend pod and verify it cannot access secrets.