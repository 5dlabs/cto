# Secrets Management

This document describes how secrets are managed in the CTO platform.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           WORKSTATION                                        │
│  ┌──────────────┐                                                           │
│  │  1Password   │  ← Source of truth (create/manage secrets here)           │
│  │  (human      │                                                           │
│  │   access)    │                                                           │
│  └──────┬───────┘                                                           │
│         │                                                                   │
│         │  One-time seeding: ./infra/scripts/openbao/seed-from-1password.sh │
│         ▼                                                                   │
└─────────────────────────────────────────────────────────────────────────────┘
          │
          │ kubectl exec ... bao kv put ...
          ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           KUBERNETES CLUSTER                                 │
│                                                                             │
│  ┌──────────────┐      ┌─────────────────┐      ┌──────────────┐           │
│  │   OpenBao    │ ───► │ External Secrets│ ───► │  K8s Secrets │           │
│  │  (cluster    │      │    Operator     │      │  (consumed   │           │
│  │   secrets)   │      │                 │      │   by pods)   │           │
│  └──────────────┘      └─────────────────┘      └──────────────┘           │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Components

| Component | Role | Location |
|-----------|------|----------|
| **1Password** | Source of truth for credentials | Workstation access via `op` CLI |
| **OpenBao** | Cluster-native secrets store | `openbao` namespace |
| **External Secrets Operator** | Syncs OpenBao → K8s Secrets | `external-secrets` namespace |
| **ClusterSecretStore** | ESO configuration for OpenBao | `openbao` ClusterSecretStore |

## When to Run Seeding

The seeding script is a **one-time bootstrap operation**. Run it when:

| Trigger | Command |
|---------|---------|
| Setting up a new cluster | `./infra/scripts/openbao/seed-from-1password.sh` |
| After OpenBao reset/reinstall | `./infra/scripts/openbao/seed-from-1password.sh` |
| Adding a new agent | `./infra/scripts/openbao/seed-from-1password.sh --category github-apps` |
| Rotating credentials | Update 1Password, then re-run seeding |
| ExternalSecrets failing | Check ArgoCD, run seeding if secrets missing |

## Seeding Script

### Usage

```bash
# Preview changes (recommended first)
./infra/scripts/openbao/seed-from-1password.sh --dry-run

# Seed all secrets
./infra/scripts/openbao/seed-from-1password.sh

# Seed specific category
./infra/scripts/openbao/seed-from-1password.sh --category github-apps
./infra/scripts/openbao/seed-from-1password.sh --category linear-apps
./infra/scripts/openbao/seed-from-1password.sh --category api-keys
./infra/scripts/openbao/seed-from-1password.sh --category tools
./infra/scripts/openbao/seed-from-1password.sh --category infrastructure
./infra/scripts/openbao/seed-from-1password.sh --category research

# After seeding, refresh ExternalSecrets
kubectl annotate externalsecret -A --all force-sync="$(date +%s)" --overwrite
```

### Prerequisites

- 1Password CLI (`op`) installed and signed in
- `kubectl` configured with cluster access
- OpenBao running and unsealed

## Required 1Password Items

### GitHub Apps (14 agents)

Each agent needs a GitHub App in 1Password:

| Item Name | OpenBao Key | Fields Required |
|-----------|-------------|-----------------|
| `GitHub-App-Atlas` | `github-app-atlas` | `app-id`, `client-id`, `private-key` |
| `GitHub-App-Blaze` | `github-app-blaze` | `app-id`, `client-id`, `private-key` |
| `GitHub-App-Bolt` | `github-app-bolt` | `app-id`, `client-id`, `private-key` |
| `GitHub-App-Cipher` | `github-app-cipher` | `app-id`, `client-id`, `private-key` |
| `GitHub-App-Cleo` | `github-app-cleo` | `app-id`, `client-id`, `private-key` |
| `GitHub-App-Grizz` | `github-app-grizz` | `app-id`, `client-id`, `private-key` |
| `GitHub-App-Morgan` | `github-app-morgan` | `app-id`, `client-id`, `private-key` |
| `GitHub-App-Nova` | `github-app-nova` | `app-id`, `client-id`, `private-key` |
| `GitHub-App-Rex` | `github-app-rex` | `app-id`, `client-id`, `private-key` |
| `GitHub-App-Spark` | `github-app-spark` | `app-id`, `client-id`, `private-key` |
| `GitHub-App-Stitch` | `github-app-stitch` | `app-id`, `client-id`, `private-key` |
| `GitHub-App-Tap` | `github-app-tap` | `app-id`, `client-id`, `private-key` |
| `GitHub-App-Tess` | `github-app-tess` | `app-id`, `client-id`, `private-key` |
| `GitHub-App-Vex` | `github-app-vex` | `app-id`, `client-id`, `private-key` |

### Linear Apps (14 agents)

All Linear credentials are stored in one item with sections per agent:

| Item Name | Sections | Fields per Section |
|-----------|----------|-------------------|
| `Linear Agent Client Secrets (Rotated YYYY-MM-DD)` | Atlas, Blaze, Bolt, Cipher, Cleo, Grizz, Morgan, Nova, Rex, Spark, Tap, Tess, Vex | `client_id`, `client_secret`, `webhook_secret`, `access_token` |

### API Keys

Multiple 1Password items combine into one OpenBao secret:

