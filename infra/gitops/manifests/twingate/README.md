# Twingate Manifests

This directory contains the `TwingateResource` and `TwingateResourceAccess` objects managed by Argo CD application [`infra/gitops/applications/networking/twingate.yaml`](../../applications/networking/twingate.yaml).

## Ownership model

Twingate is split across three Argo CD applications:

1. `twingate-operator`
   - File: [`infra/gitops/applications/operators/twingate-operator.yaml`](../../applications/operators/twingate-operator.yaml)
   - Deploys the operator chart and sets `network` + `remoteNetworkId` in Helm values.
2. `twingate`
   - File: [`infra/gitops/applications/networking/twingate.yaml`](../../applications/networking/twingate.yaml)
   - Applies resources from this directory (`TwingateResource` + `TwingateResourceAccess`).
3. `twingate-connector`
   - File: [`infra/gitops/applications/networking/twingate-connector.yaml`](../../applications/networking/twingate-connector.yaml)
   - Deploys connector pods from the official Helm chart in namespace `cto`.

The operator application includes `ignoreDifferences` for normalized CRD fields and stale legacy objects. Keep those entries unless the CRD/controller behavior changes.

## Managed resources in this directory

- `pod-network.yaml`: `TwingateResource` for `10.42.0.0/16`
- `service-network.yaml`: `TwingateResource` for `10.43.0.0/16`
- `openclaw-tool-server.yaml`: `TwingateResource` for `openclaw-tool-server.openclaw.svc.cluster.local`
- `*-access.yaml`: `TwingateResourceAccess` bindings for principal `R3JvdXA6NzQ2MDQy`

## Secret prerequisites

External Secrets sync from OpenBao key `secret/tools-twingate` into:

- `operators/twingate-api-secret` (`apiToken`)
- `cto/twingate-connector-tokens` (`TWINGATE_ACCESS_TOKEN`, `TWINGATE_REFRESH_TOKEN`)

Source of truth: [`infra/gitops/manifests/external-secrets/cto-secrets.yaml`](../external-secrets/cto-secrets.yaml).

## Verification

```bash
# Argo status
kubectl get application twingate-operator -n argocd
kubectl get application twingate -n argocd
kubectl get application twingate-connector -n argocd

# Operator-managed objects (namespace: operators)
kubectl get twingateresource -n operators
kubectl get twingateresourceaccess -n operators

# Connector deployment (namespace: cto)
kubectl get pods -n cto -l app.kubernetes.io/name=twingate-connector

# Operator logs
kubectl logs -n operators -l app.kubernetes.io/name=twingate-operator --tail=100
```

## Common drift symptoms

- Argo `OutOfSync` on `twingateresourceaccesses.twingate.com` schema fields:
  - Expected after CRD normalization; see `ignoreDifferences` in `twingate-operator.yaml`.
- Resources synced but no client access:
  - Check `principalId` values in `*-access.yaml` and verify membership in Twingate admin.
- Connector app healthy but no tunnel traffic:
  - Verify `twingate-connector-tokens` secret exists in `cto` and tokens are valid.
