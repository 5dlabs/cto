Implement subtask 10001: Create dedicated ServiceAccounts for PM server and frontend

## Objective
Define and apply dedicated ServiceAccount resources for each workload in the sigma1-prod namespace, ensuring no pod uses the default ServiceAccount.

## Steps
1. Create a YAML manifest `infra/rbac/serviceaccounts.yaml`.
2. Define `sa-pm-server` ServiceAccount in namespace `sigma1-prod` with `automountServiceAccountToken: true`.
3. Define `sa-frontend` ServiceAccount in namespace `sigma1-prod` with `automountServiceAccountToken: true`.
4. Apply the manifest: `kubectl apply -f infra/rbac/serviceaccounts.yaml`.
5. Verify both ServiceAccounts exist: `kubectl get sa -n sigma1-prod`.

## Validation
Run `kubectl get sa -n sigma1-prod` and confirm `sa-pm-server` and `sa-frontend` are listed. Describe each SA to verify namespace and metadata are correct.