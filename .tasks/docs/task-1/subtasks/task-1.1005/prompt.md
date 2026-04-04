Implement subtask 1005: Create ExternalSecret CRDs for NOUS_API_KEY, DISCORD_WEBHOOK_URL, and SERVICE_API_KEY

## Objective
Create ExternalSecret CRDs that reference the cluster's backing secret store to sync `NOUS_API_KEY`, `DISCORD_WEBHOOK_URL`, and `SERVICE_API_KEY` into Kubernetes Secrets in the `sigma-1-dev` namespace.

## Steps
1. Create an ExternalSecret CR for `NOUS_API_KEY`: name=`sigma-1-nous-api-key`, namespace=`sigma-1-dev`, secretStoreRef pointing to the cluster's SecretStore/ClusterSecretStore, remoteRef.key=`sigma-1/nous-api-key`, target secret name=`sigma-1-nous-api-key`.
2. Create an ExternalSecret CR for `DISCORD_WEBHOOK_URL`: name=`sigma-1-discord-webhook-url`, remoteRef.key=`sigma-1/discord-webhook-url`, target secret name=`sigma-1-discord-webhook-url`.
3. Create an ExternalSecret CR for `SERVICE_API_KEY`: name=`sigma-1-service-api-key`, remoteRef.key=`sigma-1/service-api-key`, target secret name=`sigma-1-service-api-key`.
4. Apply all three manifests.
5. Verify each ExternalSecret reaches `Ready=True` within 2 minutes.
6. Verify the target Kubernetes Secrets exist and have non-empty data keys.

## Validation
All three ExternalSecrets (`sigma-1-nous-api-key`, `sigma-1-discord-webhook-url`, `sigma-1-service-api-key`) show `Ready=True` in status.conditions. All three target Kubernetes Secrets exist with non-empty data. Total synced secret count in namespace is 5 (combined with subtask 1004).