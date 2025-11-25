# Secret Store Setup

This directory contains External Secrets configuration for the platform.
Secrets are stored in **HashiCorp Vault** and synced to Kubernetes namespaces
via External Secrets Operator.

## Architecture

```text
┌─────────────────────┐     ┌──────────────────────┐     ┌─────────────────┐
│  HashiCorp Vault    │────▶│  External Secrets    │────▶│  K8s Secret     │
│  (vault namespace)  │     │  Operator            │     │  (target ns)    │
└─────────────────────┘     └──────────────────────┘     └─────────────────┘
         │
         ▼
    KV v2 Secrets
    ├── secret/api-keys
    ├── secret/github-app-*
    ├── secret/github-pat
    ├── secret/ngrok-credentials
    └── ...
```

## Quick Start

### Adding a New Secret to Vault

```bash
# Port forward to Vault
kubectl port-forward svc/vault -n vault 8200:8200 &

# Login (get token from secure storage)
export VAULT_ADDR=http://127.0.0.1:8200
vault login <root_token>

# Add a secret
vault kv put secret/my-secret key1=value1 key2=value2

# Verify
vault kv get secret/my-secret
```

### Creating an ExternalSecret

After adding the secret to Vault, create an ExternalSecret to sync it:

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: my-secret
  namespace: target-namespace
spec:
  refreshInterval: 30s
  secretStoreRef:
    name: vault-secret-store
    kind: ClusterSecretStore
  target:
    name: my-secret
    creationPolicy: Owner
  data:
  - secretKey: KEY1
    remoteRef:
      key: my-secret
      property: key1
```

## Files

| File | Description |
|------|-------------|
| `vault-cluster-secret-store.yaml` | ClusterSecretStore pointing to Vault |
| `cluster-secret-store.yaml` | Legacy Kubernetes-backed store (deprecated) |
| `agent-secrets-external-secrets.yaml` | API keys and GitHub App credentials |
| `arc-external-secrets.yaml` | ARC runner GitHub PAT |
| `ngrok-operator-external-secrets.yaml` | ngrok credentials |
| `cloudflare-external-secrets.yaml` | Cloudflare API token |
| `ghcr-external-secrets.yaml` | GitHub Container Registry credentials |
| `toolman-external-secrets.yaml` | ToolMan service secrets |
| `argocd-repositories-external-secrets.yaml` | ArgoCD repo credentials |

## Vault Operations

### Unsealing After Restart

Vault seals when the pod restarts. To unseal:

```bash
kubectl exec -it vault-0 -n vault -- vault operator unseal
# Enter unseal key when prompted (repeat 3 times with different keys)
```

### Listing Secrets

```bash
kubectl port-forward svc/vault -n vault 8200:8200 &
export VAULT_ADDR=http://127.0.0.1:8200
vault login <token>
vault kv list secret/
```

### Backup

```bash
kubectl exec vault-0 -n vault -- \
  vault operator raft snapshot save /tmp/vault-snapshot.snap
kubectl cp vault/vault-0:/tmp/vault-snapshot.snap ./vault-backup-$(date +%Y%m%d).snap
```

## Verification

Check that External Secrets are syncing:

```bash
# Check ClusterSecretStore status
kubectl get clustersecretstore vault-secret-store

# Check ExternalSecrets across namespaces
kubectl get externalsecret -A

# Describe specific ExternalSecret
kubectl describe externalsecret <name> -n <namespace>

# Check synced secrets
kubectl get secret -n <namespace>
```

## Troubleshooting

### ExternalSecret Not Syncing

1. Check Vault is unsealed: `kubectl exec vault-0 -n vault -- vault status`
1. Check ClusterSecretStore: `kubectl get clustersecretstore vault-secret-store`
1. Check ESO logs: `kubectl logs -n external-secrets-system -l app=external-secrets`

### Secret Not Found in Vault

```bash
vault kv get secret/<path>  # Check if path is correct
vault kv list secret/       # List available secrets
```

## See Also

- [Vault Operations Runbook](../../docs/runbooks/vault-operations.md)
- [HashiCorp Vault Documentation](https://developer.hashicorp.com/vault/docs)
- [External Secrets Operator Docs](https://external-secrets.io/)
