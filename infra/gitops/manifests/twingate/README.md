# Twingate Resources

This directory defines Twingate **resource + access policy CRs** managed by the
Twingate Kubernetes Operator.

It does **not** deploy the operator or connector pods directly.

## Ownership Split

- `infra/gitops/applications/operators/twingate-operator.yaml`: deploys the operator in `operators`
- `infra/gitops/applications/networking/twingate-connector.yaml`: deploys connector pods in `cto`
- `infra/gitops/applications/networking/twingate.yaml`: deploys the CRs from this directory into `operators`

## Prerequisites

1. Argo applications are synced:
   - `twingate-operator`
   - `twingate`
   - `twingate-connector`
2. Twingate API token is available to the operator via secret `twingate-api-secret` in `operators`.
3. Connector token secret `twingate-connector-tokens` exists in namespace `cto`.

## Managed Resources (Current)

`kustomization.yaml` currently applies six `v1beta` CRs:

- `TwingateResource`
  - `cto-pod-network` (`10.42.0.0/16`)
  - `cto-service-network` (`10.43.0.0/16`)
  - `openclaw-tool-server` (`openclaw-tool-server.openclaw.svc.cluster.local`)
- `TwingateResourceAccess`
  - `cto-pod-network-everyone`
  - `cto-service-network-everyone`
  - `openclaw-tool-server-everyone`

Each `TwingateResourceAccess` binds a resource to one Twingate principal ID.

## Verification

```bash
# Argo app health/sync state
kubectl get applications.argoproj.io -n argocd \
  twingate-operator twingate twingate-connector

# Operator + connector pods
kubectl get pods -n operators -l app.kubernetes.io/name=twingate-operator
kubectl get pods -n cto -l app.kubernetes.io/name=twingate-connector

# Applied v1beta resources
kubectl get twingateresource -n operators
kubectl get twingateresourceaccess -n operators
```

## Troubleshooting

- `twingate` OutOfSync and references old `v1alpha1` objects:
  - Confirm `ignoreDifferences` in `infra/gitops/applications/networking/twingate.yaml`
  - Re-sync app and verify only `v1beta` resources remain
- Resource exists but cannot be reached by users:
  - Check matching `TwingateResourceAccess` object exists for that resource
  - Verify expected principal ID in `spec.principalId`
- Connector healthy but traffic still fails:
  - Check operator logs for reconciliation errors:
    - `kubectl logs -n operators -l app.kubernetes.io/name=twingate-operator`
  - Check connector logs:
    - `kubectl logs -n cto -l app.kubernetes.io/name=twingate-connector`
