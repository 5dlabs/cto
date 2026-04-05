# Synthesis: Linear OAuth Token Management

> Historical analysis. Superseded by the PM-managed `client_credentials` token broker flow documented in `tests/cli-invocation/docs/LINEAR-OAUTH.md`.

## Summary of Critiques

### Universal Issues Identified

Every solution has been critiqued for at least one of these:

| Issue | Rex | Grizz | Blaze | Nova |
|-------|-----|-------|-------|------|
| Multi-replica coordination | ⚠️ CronJob race | ⚠️ PVC write race | ⚠️ Lock race | ✅ Single orchestrator |
| Real-time token propagation | ❌ Up to 5min delay | ✅ Instant | ⚠️ ConfigMap watch | ✅ Pub/sub |
| Local dev parity | ❌ Shell script | ⚠️ Must run proxy | ✅ Same code | ⚠️ Needs Redis |
| Infrastructure complexity | ❌ Vault + CronJob | ⚠️ New service | ✅ Library only | ⚠️ Redis + service |
| Service code changes | ✅ None | ✅ URL change only | ⚠️ New dependency | ⚠️ Subscriber code |

### Key Insights from Debate

1. **Everyone agrees**: CronJob polling every 5 minutes for a 10-day token is wasteful
2. **Everyone agrees**: Multi-replica refresh races are real and dangerous
3. **Everyone agrees**: Service restart for token update is unacceptable
4. **Disagreement**: Whether new infrastructure (proxy/Redis) is worth it vs library approach

## Recommended Solution: Hybrid Approach

Taking the best from each:

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                  Hybrid Token Management                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  LAYER 1: Refresh Logic (Single Point)                          │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ token-refresh-controller (K8s Deployment, 1 replica)        │ │
│  │                                                              │ │
│  │ • Checks token expiry every hour                            │ │
│  │ • Refreshes at 50% lifetime (5 days before expiry)          │ │
│  │ • Writes to K8s Secret directly                             │ │
│  │ • Emits Prometheus metrics                                  │ │
│  │ • Single replica = no coordination needed                   │ │
│  └────────────────────────────────────────────────────────────┘ │
│                           │                                      │
│                           ▼                                      │
│  LAYER 2: Token Storage (K8s Native)                            │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ K8s Secret: linear-oauth-tokens                             │ │
│  │                                                              │ │
│  │ Data:                                                        │ │
│  │   MORGAN_ACCESS_TOKEN: lin_oauth_...                        │ │
│  │   MORGAN_REFRESH_TOKEN: lin_ref_...                         │ │
│  │   MORGAN_EXPIRES_AT: 1738000000                             │ │
│  │   TOKEN_VERSION: 42                                         │ │
│  └────────────────────────────────────────────────────────────┘ │
│                           │                                      │
│                           ▼                                      │
│  LAYER 3: Token Distribution (Stakater Reloader)                │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ Reloader watches Secret, triggers rolling restart           │ │
│  │                                                              │ │
│  │ Deployments annotated:                                      │ │
│  │   reloader.stakater.com/auto: "true"                       │ │
│  │                                                              │ │
│  │ When Secret changes → Pods restart → New token loaded       │ │
│  └────────────────────────────────────────────────────────────┘ │
│                           │                                      │
│                           ▼                                      │
│  LAYER 4: Services (Zero Code Changes)                          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                      │
│  │PM Server │  │Controller│  │MCP Server│                      │
│  │          │  │          │  │          │                      │
│  │ Reads    │  │ Reads    │  │ Reads    │                      │
│  │ env var  │  │ env var  │  │ env var  │                      │
│  └──────────┘  └──────────┘  └──────────┘                      │
└─────────────────────────────────────────────────────────────────┘
```

### Why This Works

1. **Single Writer**: Controller is sole refresher → no races
2. **K8s Native**: No Redis, no Vault, no proxy
3. **Automatic Propagation**: Reloader handles pod restarts
4. **Zero Code Changes**: Services just read env vars
5. **Proactive Refresh**: 5-day buffer prevents last-minute failures

### Implementation

#### 1. Token Refresh Controller (Minimal Rust Binary)

```rust
// crates/token-refresh-controller/src/main.rs
use chrono::{Duration, Utc};
use kube::{Api, Client, api::PatchParams};
use k8s_openapi::api::core::v1::Secret;

const REFRESH_AT_PERCENT: f64 = 0.5;  // Refresh at 50% lifetime

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::init();
    
    let client = Client::try_default().await?;
    let secrets: Api<Secret> = Api::namespaced(client, "cto");
    
    // Main loop - check every hour
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
    
    loop {
        interval.tick().await;
        
        if let Err(e) = check_and_refresh(&secrets).await {
            tracing::error!("Refresh check failed: {}", e);
            metrics::counter!("token_refresh_error").increment(1);
        }
    }
}

