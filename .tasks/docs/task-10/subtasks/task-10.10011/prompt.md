Implement subtask 10011: RBAC: create per-service ServiceAccounts with minimal roles

## Objective
Create dedicated Kubernetes ServiceAccounts for each sigma1 service with minimal RBAC Roles and RoleBindings scoped to only what each service needs.

## Steps
Step-by-step:
1. For each service (equipment-catalog, rms, finance, customer-vetting, social-engine), create:
   - `ServiceAccount` named `sa-<service-name>` in sigma1 namespace
   - `Role` with minimal permissions (most services need NO Kubernetes API access; the Role can be empty or omitted)
   - `RoleBinding` binding the Role to the ServiceAccount
2. Update each Deployment manifest to set `spec.template.spec.serviceAccountName: sa-<service-name>` and `automountServiceAccountToken: false` (unless the service specifically needs K8s API access).
3. For the GDPR orchestrator Job (created later), create `sa-gdpr-orchestrator` with permission to read ConfigMaps (for service discovery) but nothing else.
4. Verify no service uses the `default` ServiceAccount.

## Validation
For each service, exec into the pod and attempt `curl https://kubernetes.default.svc/api/v1/namespaces/sigma1/secrets -H 'Authorization: Bearer $(cat /var/run/secrets/kubernetes.io/serviceaccount/token)'` — verify 403 Forbidden or confirm token is not mounted. Verify `kubectl get sa -n sigma1` lists all expected ServiceAccounts.