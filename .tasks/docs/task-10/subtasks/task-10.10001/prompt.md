Implement subtask 10001: Define Kubernetes RBAC Roles and RoleBindings for all service accounts

## Objective
Create least-privilege RBAC Role and RoleBinding (or ClusterRole/ClusterRoleBinding) manifests for every service account in the application namespace, ensuring each service can only access the resources it needs.

## Steps
1. Inventory all ServiceAccounts currently in use across all Deployments, StatefulSets, CronJobs, and operators. 2. For each ServiceAccount, determine the minimum set of Kubernetes API resources and verbs it needs (e.g., backend services may only need `get` on ConfigMaps/Secrets, cloudflared needs none, operators need specific CRD access). 3. Create Role manifests scoped to the application namespace for each service account with only the required rules. 4. Create corresponding RoleBinding manifests binding each Role to its ServiceAccount. 5. For cluster-scoped needs (e.g., operators that watch across namespaces), create ClusterRole/ClusterRoleBinding with tight resource scoping. 6. Remove any existing overly-permissive bindings (e.g., default `cluster-admin` grants). 7. Apply all manifests and verify with `kubectl auth can-i --as=system:serviceaccount:<ns>:<sa>` for each account.

## Validation
For each service account, run `kubectl auth can-i --list --as=system:serviceaccount:<ns>:<sa>` and verify the permissions match the documented minimum set. Attempt an unauthorized action (e.g., `kubectl auth can-i delete pods --as=system:serviceaccount:<ns>:backend-sa`) and confirm it returns 'no'. Verify all services continue to function correctly after RBAC is applied.