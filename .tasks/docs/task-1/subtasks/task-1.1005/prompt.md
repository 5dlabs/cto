Implement subtask 1005: Create ExternalSecret CRD for sigma-1-github-token

## Objective
Define and apply the ExternalSecret resource for the GitHub PAT used to access the 5dlabs/sigma-1 repository.

## Steps
1. Create `externalsecret-github-token.yaml` in the sigma-1 namespace:
   - `metadata.name`: `sigma-1-github-token`
   - `metadata.labels`: include `sigma-1-pipeline: infra`
   - `spec.secretStoreRef`: reference the cluster's existing ClusterSecretStore
   - `spec.target.name`: `sigma-1-github-token`
   - `spec.data[0].secretKey`: `GITHUB_TOKEN`
   - `spec.data[0].remoteRef.key`: the backing store path for the GitHub PAT
   - `spec.refreshInterval`: `1h`
2. Apply with `kubectl apply -f externalsecret-github-token.yaml -n sigma-1`.
3. Wait for the ExternalSecret status to show `SecretSynced`.

## Validation
`kubectl get externalsecret sigma-1-github-token -n sigma-1 -o jsonpath='{.status.conditions[0].reason}'` returns 'SecretSynced'. The resulting Secret `sigma-1-github-token` exists and has a non-empty `GITHUB_TOKEN` data key.