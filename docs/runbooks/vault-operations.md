# HashiCorp Vault Operations Runbook

This runbook covers operational procedures for the HashiCorp Vault deployment.

## Architecture Overview

- **Deployment Mode**: Standalone with Raft integrated storage
- **Unseal Method**: Manual Shamir keys (5 shares, 3 threshold)
- **Namespace**: `vault`
- **Service**: `vault.vault:8200`
- **Integration**: External Secrets Operator via Kubernetes auth

## Initial Setup

### Prerequisites

Install the Vault CLI:

```bash
# macOS
brew tap hashicorp/tap
brew install hashicorp/tap/vault

# Linux
curl -fsSL https://apt.releases.hashicorp.com/gpg | sudo apt-key add -
sudo apt-add-repository \
  "deb [arch=amd64] https://apt.releases.hashicorp.com $(lsb_release -cs) main"
sudo apt-get update && sudo apt-get install vault
```

### First-Time Initialization

After Vault is deployed via ArgoCD, run the initialization script:

```bash
cd infra/scripts
./vault-init.sh
```

This script will:

1. Initialize Vault with 5 key shares (3 threshold)
1. Unseal Vault
1. Enable KV v2 secrets engine at `secret/`
1. Configure Kubernetes authentication
1. Create the `external-secrets` policy and role

**CRITICAL**: The script saves unseal keys to a JSON file. Secure these keys
immediately (password manager, hardware security module, etc.) and delete
the file.

### Migrate Secrets from Kubernetes

After initialization, migrate existing secrets:

```bash
# Port forward to Vault
kubectl port-forward svc/vault -n vault 8200:8200 &

# Set environment variables (use values from initialization)
export VAULT_ADDR=http://127.0.0.1:8200
export VAULT_TOKEN=<root_token>

# Run migration
./migrate-secrets-to-vault.sh
```

---

## Common Operations

### Unsealing After Restart

Vault seals automatically when the pod restarts. You must unseal it manually.

```bash
# Check seal status
kubectl exec vault-0 -n vault -- vault status

# Unseal (repeat 3 times with different keys)
kubectl exec -it vault-0 -n vault -- vault operator unseal
# Enter unseal key when prompted
```

Or via port-forward:

```bash
kubectl port-forward svc/vault -n vault 8200:8200 &
export VAULT_ADDR=http://127.0.0.1:8200

vault operator unseal <key1>
vault operator unseal <key2>
vault operator unseal <key3>
```

### Adding a New Secret

```bash
# Port forward
kubectl port-forward svc/vault -n vault 8200:8200 &
export VAULT_ADDR=http://127.0.0.1:8200

# Login with root token
vault login <root_token>

# Add a secret
vault kv put secret/my-secret key1=value1 key2=value2

# Verify
vault kv get secret/my-secret
```

### Updating an Existing Secret

```bash
vault kv put secret/api-keys ANTHROPIC_API_KEY=sk-ant-xxx NEW_KEY=value
```

### Listing Secrets

```bash
# List all secrets at root
vault kv list secret/

# List metadata for a specific secret
vault kv metadata get secret/api-keys
```

### Viewing Secret History

```bash
# Get current version
vault kv get secret/api-keys

# Get specific version
vault kv get -version=2 secret/api-keys

# List all versions
vault kv metadata get secret/api-keys
```

### Deleting a Secret

```bash
# Soft delete (can be recovered)
vault kv delete secret/old-secret

# Permanent delete (unrecoverable)
vault kv destroy -versions=1,2,3 secret/old-secret
vault kv metadata delete secret/old-secret
```

---

## Backup and Recovery

### Creating a Backup

```bash
# Create Raft snapshot
kubectl exec vault-0 -n vault -- \
  vault operator raft snapshot save /tmp/vault-snapshot.snap

# Copy to local machine
kubectl cp vault/vault-0:/tmp/vault-snapshot.snap \
  ./vault-backup-$(date +%Y%m%d).snap

# Clean up
kubectl exec vault-0 -n vault -- rm /tmp/vault-snapshot.snap
```

**Recommendation**: Automate daily backups with a CronJob.

### Restoring from Backup

