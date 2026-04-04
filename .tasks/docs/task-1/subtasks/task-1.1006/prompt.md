Implement subtask 1006: Create ServiceAccount sigma-1-pm-sa with minimal RBAC

## Objective
Create a Kubernetes ServiceAccount `sigma-1-pm-sa` in the `sigma-1-dev` namespace with a Role and RoleBinding granting read-only access to ConfigMaps and Secrets within the namespace only.

## Steps
1. Create a ServiceAccount manifest: name=`sigma-1-pm-sa`, namespace=`sigma-1-dev`.
2. Create a Role manifest: name=`sigma-1-pm-role`, namespace=`sigma-1-dev`, rules: apiGroups=[''], resources=['configmaps','secrets'], verbs=['get','list','watch'].
3. Create a RoleBinding manifest: name=`sigma-1-pm-rolebinding`, namespace=`sigma-1-dev`, roleRef to `sigma-1-pm-role`, subject=ServiceAccount `sigma-1-pm-sa`.
4. Apply all three manifests.
5. Verify the ServiceAccount, Role, and RoleBinding exist.
6. Verify using `kubectl auth can-i` that the SA can get configmaps and secrets but cannot create/delete pods in the namespace.

## Validation
`kubectl get sa sigma-1-pm-sa -n sigma-1-dev` exists. `kubectl auth can-i get configmaps -n sigma-1-dev --as=system:serviceaccount:sigma-1-dev:sigma-1-pm-sa` returns `yes`. `kubectl auth can-i get secrets -n sigma-1-dev --as=system:serviceaccount:sigma-1-dev:sigma-1-pm-sa` returns `yes`. `kubectl auth can-i create pods -n sigma-1-dev --as=system:serviceaccount:sigma-1-dev:sigma-1-pm-sa` returns `no`.