async fn check_and_refresh(secrets: &Api<Secret>) -> Result<(), Box<dyn std::error::Error>> {
    let secret = secrets.get("linear-oauth-tokens").await?;
    let data = secret.data.unwrap_or_default();
    
    let expires_at: i64 = decode(&data, "MORGAN_EXPIRES_AT")?.parse()?;
    let expires = chrono::DateTime::from_timestamp(expires_at, 0).unwrap();
    
    // Calculate refresh threshold (50% of remaining time)
    let token_lifetime = Duration::days(10);
    let refresh_after = token_lifetime.num_seconds() as f64 * REFRESH_AT_PERCENT;
    let refresh_threshold = expires - Duration::seconds(refresh_after as i64);
    
    if Utc::now() > refresh_threshold {
        tracing::info!("Token past 50% lifetime, refreshing...");
        
        let refresh_token = decode(&data, "MORGAN_REFRESH_TOKEN")?;
        let new_tokens = refresh_linear_token(&refresh_token).await?;
        
        // Update secret
        let version: u64 = decode(&data, "TOKEN_VERSION")
            .unwrap_or("0".to_string())
            .parse()
            .unwrap_or(0) + 1;
        
        let patch = serde_json::json!({
            "stringData": {
                "MORGAN_ACCESS_TOKEN": new_tokens.access_token,
                "MORGAN_REFRESH_TOKEN": new_tokens.refresh_token,
                "MORGAN_EXPIRES_AT": new_tokens.expires_at.to_string(),
                "TOKEN_VERSION": version.to_string(),
            }
        });
        
        secrets.patch(
            "linear-oauth-tokens",
            &PatchParams::default(),
            &kube::api::Patch::Merge(&patch)
        ).await?;
        
        tracing::info!("Token refreshed, new version: {}", version);
        metrics::counter!("token_refresh_success").increment(1);
    } else {
        let hours_until_refresh = (refresh_threshold - Utc::now()).num_hours();
        tracing::info!("Token healthy, refresh in {} hours", hours_until_refresh);
    }
    
    Ok(())
}

async fn refresh_linear_token(refresh_token: &str) -> Result<TokenResponse, Box<dyn std::error::Error>> {
    let client_id = std::env::var("LINEAR_CLIENT_ID")?;
    let client_secret = std::env::var("LINEAR_CLIENT_SECRET")?;
    
    let response = reqwest::Client::new()
        .post("https://api.linear.app/oauth/token")
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", &client_id),
            ("client_secret", &client_secret),
        ])
        .send()
        .await?;
    
    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(format!("Refresh failed: {}", error).into());
    }
    
    let token_resp: LinearTokenResponse = response.json().await?;
    
    Ok(TokenResponse {
        access_token: token_resp.access_token,
        refresh_token: token_resp.refresh_token,
        expires_at: Utc::now().timestamp() + token_resp.expires_in,
    })
}
```

#### 2. Kubernetes Deployment

```yaml
# infra/gitops/apps/token-refresh-controller/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: token-refresh-controller
  namespace: cto
spec:
  replicas: 1  # IMPORTANT: Single replica only
  selector:
    matchLabels:
      app: token-refresh-controller
  template:
    metadata:
      labels:
        app: token-refresh-controller
    spec:
      serviceAccountName: token-refresh-controller
      containers:
      - name: controller
        image: ghcr.io/5dlabs/token-refresh-controller:latest
        env:
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
        resources:
          requests:
            memory: "32Mi"
            cpu: "10m"
          limits:
            memory: "64Mi"
            cpu: "100m"
---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: token-refresh-controller
  namespace: cto
---
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: token-refresh-controller
  namespace: cto
