# Twingate Resources

This directory contains Twingate custom resources managed by the Twingate Kubernetes Operator.

## Prerequisites

1. **Twingate Operator** must be installed (see `infra/gitops/applications/operators/twingate-operator.yaml`)
2. **API Token** must be stored in OpenBao at `secret/tools-twingate` with property `TWINGATE_API_TOKEN`

## Adding the API Token to OpenBao

Before the operator can function, you must store the Twingate API token in OpenBao:

```bash
# Get the OpenBao root token from 1Password
ROOT_TOKEN=$(op item get "OpenBao Unseal Keys - CTO Platform" --format=json | \
  jq -r '.fields[] | select(.label == "password" or .label == "Root Token") | .value')

# Store the Twingate API token (replace with your actual token from Twingate Admin Console)
kubectl exec -n openbao openbao-0 -- env BAO_TOKEN="$ROOT_TOKEN" \
  bao kv put secret/tools-twingate TWINGATE_API_TOKEN="<YOUR_TWINGATE_API_TOKEN>"

# Verify it was stored
kubectl exec -n openbao openbao-0 -- env BAO_TOKEN="$ROOT_TOKEN" \
  bao kv get secret/tools-twingate
```

**Security Note:** The API token is only shown here for initial setup. After adding it to OpenBao, it will be automatically synced to the Kubernetes secret via External Secrets Operator. The token is **never stored in Git** - only the ExternalSecret configuration that references OpenBao.

## Resources

- **RemoteNetwork** (`remote-network.yaml`) - Represents the Latitude cluster in Twingate
- **Connector** (`connector.yaml`) - Deploys connector pods that establish the secure tunnel
- **Network Resource** (`network-resource.yaml`) - Exposes the pod CIDR (10.4.0.0/16) to Twingate clients

## Network Access

Once configured, Twingate clients can access:
- **Pod Network**: `10.4.0.0/16` (all pods in the cluster)
- **Services**: Via pod network access

## Verification

After deployment, verify the resources:

```bash
# Check RemoteNetwork
kubectl get twingateremotenetwork -n operators

# Check Connector
kubectl get twingateconnector -n operators

# Check Network Resource
kubectl get twingateresource -n operators

# Check operator logs
kubectl logs -n operators -l app.kubernetes.io/name=twingate-operator
```
