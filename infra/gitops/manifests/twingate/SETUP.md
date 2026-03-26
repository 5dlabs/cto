# Twingate Setup Runbook

This runbook reflects the current split deployment model:

- Operator app: [`infra/gitops/applications/operators/twingate-operator.yaml`](../../applications/operators/twingate-operator.yaml)
- Resource app: [`infra/gitops/applications/networking/twingate.yaml`](../../applications/networking/twingate.yaml)
- Connector app: [`infra/gitops/applications/networking/twingate-connector.yaml`](../../applications/networking/twingate-connector.yaml)

## 1. Seed OpenBao secrets

Store the required values under `secret/tools-twingate`.

```bash
# Get OpenBao root token from 1Password
ROOT_TOKEN=$(op item get "OpenBao Unseal Keys - CTO Platform" --format=json | \
  jq -r '.fields[] | select(.label == "password" or .label == "Root Token") | .value')

# Get Twingate API key
TWINGATE_API_TOKEN=$(op item get "Twingate API Key" --fields credential --reveal)

# Set connector tokens (from Twingate Admin or API)
TWINGATE_CONNECTOR_ACCESS_TOKEN="<set-me>"
TWINGATE_CONNECTOR_REFRESH_TOKEN="<set-me>"

kubectl exec -n openbao openbao-0 -- env BAO_TOKEN="$ROOT_TOKEN" \
  bao kv put secret/tools-twingate \
  TWINGATE_API_TOKEN="$TWINGATE_API_TOKEN" \
  TWINGATE_CONNECTOR_ACCESS_TOKEN="$TWINGATE_CONNECTOR_ACCESS_TOKEN" \
  TWINGATE_CONNECTOR_REFRESH_TOKEN="$TWINGATE_CONNECTOR_REFRESH_TOKEN"
```

## 2. Confirm ExternalSecret sync

Expected Kubernetes secrets:

- `operators/twingate-api-secret` with key `apiToken`
- `cto/twingate-connector-tokens` with keys `TWINGATE_ACCESS_TOKEN`, `TWINGATE_REFRESH_TOKEN`

```bash
kubectl get externalsecret twingate-api-secret -n operators
kubectl get externalsecret twingate-connector-tokens -n cto
kubectl get secret twingate-api-secret -n operators
kubectl get secret twingate-connector-tokens -n cto
```

## 3. Sync and verify Argo CD apps

```bash
kubectl get application twingate-operator -n argocd
kubectl get application twingate -n argocd
kubectl get application twingate-connector -n argocd
```

If an app is not healthy/synced, trigger sync:

```bash
argocd app sync twingate-operator
argocd app sync twingate
argocd app sync twingate-connector
```

## 4. Validate runtime state

```bash
# Operator resources
kubectl get twingateresource -n operators
kubectl get twingateresourceaccess -n operators

# Connector pods
kubectl get pods -n cto -l app.kubernetes.io/name=twingate-connector

# Operator logs
kubectl logs -n operators -l app.kubernetes.io/name=twingate-operator --tail=100
```

## Troubleshooting

### Argo `OutOfSync` on Twingate CRDs

The operator app intentionally ignores normalized CRD fields (`twingateresourceaccesses.twingate.com`) via `ignoreDifferences`. Confirm the ignore list still exists in [`infra/gitops/applications/operators/twingate-operator.yaml`](../../applications/operators/twingate-operator.yaml).

### `twingate-connector` pods CrashLoop

1. Check token secret keys and values in `cto/twingate-connector-tokens`.
2. Verify connector network slug in `twingate-connector.yaml` (`connector.network`).
3. Check pod logs:

```bash
kubectl logs -n cto -l app.kubernetes.io/name=twingate-connector --tail=200
```

### Resources exist but users cannot reach services

1. Verify `principalId` values in `*-access.yaml` map to the intended Twingate group.
2. Verify resource addresses/CIDRs match the cluster networking (`10.42.0.0/16` pods, `10.43.0.0/16` services).
3. Confirm users are in the group referenced by `principalId`.
