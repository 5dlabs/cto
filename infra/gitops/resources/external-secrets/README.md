# External Secrets Configuration

This directory contains the External Secrets Operator configuration for syncing secrets from OpenBao to Kubernetes.

## Overview

The External Secrets Operator automatically syncs secrets from OpenBao into Kubernetes Secrets. This eliminates the need for manual secret creation after OpenBao is unsealed.

## Components

- **cluster-secret-store.yaml** - ClusterSecretStore that connects to OpenBao
- **argocd-secrets.yaml** - ExternalSecrets for the `argocd` namespace (repository credentials)
- **cto-secrets.yaml** - ExternalSecrets for the `cto` namespace
- **infra-secrets.yaml** - ExternalSecrets for the `infra` namespace
- **minio-secrets.yaml** - ExternalSecrets for the `minio-screenshots` namespace

## Prerequisites

1. **External Secrets Operator** must be installed (deployed via `external-secrets.yaml` ArgoCD Application)
2. **OpenBao** must be deployed and **unsealed**
3. **OpenBao root token** must be stored as a Kubernetes secret

## Setup

### 1. Unseal OpenBao

If OpenBao is sealed (e.g., after pod restart), unseal it using keys from 1Password:

```bash
# Get unseal keys from 1Password
op item get "OpenBao Unseal Keys - CTO Platform" --format=json | \
  jq -r '.fields[] | select(.label | test("Unseal Key"; "i")) | "\(.label): \(.value)"'

# Unseal (need 3 of 5 keys)
kubectl exec -n openbao openbao-0 -- bao operator unseal <KEY1>
kubectl exec -n openbao openbao-0 -- bao operator unseal <KEY2>
kubectl exec -n openbao openbao-0 -- bao operator unseal <KEY3>
```

### 2. Create OpenBao Token Secret

The External Secrets Operator needs a token to authenticate with OpenBao:

```bash
# Get root token from 1Password and create the secret
ROOT_TOKEN=$(op item get "OpenBao Unseal Keys - CTO Platform" --format=json | \
  jq -r '.fields[] | select(.label == "password" or .label == "Root Token") | .value')

kubectl create secret generic openbao-token \
  --from-literal=token="$ROOT_TOKEN" \
  -n openbao \
  --dry-run=client -o yaml | kubectl apply -f -
```

### 3. Verify Secret Sync

Once the ClusterSecretStore is connected, ExternalSecrets will automatically sync:

```bash
# Check ClusterSecretStore status
kubectl get clustersecretstores openbao -o jsonpath='{.status.conditions}'

# Check ExternalSecret status in a namespace
kubectl get externalsecrets -n cto

# Verify a synced secret
kubectl get secret ghcr-secret -n cto -o yaml
```

## Adding New Secrets

To add a new secret that should be synced from OpenBao:

1. Store the secret in OpenBao:
   ```bash
   ROOT_TOKEN=$(op item get "OpenBao Unseal Keys - CTO Platform" --format=json | \
     jq -r '.fields[] | select(.label == "password" or .label == "Root Token") | .value')
   
   kubectl exec -n openbao openbao-0 -- env BAO_TOKEN="$ROOT_TOKEN" \
     bao kv put secret/my-new-secret key1=value1 key2=value2
   ```

2. Create an ExternalSecret resource in the appropriate file (e.g., `cto-secrets.yaml`)

3. Commit and push - ArgoCD will sync the new ExternalSecret

## Troubleshooting

### ClusterSecretStore not ready

Check if the openbao-token secret exists and has a valid token:
```bash
kubectl get secret openbao-token -n openbao
kubectl exec -n openbao openbao-0 -- bao status
```

### ExternalSecret not syncing

Check the ExternalSecret status:
```bash
kubectl describe externalsecret <name> -n <namespace>
```

Common issues:
- OpenBao is sealed
- Token is invalid or expired
- Secret path doesn't exist in OpenBao

### Secret data mismatch

If the secret exists but has wrong data, delete it and let ESO recreate it:
```bash
kubectl delete secret <name> -n <namespace>
# ESO will recreate it within the refreshInterval (default: 1h)
```

## Secrets Mapping

| OpenBao Path | Kubernetes Secret | Namespace |
|--------------|-------------------|-----------|
| `secret/tools-github` | `argocd-repo-creds` | `argocd` |
| `secret/ghcr-secret` | `ghcr-secret` | `cto` |
| `secret/api-keys` | `cto-secrets` | `cto` |
| `secret/linear-sync` | `linear-secrets` | `cto` |
| `secret/research-twitter` | `research-twitter-secrets` | `cto` |
| `secret/tools-github` | `tools-github-secrets` | `cto` |
| `secret/tools-firecrawl` | `tools-firecrawl-secrets` | `cto` |
| `secret/tools-kubernetes` | `tools-kubernetes-secrets` | `cto` |
| `secret/github-app-morgan` | `github-app-5dlabs-morgan` | `cto` |
| `secret/cloudflare` | `cloudflare-api-credentials` | `infra` |
| `secret/minio-screenshots-credentials` | `minio-screenshots-credentials` | `minio-screenshots` |

## ArgoCD Repository Credentials

The `argocd-repo-creds` secret is critical for ArgoCD to function. It provides credentials for all `https://github.com/5dlabs/*` repositories. If ArgoCD shows errors like "could not read Username for 'https://github.com'", this secret is likely missing.

**Important:** After unsealing OpenBao and creating the token secret, the ArgoCD repo credentials will be automatically synced within the refresh interval (1 hour) or you can force sync by deleting the ExternalSecret and letting it recreate:

```bash
kubectl delete externalsecret argocd-repo-creds -n argocd
# ESO will immediately recreate and sync the secret
```
