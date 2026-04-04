Implement subtask 10003: Harden RBAC for PM server ServiceAccount

## Objective
Tighten the sigma-1-pm-server ServiceAccount RBAC to read-only access for configmaps and secrets in the sigma-1-dev namespace only. Ensure no cluster-wide permissions exist.

## Steps
1. Edit or create `manifests/production/rbac-pm-server.yaml`.
2. Define a `Role` (not ClusterRole) in namespace `sigma-1-dev` named `sigma-1-pm-server-role` with rules:
   - apiGroups: [""], resources: ["configmaps", "secrets"], verbs: ["get", "list", "watch"]
3. Define a `RoleBinding` named `sigma-1-pm-server-binding` binding the Role to `ServiceAccount:sigma-1-pm-server` in namespace `sigma-1-dev`.
4. Remove any existing ClusterRoleBinding or ClusterRole that grants the sigma-1-pm-server ServiceAccount broader permissions.
5. Apply the manifest: `kubectl apply -f manifests/production/rbac-pm-server.yaml`.
6. Verify with `kubectl auth can-i` commands.

## Validation
`kubectl auth can-i create secrets --as=system:serviceaccount:sigma-1-dev:sigma-1-pm-server -n sigma-1-dev` returns 'no'. `kubectl auth can-i get configmaps --as=system:serviceaccount:sigma-1-dev:sigma-1-pm-server -n sigma-1-dev` returns 'yes'. `kubectl auth can-i get pods --as=system:serviceaccount:sigma-1-dev:sigma-1-pm-server -n default` returns 'no' (no cross-namespace access).