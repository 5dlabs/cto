Implement subtask 1003: Create ExternalSecret CRD for sigma-1-discord-webhook

## Objective
Define and apply the ExternalSecret resource for the Discord webhook URL, referencing the cluster's existing SecretStore.

## Steps
1. Create `externalsecret-discord-webhook.yaml` in the sigma-1 namespace:
   - `metadata.name`: `sigma-1-discord-webhook`
   - `metadata.labels`: include `sigma-1-pipeline: infra`
   - `spec.secretStoreRef`: reference the cluster's existing ClusterSecretStore
   - `spec.target.name`: `sigma-1-discord-webhook`
   - `spec.data[0].secretKey`: `DISCORD_WEBHOOK_URL`
   - `spec.data[0].remoteRef.key`: the backing store path for the Discord webhook
   - `spec.refreshInterval`: `1h`
2. Apply with `kubectl apply -f externalsecret-discord-webhook.yaml -n sigma-1`.
3. Wait for the ExternalSecret status to show `SecretSynced`.

## Validation
`kubectl get externalsecret sigma-1-discord-webhook -n sigma-1 -o jsonpath='{.status.conditions[0].reason}'` returns 'SecretSynced'. The resulting Secret `sigma-1-discord-webhook` exists and has a non-empty `DISCORD_WEBHOOK_URL` data key.