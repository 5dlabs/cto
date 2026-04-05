# Rex's Solution: Kubernetes Operator + ExternalSecrets Pattern

> Historical analysis. Superseded by the PM-managed `client_credentials` token broker flow documented in `tests/cli-invocation/docs/LINEAR-OAUTH.md`.

## Philosophy

Token management is infrastructure, not application logic. Treat OAuth tokens like any other secret in Kubernetes - let specialized infrastructure handle rotation.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Kubernetes Cluster                        │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────────┐     ┌──────────────────────────────┐ │
│  │ linear-oauth-    │     │ ExternalSecret               │ │
│  │ controller       │────▶│ (syncs to Secret)            │ │
│  │ (CronJob/5min)   │     └──────────────────────────────┘ │
│  └──────────────────┘               │                       │
│          │                          ▼                       │
│          │               ┌──────────────────────────────┐  │
│          │               │ linear-oauth-tokens (Secret) │  │
│          ▼               │ - MORGAN_ACCESS_TOKEN        │  │
│  ┌──────────────────┐    │ - MORGAN_REFRESH_TOKEN       │  │
│  │ OpenBao/Vault    │◀───│ - MORGAN_EXPIRES_AT          │  │
│  │ (Source of truth)│    └──────────────────────────────┘  │
│  └──────────────────┘               │                       │
│                                     │ (mounted as env vars) │
│                                     ▼                       │
│                         ┌──────────────────────────────┐   │
│                         │ PM Server, Controller, etc.  │   │
│                         └──────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## Implementation

### 1. Token Storage (OpenBao/Vault)

```hcl
# Path: secret/data/linear/morgan
{
  "access_token": "lin_oauth_...",
  "refresh_token": "lin_ref_...",
  "expires_at": 1738000000,
  "client_id": "752f67d53b2b0dab2191832ba0aa43d9",
  "client_secret": "..."
}
```

### 2. Token Refresh Controller (Rust binary)

```rust
// crates/linear-oauth-controller/src/main.rs
use chrono::{Duration, Utc};

const REFRESH_BUFFER: Duration = Duration::hours(24);

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Read current token from OpenBao
    let token_data = openbao_client.read("secret/data/linear/morgan").await?;
    
    // 2. Check if refresh needed (expires within 24 hours)
    let expires_at = token_data.expires_at;
    let refresh_at = expires_at - REFRESH_BUFFER;
    
    if Utc::now() > refresh_at {
        info!("Token expires in < 24h, refreshing...");
        
        // 3. Call Linear OAuth refresh endpoint
        let new_token = refresh_linear_token(
            &token_data.refresh_token,
            &token_data.client_id,
            &token_data.client_secret,
        ).await?;
        
        // 4. Write new token to OpenBao
        openbao_client.write("secret/data/linear/morgan", TokenData {
            access_token: new_token.access_token,
            refresh_token: new_token.refresh_token,
            expires_at: Utc::now().timestamp() + new_token.expires_in,
            ..token_data
        }).await?;
        
        info!("Token refreshed successfully, new expiry: {}", new_token.expires_in);
        
        // 5. Emit metric/alert
        metrics::counter!("linear_oauth_refresh_success").increment(1);
    } else {
        info!("Token still valid until {}, no refresh needed", expires_at);
    }
    
    Ok(())
}
```

### 3. Kubernetes CronJob

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: linear-oauth-refresh
  namespace: cto
spec:
  schedule: "*/5 * * * *"  # Every 5 minutes
  concurrencyPolicy: Forbid
  jobTemplate:
    spec:
      template:
        spec:
          serviceAccountName: linear-oauth-controller
          containers:
          - name: refresh
            image: ghcr.io/5dlabs/linear-oauth-controller:latest
            env:
            - name: OPENBAO_ADDR
              value: "http://openbao.vault:8200"
            - name: LINEAR_CLIENT_ID
              valueFrom:
                secretKeyRef:
                  name: linear-app-morgan
                  key: client_id
            - name: LINEAR_CLIENT_SECRET
              valueFrom:
                secretKeyRef:
                  name: linear-app-morgan
                  key: client_secret
          restartPolicy: OnFailure
```

### 4. ExternalSecret for Token Distribution

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: linear-oauth-tokens
  namespace: cto
spec:
  refreshInterval: 1m
  secretStoreRef:
    name: openbao
    kind: ClusterSecretStore
  target:
    name: linear-oauth-tokens
  data:
  - secretKey: LINEAR_APP_MORGAN_ACCESS_TOKEN
    remoteRef:
      key: secret/data/linear/morgan
      property: access_token
  - secretKey: LINEAR_APP_MORGAN_REFRESH_TOKEN
    remoteRef:
      key: secret/data/linear/morgan
      property: refresh_token
```

## Answers to Requirements

### 1. Where are tokens stored?
OpenBao/Vault as source of truth. ExternalSecrets sync to K8s Secrets. Services mount secrets as env vars.

### 2. What triggers a refresh?
CronJob runs every 5 minutes. Refreshes if token expires within 24 hours.

### 3. How do services get the new token?
ExternalSecrets syncs OpenBao → K8s Secret every 1 minute. Pod restart may be needed for env var changes (or use Reloader).

### 4. What happens if refresh fails?
- CronJob retries on failure
- Alerts via Prometheus metric `linear_oauth_refresh_failure`
- 24-hour buffer means you have time to fix manually
- If refresh token is revoked, alert for manual re-auth

### 5. How is this deployed/maintained?
- Part of GitOps in `infra/gitops/apps/linear-oauth-controller/`
- Helm chart for CronJob + ExternalSecret
- Terraform for initial OpenBao secret setup

## Local Development Story

For local dev, use a shell script that mimics the controller:

```bash
#!/bin/bash
# scripts/refresh-linear-token.sh
source .env.local

# Check expiry
EXPIRES_AT=$(op read "op://Automation/Linear Morgan OAuth/expires_at")
NOW=$(date +%s)
BUFFER=$((24 * 60 * 60))

if (( NOW + BUFFER > EXPIRES_AT )); then
    echo "Refreshing token..."
    REFRESH_TOKEN=$(op read "op://Automation/Linear Morgan OAuth/refresh_token")
    
    RESPONSE=$(curl -s -X POST https://api.linear.app/oauth/token \
        -d "grant_type=refresh_token" \
        -d "refresh_token=$REFRESH_TOKEN" \
        -d "client_id=$LINEAR_CLIENT_ID" \
        -d "client_secret=$LINEAR_CLIENT_SECRET")
    
    NEW_ACCESS=$(echo "$RESPONSE" | jq -r '.access_token')
    NEW_REFRESH=$(echo "$RESPONSE" | jq -r '.refresh_token')
    NEW_EXPIRES=$((NOW + $(echo "$RESPONSE" | jq -r '.expires_in')))
    
    op item edit "Linear Morgan OAuth" \
        "access_token=$NEW_ACCESS" \
        "refresh_token=$NEW_REFRESH" \
        "expires_at=$NEW_EXPIRES"
    
    # Update .env.local
    sed -i '' "s/LINEAR_APP_MORGAN_ACCESS_TOKEN=.*/LINEAR_APP_MORGAN_ACCESS_TOKEN=\"$NEW_ACCESS\"/" .env.local
fi
```

Add to launchd to run every hour locally.

## Pros

- Infrastructure-native approach
- Clear separation of concerns
- Works in K8s and locally
- Observability built-in
- Battle-tested patterns (ExternalSecrets, CronJob)

## Cons

- Requires OpenBao/Vault setup
- More moving parts
- Pod restart may be needed for token rotation
- Initial complexity