rules:
- apiGroups: [""]
  resources: ["secrets"]
  resourceNames: ["linear-oauth-tokens"]
  verbs: ["get", "patch"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: token-refresh-controller
  namespace: cto
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: Role
  name: token-refresh-controller
subjects:
- kind: ServiceAccount
  name: token-refresh-controller
  namespace: cto
```

#### 3. Service Annotations for Reloader

```yaml
# Example: PM Server deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: pm-server
  namespace: cto
  annotations:
    reloader.stakater.com/auto: "true"  # Auto-restart on secret change
spec:
  template:
    spec:
      containers:
      - name: pm-server
        env:
        - name: LINEAR_OAUTH_TOKEN
          valueFrom:
            secretKeyRef:
              name: linear-oauth-tokens
              key: MORGAN_ACCESS_TOKEN
```

#### 4. Local Development Script

```bash
#!/bin/bash
# scripts/local-token-refresh.sh
# Run this once to set up, then launchd handles the rest

set -e

# Get current state from 1Password
ACCESS_TOKEN=$(op read "op://Automation/Linear Morgan OAuth/access_token")
REFRESH_TOKEN=$(op read "op://Automation/Linear Morgan OAuth/refresh_token")
EXPIRES_AT=$(op read "op://Automation/Linear Morgan OAuth/expires_at" 2>/dev/null || echo "0")
CLIENT_ID=$(op read "op://Automation/Linear Morgan OAuth/client_id")
CLIENT_SECRET=$(op read "op://Automation/Linear Morgan OAuth/client_secret")

NOW=$(date +%s)
LIFETIME=$((10 * 24 * 60 * 60))  # 10 days
REFRESH_THRESHOLD=$(( EXPIRES_AT - (LIFETIME / 2) ))  # 50%

if (( NOW > REFRESH_THRESHOLD )); then
    echo "🔄 Refreshing token..."
    
    RESPONSE=$(curl -s -X POST https://api.linear.app/oauth/token \
        -d "grant_type=refresh_token" \
        -d "refresh_token=$REFRESH_TOKEN" \
        -d "client_id=$CLIENT_ID" \
        -d "client_secret=$CLIENT_SECRET")
    
    if echo "$RESPONSE" | jq -e '.error' > /dev/null 2>&1; then
        echo "❌ Refresh failed: $(echo "$RESPONSE" | jq -r '.error_description')"
        exit 1
    fi
    
    NEW_ACCESS=$(echo "$RESPONSE" | jq -r '.access_token')
    NEW_REFRESH=$(echo "$RESPONSE" | jq -r '.refresh_token')
    NEW_EXPIRES=$((NOW + $(echo "$RESPONSE" | jq -r '.expires_in')))
    
    # Update 1Password
    op item edit "Linear Morgan OAuth" \
        "access_token=$NEW_ACCESS" \
        "refresh_token=$NEW_REFRESH" \
        "expires_at=$NEW_EXPIRES"
    
    # Update .env.local
    sed -i '' "s/LINEAR_OAUTH_TOKEN=.*/LINEAR_OAUTH_TOKEN=\"$NEW_ACCESS\"/" .env.local
    sed -i '' "s/LINEAR_APP_MORGAN_ACCESS_TOKEN=.*/LINEAR_APP_MORGAN_ACCESS_TOKEN=\"$NEW_ACCESS\"/" .env.local
    
    echo "✅ Token refreshed, expires at $(date -r $NEW_EXPIRES)"
else
    HOURS_UNTIL_REFRESH=$(( (REFRESH_THRESHOLD - NOW) / 3600 ))
    echo "✅ Token healthy, refresh in $HOURS_UNTIL_REFRESH hours"
fi
```

#### 5. Launchd Plist for Local (Once Per Day)

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>ai.5dlabs.linear-token-refresh</string>
    <key>ProgramArguments</key>
    <array>
        <string>/Users/jonathonfritz/cto/scripts/local-token-refresh.sh</string>
    </array>
    <key>StartCalendarInterval</key>
    <dict>
        <key>Hour</key>
        <integer>9</integer>
        <key>Minute</key>
        <integer>0</integer>
    </dict>
    <key>StandardOutPath</key>
    <string>/tmp/cto-launchd/token-refresh.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/cto-launchd/token-refresh.err</string>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>
```

## What This Achieves

| Requirement | How It's Met |
|-------------|--------------|
| Zero manual intervention | Controller auto-refreshes at 50% lifetime |
| Multi-service coordination | All services read same Secret, Reloader syncs |
| Failure recovery | 5-day buffer, metrics/alerts on failure |
| Observability | Prometheus metrics, structured logging |
| Security | Tokens in K8s Secrets, minimal RBAC |

## Migration Path

1. **Week 1**: Deploy token-refresh-controller (watches but doesn't modify)
2. **Week 2**: Enable write mode, verify Secret updates work
3. **Week 3**: Add Reloader annotations to services
4. **Week 4**: Remove manual refresh from workflow, monitor

## Never Re-authorize Again

With this system:
- Token refreshes automatically at 50% lifetime (every 5 days)
- 5-day buffer before expiration means plenty of time to catch issues
- If refresh fails, you get alerts AND have 5 days to fix
- Local and K8s use same refresh logic (just different storage)

**The only way you'd need to re-authorize**: Linear revokes the refresh token (security incident, app reinstall, etc.) - which should be rare.
