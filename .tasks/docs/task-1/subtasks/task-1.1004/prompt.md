Implement subtask 1004: Create ExternalSecret CRD for sigma-1-nous-api-key (optional)

## Objective
Define and apply the ExternalSecret resource for NOUS_API_KEY, marked as optional so that its absence does not block the pipeline.

## Steps
1. Create `externalsecret-nous-api-key.yaml` in the sigma-1 namespace:
   - `metadata.name`: `sigma-1-nous-api-key`
   - `metadata.labels`: include `sigma-1-pipeline: infra`
   - `metadata.annotations`: add `sigma-1/optional: "true"` to signal validation job this is optional
   - `spec.secretStoreRef`: reference the cluster's existing ClusterSecretStore
   - `spec.target.name`: `sigma-1-nous-api-key`
   - `spec.target.creationPolicy`: `Merge` (or `Owner` depending on ESO version — ensure it doesn't block if remote key is missing)
   - `spec.data[0].secretKey`: `NOUS_API_KEY`
   - `spec.data[0].remoteRef.key`: the backing store path for NOUS_API_KEY
   - `spec.refreshInterval`: `1h`
2. Apply with `kubectl apply -f externalsecret-nous-api-key.yaml -n sigma-1`.
3. Note: This ExternalSecret may remain in a non-synced state if the backing key doesn't exist — that is acceptable per D8 graceful skip requirements.

## Validation
`kubectl get externalsecret sigma-1-nous-api-key -n sigma-1` exists. If the backing key exists, status shows 'SecretSynced' and Secret contains non-empty `NOUS_API_KEY`. If the backing key is absent, the ExternalSecret resource still exists (may show error status) but no hard failure occurs in apply.