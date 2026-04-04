Implement subtask 1002: Create ExternalSecret CRD for sigma-1-linear-token

## Objective
Define and apply the ExternalSecret resource for the Linear API token, referencing the cluster's existing SecretStore and targeting the correct backing store path.

## Steps
1. Create `externalsecret-linear-token.yaml` in the sigma-1 namespace:
   - `metadata.name`: `sigma-1-linear-token`
   - `metadata.labels`: include `sigma-1-pipeline: infra`
   - `spec.secretStoreRef`: reference the cluster's existing ClusterSecretStore (name TBD per decision point)
   - `spec.target.name`: `sigma-1-linear-token`
   - `spec.data[0].secretKey`: `LINEAR_API_TOKEN`
   - `spec.data[0].remoteRef.key`: the backing store path for the Linear token
   - `spec.refreshInterval`: `1h`
2. Apply with `kubectl apply -f externalsecret-linear-token.yaml -n sigma-1`.
3. Wait for the ExternalSecret status to show `SecretSynced`.

## Validation
`kubectl get externalsecret sigma-1-linear-token -n sigma-1 -o jsonpath='{.status.conditions[0].reason}'` returns 'SecretSynced'. The resulting Secret `sigma-1-linear-token` exists and has a non-empty `LINEAR_API_TOKEN` data key.