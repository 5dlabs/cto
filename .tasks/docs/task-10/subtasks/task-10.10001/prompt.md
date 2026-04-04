Implement subtask 10001: Create RBAC ServiceAccount and Role with least-privilege permissions

## Objective
Create the ServiceAccount `sigma-1-pipeline-sa`, a Role `sigma-1-pipeline-role` with read-only access to ConfigMaps and Secrets in the sigma-1 namespace (no write access to Secrets), and a RoleBinding linking them together. All manifests go in the sigma-1 Helm chart or kustomize overlay.

## Steps
Step-by-step:
1. Create `rbac/serviceaccount.yaml` defining ServiceAccount `sigma-1-pipeline-sa` in namespace `sigma-1`.
2. Create `rbac/role.yaml` defining Role `sigma-1-pipeline-role` with rules:
   - apiGroups: [""], resources: ["configmaps", "secrets"], verbs: ["get", "list", "watch"]
   - Explicitly exclude create, update, patch, delete on secrets.
3. Create `rbac/rolebinding.yaml` binding the Role to the ServiceAccount.
4. Ensure all resources have labels `app.kubernetes.io/part-of: sigma-1` and `app.kubernetes.io/component: rbac`.
5. If cross-namespace bridge access is needed, create a separate ClusterRole manifest (gated by a Helm values flag `rbac.crossNamespace.enabled`).

## Validation
Run `kubectl auth can-i get secrets -n sigma-1 --as=system:serviceaccount:sigma-1:sigma-1-pipeline-sa` → 'yes'. Run `kubectl auth can-i create secrets -n sigma-1 --as=system:serviceaccount:sigma-1:sigma-1-pipeline-sa` → 'no'. Run `kubectl auth can-i get secrets -n default --as=system:serviceaccount:sigma-1:sigma-1-pipeline-sa` → 'no'. Verify all three RBAC resources exist via `kubectl get sa,role,rolebinding -n sigma-1`.