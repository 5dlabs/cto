Implement subtask 10004: Create frontend ServiceAccount with read-only API access

## Objective
Create a separate ServiceAccount for the frontend deployment with minimal read-only permissions. Only applicable if D5 includes Tasks 6-9.

## Steps
1. Create `manifests/production/rbac-frontend.yaml`.
2. Define a `ServiceAccount` named `sigma-1-frontend` in namespace `sigma-1-dev`.
3. Define a `Role` named `sigma-1-frontend-role` with minimal rules — read-only access to configmaps only (the frontend should call the PM server API, not directly access secrets):
   - apiGroups: [""], resources: ["configmaps"], verbs: ["get", "list"]
4. Define a `RoleBinding` named `sigma-1-frontend-binding` binding the Role to the frontend ServiceAccount.
5. Update the frontend Deployment manifest to use `serviceAccountName: sigma-1-frontend`.
6. Apply: `kubectl apply -f manifests/production/rbac-frontend.yaml`.

## Validation
`kubectl auth can-i get configmaps --as=system:serviceaccount:sigma-1-dev:sigma-1-frontend -n sigma-1-dev` returns 'yes'. `kubectl auth can-i get secrets --as=system:serviceaccount:sigma-1-dev:sigma-1-frontend -n sigma-1-dev` returns 'no'. `kubectl auth can-i list pods --as=system:serviceaccount:sigma-1-dev:sigma-1-frontend -n sigma-1-dev` returns 'no'.