| 1Password Item | Field | OpenBao Key Property |
|----------------|-------|---------------------|
| `Anthropic API Key` | `credential` | `ANTHROPIC_API_KEY` |
| `OpenAI API Key` | `credential` | `OPENAI_API_KEY` |
| `Google-Gemini API Key` | `api_key` | `GEMINI_API_KEY` |
| `Google API Key` | `credential` | `GOOGLE_API_KEY` |
| `Cursor API Key` | `credential` | `CURSOR_API_KEY` |
| `Context7 API Key` | `credential` | `CONTEXT7_API_KEY` |
| `Perplexity API Key` | `credential` | `PERPLEXITY_API_KEY` |
| `xAI API Key` | `credential` | `XAI_API_KEY` |
| `Factory API Key` | `credential` | `FACTORY_API_KEY` |
| `Brave Search API Key` | `api_key` | `BRAVE_API_KEY` |
| `MiniMax API Keys` | `Main API Key`, `Group ID` | `MINIMAX_API_KEY`, `MINIMAX_GROUP_ID` |

### Infrastructure Secrets

| 1Password Item | OpenBao Key | Fields |
|----------------|-------------|--------|
| `Cloudflare API` | `cloudflare` | `email`, `api-key` (credential) |
| `GHCR Pull Secret` | `ghcr-secret` | `.dockerconfigjson` (from credential JSON) |
| `Discord Alertmanager Webhook` | `alertmanager-discord` | `webhook-url` |
| `Linear API Credentials` | `linear-sync` | `LINEAR_API_KEY`, `LINEAR_WEBHOOK_SECRET`, etc. |

### Tool Secrets

| 1Password Item | OpenBao Key | Fields |
|----------------|-------------|--------|
| `GitHub PAT - Tools MCP Server` | `tools-github` | `GITHUB_PERSONAL_ACCESS_TOKEN` |
| `Firecrawl API Key` | `tools-firecrawl` | `FIRECRAWL_API_KEY` |
| `MCP-tavily API Key` | `tools-tavily` | `TAVILY_API_KEY` |
| `Latitude.sh API` | `tools-latitude` | `LATITUDE_API_KEY` |
| `Kubeconfig - Latitude cto-dal` | `tools-kubernetes` | `KUBECONFIG` |
| `Kubeconfig - Latitude cto-fra` | `tools-kubernetes-fra` | `KUBECONFIG` |
| `App Store Connect API` | `tools-appstore-connect` | `APP_STORE_KEY_ID`, `APP_STORE_ISSUER_ID`, `APP_STORE_P8_KEY` |

## Monitoring and Alerts

### ArgoCD PreSync Validation

Before `external-secrets-config` deploys, a PreSync hook validates that all required OpenBao secrets exist. If secrets are missing, the sync fails with instructions to run the seeding script.

See: `infra/gitops/manifests/external-secrets/presync-validation.yaml`

### Prometheus Alerts

The following alerts fire when ExternalSecrets fail:

| Alert | Severity | Description |
|-------|----------|-------------|
| `ExternalSecretSyncFailed` | critical | Single ExternalSecret failing for 5+ minutes |
| `MultipleExternalSecretsFailing` | critical | 3+ ExternalSecrets failing for 10+ minutes |
| `ExternalSecretStale` | warning | ExternalSecret hasn't synced in 24+ hours |

## Adding a New Secret

### 1. Create the 1Password Item

Create the item with the appropriate fields in 1Password.

### 2. Update the Seeding Script (if needed)

If adding a new category or changing field names, update:
`infra/scripts/openbao/seed-from-1password.sh`

### 3. Create the ExternalSecret

Add an ExternalSecret manifest in:
`infra/gitops/manifests/external-secrets/`

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: my-new-secret
  namespace: my-namespace
spec:
  refreshInterval: 1h
  secretStoreRef:
    kind: ClusterSecretStore
    name: openbao
  target:
    name: my-new-secret
    creationPolicy: Owner
  data:
    - secretKey: MY_KEY
      remoteRef:
        key: my-openbao-key
        property: MY_KEY
```

### 4. Run Seeding

```bash
./infra/scripts/openbao/seed-from-1password.sh --category <category>
kubectl annotate externalsecret -A --all force-sync="$(date +%s)" --overwrite
```

## Troubleshooting

### ExternalSecret Shows "SecretSyncedError"

1. Check if the OpenBao secret exists:
   ```bash
   BAO_TOKEN=$(kubectl get secret openbao-token -n openbao -o jsonpath='{.data.token}' | base64 -d)
   kubectl exec -n openbao openbao-0 -- sh -c "BAO_TOKEN='$BAO_TOKEN' bao kv get secret/<key>"
   ```

2. If missing, run the seeding script:
   ```bash
   ./infra/scripts/openbao/seed-from-1password.sh
   ```

3. Force refresh the ExternalSecret:
   ```bash
   kubectl annotate externalsecret <name> -n <namespace> force-sync="$(date +%s)" --overwrite
   ```

### OpenBao is Sealed

1. Get unseal keys from 1Password: `OpenBao Unseal Keys - CTO Platform`
2. Unseal OpenBao:
   ```bash
   kubectl exec -n openbao openbao-0 -- bao operator unseal <key1>
   kubectl exec -n openbao openbao-0 -- bao operator unseal <key2>
   kubectl exec -n openbao openbao-0 -- bao operator unseal <key3>
   ```

### 1Password CLI Not Signed In

```bash
eval "$(op signin)"
```
