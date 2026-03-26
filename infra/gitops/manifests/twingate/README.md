# Twingate Manifests

This directory contains `TwingateResource` and `TwingateResourceAccess` CRs managed by Argo CD application `twingate`.

It does not deploy the operator or connector pods directly.

## Ownership model

Twingate is split across three Argo CD applications:

1. `twingate-operator`
   - File: `infra/gitops/applications/operators/twingate-operator.yaml`
   - Deploys the operator and sets network/remote network values.
2. `twingate`
   - File: `infra/gitops/applications/networking/twingate.yaml`
   - Applies resources from this directory (`TwingateResource` + `TwingateResourceAccess`).
3. `twingate-connector`
   - File: `infra/gitops/applications/networking/twingate-connector.yaml`
   - Deploys connector pods in namespace `cto`.

The operator application includes `ignoreDifferences` for normalized CRD fields and legacy drift entries. Keep those until CRD/controller behavior changes.

## Managed resources in this directory

- `pod-network.yaml`: `TwingateResource` for `10.42.0.0/16`
- `service-network.yaml`: `TwingateResource` for `10.43.0.0/16`
- `openclaw-tool-server.yaml`: `TwingateResource` for `openclaw-tool-server.openclaw.svc.cluster.local`
- `*-access.yaml`: `TwingateResourceAccess` bindings for the configured principal

## Secret prerequisites

External Secrets sync from OpenBao key `secret/tools-twingate` into:

- `operators/twingate-api-secret` (`apiToken`)
- `cto/twingate-connector-tokens` (`TWINGATE_ACCESS_TOKEN`, `TWINGATE_REFRESH_TOKEN`)

Source of truth: `infra/gitops/manifests/external-secrets/cto-secrets.yaml`.

## Verification

```bash
# Argo app health/sync
kubectl get applications.argoproj.io -n argocd \
  twingate-operator twingate twingate-connector

# Operator and connector pods
kubectl get pods -n operators -l app.kubernetes.io/name=twingate-operator
kubectl get pods -n cto -l app.kubernetes.io/name=twingate-connector

# Applied resources
kubectl get twingateresource -n operators
kubectl get twingateresourceaccess -n operators

# Operator logs
kubectl logs -n operators -l app.kubernetes.io/name=twingate-operator --tail=100
```

## Troubleshooting

- `twingate` appears OutOfSync on schema fields:
  - Check `ignoreDifferences` in `twingate-operator.yaml`.
- Resources synced but clients cannot reach target:
  - Verify corresponding `TwingateResourceAccess` exists.
  - Verify expected `principalId` and Twingate group membership.
- Connector healthy but no tunnel traffic:
  - Verify `twingate-connector-tokens` exists in `cto` and tokens are valid.
