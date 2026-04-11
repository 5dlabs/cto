---
name: secrets-management
description: External Secrets Operator and OpenBao patterns for secure secret synchronization and management.
agents: [bolt, cipher]
triggers: [secret, openbao, vault, external-secrets, eso, credentials, token]
---

# Secrets Management

Secure secret synchronization using External Secrets Operator (ESO) and OpenBao.

## Architecture Overview

```
┌─────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   OpenBao   │────►│ External Secrets │────►│ K8s Secrets     │
│   (Vault)   │     │    Operator      │     │ (auto-synced)   │
└─────────────┘     └──────────────────┘     └─────────────────┘
```

## OpenBao Setup

OpenBao is the secrets vault (HashiCorp Vault fork, MPL 2.0 licensed).

### Initialize and Unseal

```bash
# Initialize (first time only)
kubectl exec -n openbao openbao-0 -- bao operator init

# Unseal (required after pod restart)
kubectl exec -n openbao openbao-0 -- bao operator unseal <key1>
kubectl exec -n openbao openbao-0 -- bao operator unseal <key2>
kubectl exec -n openbao openbao-0 -- bao operator unseal <key3>

# Verify status
kubectl exec -n openbao openbao-0 -- bao status
```

### Enable KV Secrets Engine

```bash
kubectl exec -n openbao openbao-0 -- bao secrets enable -path=secret kv-v2
```

### Store a Secret

```bash
kubectl exec -n openbao openbao-0 -- bao kv put secret/myapp/db \
  username=myuser \
  password=mypassword
```

## External Secrets Operator

ESO syncs secrets from OpenBao to Kubernetes Secrets.

### ClusterSecretStore

Cluster-wide connection to OpenBao:

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ClusterSecretStore
metadata:
  name: openbao
spec:
  provider:
    vault:
      server: "http://openbao.openbao.svc:8200"
      path: "secret"
      version: "v2"
      auth:
        tokenSecretRef:
          name: openbao-token
          namespace: external-secrets
          key: token
```

### ExternalSecret

Sync specific secrets to a namespace:

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: myapp-db-credentials
  namespace: myapp
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: openbao
    kind: ClusterSecretStore
  
  target:
    name: myapp-db-credentials  # K8s Secret name
    creationPolicy: Owner
  
  data:
    - secretKey: username       # Key in K8s Secret
      remoteRef:
        key: secret/myapp/db    # Path in OpenBao
        property: username      # Field in OpenBao secret
    
    - secretKey: password
      remoteRef:
        key: secret/myapp/db
        property: password
```

### Template Secrets

Transform secrets during sync:

```yaml
spec:
  target:
    name: myapp-connection-string
    template:
      type: Opaque
      data:
        DATABASE_URL: |
          postgresql://{{ .username }}:{{ .password }}@db.svc:5432/myapp
  
  data:
    - secretKey: username
      remoteRef:
        key: secret/myapp/db
        property: username
    - secretKey: password
      remoteRef:
        key: secret/myapp/db
        property: password
```

## Common Patterns

### Database Credentials

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: postgres-credentials
  namespace: databases
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: openbao
    kind: ClusterSecretStore
  target:
    name: postgres-credentials
  dataFrom:
    - extract:
        key: secret/databases/postgres
```

### API Keys

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: api-keys
  namespace: myapp
spec:
  refreshInterval: 30m
  secretStoreRef:
    name: openbao
    kind: ClusterSecretStore
  data:
    - secretKey: OPENAI_API_KEY
      remoteRef:
        key: secret/apis/openai
        property: api_key
    - secretKey: GITHUB_TOKEN
      remoteRef:
        key: secret/apis/github
        property: token
```

## Validation Commands

```bash
# Check ExternalSecret sync status
kubectl get externalsecrets -A
kubectl describe externalsecret <name> -n <namespace>

# Check synced K8s Secret
kubectl get secret <name> -n <namespace> -o yaml

# Check ClusterSecretStore status
kubectl get clustersecretstores

# OpenBao status
kubectl exec -n openbao openbao-0 -- bao status
```

## Troubleshooting

### ExternalSecret not syncing

```bash
# Check ESO controller logs
kubectl logs -n external-secrets -l app.kubernetes.io/name=external-secrets

# Check ExternalSecret status
kubectl describe externalsecret <name> -n <namespace>
```

### OpenBao sealed after restart

```bash
# Check seal status
kubectl exec -n openbao openbao-0 -- bao status

# Unseal (need 3 of 5 keys typically)
kubectl exec -n openbao openbao-0 -- bao operator unseal
```

## Best Practices

1. **Never commit secrets** - Use ExternalSecrets for all credentials
2. **Use refresh intervals** - 1h for stable secrets, 30m for rotating
3. **Scope secrets narrowly** - One ExternalSecret per application
4. **Template connection strings** - Avoid exposing raw credentials
5. **Monitor sync status** - Alert on ExternalSecret failures
6. **Backup unseal keys** - Store securely outside cluster
