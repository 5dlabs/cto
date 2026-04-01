Implement subtask 10002: Update all Deployment specs to use dedicated ServiceAccounts

## Objective
Modify every Deployment in sigma1-prod to reference its dedicated ServiceAccount, removing any reliance on the default ServiceAccount.

## Steps
1. Edit the PM server Deployment spec: set `spec.template.spec.serviceAccountName: sa-pm-server` and `automountServiceAccountToken: true`.
2. Edit the frontend Deployment spec: set `spec.template.spec.serviceAccountName: sa-frontend` and `automountServiceAccountToken: true`.
3. Ensure no Deployment omits `serviceAccountName` (which would default to `default`).
4. Apply updated Deployments and wait for rollout to complete.
5. Verify with: `kubectl get pods -n sigma1-prod -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.spec.serviceAccountName}{"\n"}{end}'` — no pod should show `default`.

## Validation
Run `kubectl get pods -n sigma1-prod -o jsonpath='{.items[*].spec.serviceAccountName}'` and confirm output contains only `sa-pm-server` and `sa-frontend`, never `default`.