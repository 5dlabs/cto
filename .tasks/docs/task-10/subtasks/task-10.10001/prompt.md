Implement subtask 10001: Define Kubernetes RBAC Roles and RoleBindings for all service accounts

## Objective
Create least-privilege RBAC Roles and RoleBindings for every service account in the production namespace, ensuring each service can only access the Kubernetes API resources it needs.

## Steps
1. Inventory all ServiceAccounts in the production namespace (backend services, Morgan agent, cloudflared, PostgreSQL operator, Redis operator, Grafana/Loki, etc.). 2. For each ServiceAccount, document the minimum Kubernetes API permissions required (e.g., Morgan may need to read ConfigMaps; cloudflared needs no API access; CNPG operator needs broad access within its scope). 3. Create Role resources scoped to the production namespace with minimal verbs (get, list, watch — avoid wildcard). 4. Create RoleBindings linking each Role to its corresponding ServiceAccount. 5. Remove any default ClusterRoleBindings that grant excessive permissions to namespace service accounts. 6. Create a ClusterRole and ClusterRoleBinding for the CNPG operator if not already managed by the operator's Helm chart. 7. Apply all RBAC manifests via `kubectl apply`.

## Validation
For each service account, use `kubectl auth can-i --as=system:serviceaccount:{ns}:{sa}` to verify it CAN perform its required operations and CANNOT perform unauthorized operations (e.g., backend SA cannot delete pods, cannot access secrets it doesn't own). Create a test matrix of allowed/denied operations and validate each cell.