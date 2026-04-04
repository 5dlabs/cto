Implement subtask 1002: Create sigma-1-secrets Kubernetes Secret with 4 keys

## Objective
Create the Kubernetes Secret `sigma-1-secrets` in the sigma-1-dev namespace containing LINEAR_API_KEY, DISCORD_WEBHOOK_URL, NOUS_API_KEY, and GITHUB_TOKEN. Use external-secrets CRs if available, otherwise sealed-secrets placeholders for dev.

## Steps
1. Create a Secret manifest `secret.yaml` of type Opaque in namespace `sigma-1-dev`.
2. Define exactly 4 keys: `LINEAR_API_KEY`, `DISCORD_WEBHOOK_URL`, `NOUS_API_KEY`, `GITHUB_TOKEN`.
3. If external-secrets operator is available in the cluster, create an ExternalSecret CR that syncs these keys from the external secret store into `sigma-1-secrets`.
4. If external-secrets is not available, create a SealedSecret manifest with dev placeholder values that can be replaced before production.
5. Apply the manifest and verify the secret exists with all 4 keys.
6. Do NOT commit plaintext secret values to the repository — use sealed-secrets or external-secrets patterns only.

## Validation
`kubectl get secret sigma-1-secrets -n sigma-1-dev` exists. `kubectl get secret sigma-1-secrets -n sigma-1-dev -o jsonpath='{.data}'` contains exactly 4 keys: LINEAR_API_KEY, DISCORD_WEBHOOK_URL, NOUS_API_KEY, GITHUB_TOKEN. Each key has a non-empty base64-encoded value.