```bash
# Copy snapshot to pod
kubectl cp ./vault-backup.snap vault/vault-0:/tmp/vault-snapshot.snap

# Restore (Vault must be unsealed)
kubectl exec vault-0 -n vault -- \
  vault operator raft snapshot restore /tmp/vault-snapshot.snap
```

---

## Troubleshooting

### Vault Pod Not Starting

Check pod status and logs:

```bash
kubectl get pods -n vault
kubectl describe pod vault-0 -n vault
kubectl logs vault-0 -n vault
```

Common issues:

- PVC not bound: Check storage class availability
- Image pull errors: Verify network/registry access

### External Secrets Not Syncing

1. Check ClusterSecretStore status:

```bash
kubectl get clustersecretstore vault-secret-store -o yaml
```

1. Check ExternalSecret status:

```bash
kubectl get externalsecret -A
kubectl describe externalsecret <name> -n <namespace>
```

1. Verify Vault is unsealed:

```bash
kubectl exec vault-0 -n vault -- vault status
```

1. Check ESO service account permissions:

```bash
# Test authentication from ESO namespace
kubectl run vault-test --rm -it --restart=Never \
  --image=hashicorp/vault:1.21.0 \
  --serviceaccount=external-secrets \
  -n external-secrets-system \
  -- vault login -method=kubernetes role=external-secrets
```

### Cannot Connect to Vault

1. Check service exists:

```bash
kubectl get svc -n vault
```

1. Check endpoint:

```bash
kubectl get endpoints vault -n vault
```

1. Test connectivity from another pod:

```bash
kubectl run curl-test --rm -it --restart=Never --image=curlimages/curl \
  -- curl -s http://vault.vault:8200/v1/sys/health
```

---

## Security Best Practices

### Root Token Management

1. After initial setup, create admin tokens with limited TTL
1. Revoke the root token if not needed:

```bash
vault token revoke <root_token>
```

1. Generate new root token only when needed using unseal keys

### Unseal Key Distribution

- Distribute unseal keys to different team members
- Store in separate secure locations
- Never store all keys in one place
- Consider using a password manager with shared vaults

### Audit Logging

Enable audit logging for compliance:

```bash
vault audit enable file file_path=/vault/audit/audit.log
```

---

## Monitoring

### Health Check Endpoints

- `/v1/sys/health` - Overall health
- `/v1/sys/seal-status` - Seal status

### Metrics

Vault exposes Prometheus metrics. To enable:

1. Add to Vault config:

```hcl
telemetry {
  prometheus_retention_time = "30s"
  disable_hostname = true
}
```

1. Create ServiceMonitor for Prometheus Operator

### Alerts to Configure

- Vault sealed (critical)
- Vault pod not ready (critical)
- High token creation rate (warning)
- Approaching storage limit (warning)

---

## Migration from Kubernetes SecretStore

### Phase 1: Deploy Vault

1. Merge Vault ArgoCD application to main
1. Wait for sync and pod running
1. Run `vault-init.sh`

### Phase 2: Migrate Secrets

1. Run `migrate-secrets-to-vault.sh`
1. Verify secrets in Vault: `vault kv list secret/`

### Phase 3: Update ExternalSecrets

Update each ExternalSecret to use `vault-secret-store`:

```yaml
spec:
  secretStoreRef:
    name: vault-secret-store  # Changed from secret-store
    kind: ClusterSecretStore
```

Migrate in order:

1. Low-risk: tools
1. Medium-risk: ngrok, cloudflare, GHCR
1. Higher-risk: GitHub Apps, API keys
1. Last: ARC runner secrets

### Phase 4: Cleanup

After verifying all ExternalSecrets sync:

1. Remove old ClusterSecretStore
1. Delete secrets from `secret-store` namespace
1. Archive old secret-store manifests

---

## Quick Reference

| Command | Description |
|---------|-------------|
| `vault status` | Check seal status |
| `vault operator unseal` | Unseal Vault |
| `vault kv list secret/` | List secrets |
| `vault kv get secret/<path>` | Read a secret |
| `vault kv put secret/<path> k=v` | Write a secret |
| `vault operator raft snapshot save` | Create backup |
| `vault login` | Authenticate |
