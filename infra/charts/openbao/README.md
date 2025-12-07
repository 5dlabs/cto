# OpenBao Configuration

OpenBao is an open-source fork of HashiCorp Vault, maintained by the Linux Foundation under the MPL 2.0 license.

## Quick Start (Kind)

```bash
# Add Helm repo
helm repo add openbao https://openbao.github.io/openbao-helm
helm repo update

# Create namespace
kubectl create namespace openbao

# Install OpenBao
helm install openbao openbao/openbao \
  --namespace openbao \
  --values values-kind.yaml

# Apply supplementary RBAC for service registration
kubectl apply -f rbac-service-registration.yaml

# Initialize and store credentials in 1Password (recommended)
./scripts/init-openbao.sh --dev

# Or manually initialize (save output securely!)
kubectl exec -n openbao openbao-0 -- bao operator init \
  -key-shares=1 \
  -key-threshold=1 \
  -format=json

# Port forward for UI access
kubectl port-forward -n openbao svc/openbao 8201:8200
# Access at http://localhost:8201
```

## Files

| File | Description |
|------|-------------|
| `values-kind.yaml` | Helm values for Kind cluster deployment |
| `rbac-service-registration.yaml` | RBAC for Kubernetes service registration (fixes 403 errors) |
| `scripts/init-openbao.sh` | Initialize OpenBao and store credentials in 1Password |
| `scripts/unseal-openbao.sh` | Unseal OpenBao using credentials from 1Password |

## Initialization Scripts

### Prerequisites
- `kubectl` configured with cluster access
- `op` (1Password CLI) installed and authenticated
- `jq` installed

### Initialize a New Instance

```bash
# Development mode (1 key, 1 threshold)
./scripts/init-openbao.sh --dev

# Production mode (5 keys, 3 threshold)
./scripts/init-openbao.sh --key-shares 5 --threshold 3

# Custom 1Password item title
./scripts/init-openbao.sh --dev --title "MyApp OpenBao Prod"

# Custom namespace
./scripts/init-openbao.sh --namespace my-openbao --dev
```

### Unseal After Pod Restart

```bash
# Using default item title (OpenBao - <namespace>)
./scripts/unseal-openbao.sh

# Custom 1Password item title
./scripts/unseal-openbao.sh --title "MyApp OpenBao Prod"
```

### Retrieve Credentials Manually

```bash
# List all fields
op item get "OpenBao - openbao" --reveal

# Get just the root token
op item get "OpenBao - openbao" --field password --reveal

# Get unseal keys
op item get "OpenBao - openbao" --field "Unseal Key 1" --reveal
```

## Key Differences from HashiCorp Vault

| Item | HashiCorp Vault | OpenBao |
|------|-----------------|---------|
| CLI command | `vault` | `bao` |
| Image | `hashicorp/vault` | `quay.io/openbao/openbao` |
| Data path | `/vault/data` | `/openbao/data` |
| Helm repo | `helm.releases.hashicorp.com` | `openbao.github.io/openbao-helm` |
| License | BSL 1.1 | MPL 2.0 (open source) |

## Service Registration Labels

After applying the RBAC, OpenBao will update pod labels for service discovery:

- `openbao-active` - Whether this node is the active leader
- `openbao-sealed` - Whether the vault is sealed
- `openbao-initialized` - Whether the vault has been initialized
- `openbao-perf-standby` - Whether this is a performance standby

## Resources

- [OpenBao Documentation](https://openbao.org/docs/)
- [OpenBao Helm Chart](https://github.com/openbao/openbao-helm)
- [Migration from Vault](https://openbao.org/docs/migration/)

