# Vault Secrets Operator (VSO) Configuration

This directory contains the Vault Secrets Operator configuration for syncing secrets from HashiCorp Vault to Kubernetes.

## Architecture

```
Vault Server (vault namespace)
    ↓
VaultConnection (vault-secrets-operator namespace)
    ↓
VaultAuth (kubernetes auth method)
    ↓
VaultStaticSecret (various namespaces)
    ↓
Kubernetes Secret (target namespace)
```

## Directory Structure

- `vault-connection.yaml` - VaultConnection CRD defining how to connect to Vault
- `vault-auth.yaml` - VaultAuth CRD defining Kubernetes authentication
- `secrets/` - VaultStaticSecret resources organized by category:
  - `api-keys.yaml` - API keys for agent platform
  - `github-apps.yaml` - GitHub App credentials for all agents
  - `infrastructure.yaml` - Infrastructure secrets (Cloudflare, NGrok, GitHub PAT, etc.)
  - `doc-server.yaml` - Documentation server configuration
  - `ghcr.yaml` - GitHub Container Registry credentials
  - `tools.yaml` - Tools service secrets

## Secrets to Add in Vault UI

After running `vault-init.sh`, access Vault UI and create the following secrets at `secret/`:

### API Keys (`secret/api-keys`)

| Key | Description |
|-----|-------------|
| `ANTHROPIC_API_KEY` | Anthropic Claude API key |
| `CURSOR_API_KEY` | Cursor API key |
| `OPENAI_API_KEY` | OpenAI API key |
| `GOOGLE_API_KEY` | Google API key |
| `XAI_API_KEY` | xAI API key |
| `GEMINI_API_KEY` | Google Gemini API key |
| `FACTORY_API_KEY` | Factory API key |
| `CONTEXT7_API_KEY` | Context7 API key |
| `PERPLEXITY_API_KEY` | Perplexity API key |

### GitHub Apps

Create one secret per agent at `secret/github-app-<name>`:

| Agent | Path | Keys |
|-------|------|------|
| Rex | `secret/github-app-rex` | `app-id`, `private-key`, `client-id` |
| Blaze | `secret/github-app-blaze` | `app-id`, `private-key`, `client-id` |
| Cleo | `secret/github-app-cleo` | `app-id`, `private-key`, `client-id` |
| Tess | `secret/github-app-tess` | `app-id`, `private-key`, `client-id` |
| Atlas | `secret/github-app-atlas` | `app-id`, `private-key`, `client-id` |
| Bolt | `secret/github-app-bolt` | `app-id`, `private-key`, `client-id` |
| Morgan | `secret/github-app-morgan` | `app-id`, `private-key`, `client-id` |
| Cipher | `secret/github-app-cipher` | `app-id`, `private-key`, `client-id` |
| Stitch | `secret/github-app-stitch` | `app-id`, `private-key`, `client-id` |

### Infrastructure Secrets

| Path | Keys | Description |
|------|------|-------------|
| `secret/cloudflare` | `api-token` | Cloudflare API token for external-dns |
| `secret/ngrok-credentials` | `API_KEY`, `AUTHTOKEN` | NGrok operator credentials |
| `secret/github-pat` | `token`, `username` | GitHub PAT for ARC and ArgoCD |
| `secret/github-webhooks` | `secret` | GitHub webhook secret |
| `secret/redis-auth` | `password` | Redis authentication password |

### ArgoCD Repository Credentials

| Path | Keys |
|------|------|
| `secret/argocd-repo-charts` | `type` (git), `url`, `username` (oauth2), `password` |
| `secret/argocd-repo-cto` | `type` (git), `url`, `username` (oauth2), `password` |

### Doc Server Configuration

| Path | Keys |
|------|------|
| `secret/doc-server-config` | `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `VECTOR_DATABASE_URL`, `DATABASE_URL` |

### GHCR (Docker Registry)

| Path | Keys |
|------|------|
| `secret/ghcr` | `.dockerconfigjson` (full JSON blob for docker auth) |

Example `.dockerconfigjson` value:
```json
{"auths":{"ghcr.io":{"username":"<github-username>","password":"<github-pat>","auth":"<base64-encoded-username:password>"}}}
```

### Tools Secrets

| Path | Keys |
|------|------|
| `secret/tools-brave-search` | `BRAVE_API_KEY` |
| `secret/tools-kubernetes` | `KUBECONFIG` |
| `secret/tools-reddit` | `REDDIT_CLIENT_ID`, `REDDIT_CLIENT_SECRET`, `REDDIT_USERNAME`, `REDDIT_PASSWORD` |
| `secret/tools-context7` | `CONTEXT7_API_KEY` |
| `secret/tools-github` | `GITHUB_PERSONAL_ACCESS_TOKEN` |

## Initialization Steps

1. Ensure Vault and VSO are deployed via ArgoCD
2. Run the initialization script:
   ```bash
   cd infra/scripts
   ./vault-init.sh
   ```
3. Save the unseal keys securely and delete the generated file
4. Access Vault UI:
   ```bash
   kubectl port-forward svc/vault -n vault 8200:8200
   ```
5. Open http://localhost:8200 and login with root token
6. Navigate to Secrets > secret/ and create each secret listed above
7. VaultStaticSecrets will automatically sync to Kubernetes

## Unsealing After Restart

Vault uses Shamir's Secret Sharing and must be unsealed after each pod restart:

```bash
kubectl port-forward svc/vault -n vault 8200:8200 &
export VAULT_ADDR=http://127.0.0.1:8200
vault operator unseal <key1>
vault operator unseal <key2>
vault operator unseal <key3>
```

You need 3 of the 5 unseal keys to unseal Vault.

## Troubleshooting

### VaultStaticSecret not syncing

1. Check VSO controller logs:
   ```bash
   kubectl logs -n vault-secrets-operator -l app.kubernetes.io/name=vault-secrets-operator
   ```

2. Check VaultStaticSecret status:
   ```bash
   kubectl get vaultstaticsecret -A
   kubectl describe vaultstaticsecret <name> -n <namespace>
   ```

3. Verify Vault auth is working:
   ```bash
   kubectl exec -n vault vault-0 -- vault auth list
   kubectl exec -n vault vault-0 -- vault read auth/kubernetes/role/vault-secrets-operator
   ```

### Vault is sealed

Check if Vault needs unsealing:
```bash
kubectl exec -n vault vault-0 -- vault status
```

If `Sealed: true`, run the unseal procedure above.

