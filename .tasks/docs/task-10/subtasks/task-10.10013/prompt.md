Implement subtask 10013: Secret rotation: document process and configure automation if external-secrets-operator is available

## Objective
Document the process for rotating all secrets (DB passwords, API keys, Stripe keys) and configure external-secrets-operator for automatic rotation if available in the cluster.

## Steps
Step-by-step:
1. Create `docs/secret-rotation-runbook.md` documenting:
   - List of all secrets in sigma1 and sigma1-db namespaces with their purpose
   - Manual rotation procedure for each: DB passwords (update CloudNativePG secret + rolling restart), Stripe keys (update K8s secret + rolling restart), OpenCorporates API key, Google API key, Signal-CLI credentials
   - Rotation schedule recommendation: DB passwords quarterly, API keys annually, Stripe keys on suspected compromise
2. Check if external-secrets-operator (ESO) is installed: `kubectl get crd | grep externalsecrets`
3. If ESO is available:
   - Create ExternalSecret CRs that sync from the configured secret store (e.g., Vault, AWS Secrets Manager)
   - Configure `refreshInterval` for automatic rotation
   - Set up annotations on Deployments for automatic restart on secret change (e.g., Reloader or stakater/reloader)
4. If ESO is not available: note this in the runbook and recommend installation as a Phase 2 improvement.

## Validation
Verify the runbook document exists and covers all secrets. If ESO is configured, change a secret value in the external store and verify the Kubernetes Secret updates within the refresh interval and the affected pod restarts. If manual, follow the runbook to rotate one non-critical secret (e.g., a test API key) and verify the service picks up the new value after restart.