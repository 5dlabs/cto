# Linear OAuth Setup for CTO Agents

## Current Status

| Agent | OAuth App | Token | Refresh |
|-------|-----------|-------|---------|
| morgan | ✅ | ✅ Valid | ⚠️ No refresh_token |
| bolt | ✅ | ❌ Expired | ⚠️ No refresh_token |
| rex | ❌ | ❌ | ❌ |
| blaze | ❌ | ❌ | ❌ |
| grizz | ❌ | ❌ | ❌ |
| nova | ❌ | ❌ | ❌ |
| tap | ❌ | ❌ | ❌ |
| spark | ❌ | ❌ | ❌ |
| cleo | ❌ | ❌ | ❌ |
| cipher | ❌ | ❌ | ❌ |
| tess | ❌ | ❌ | ❌ |
| atlas | ❌ | ❌ | ❌ |
| stitch | ❌ | ❌ | ❌ |
| vex | ❌ | ❌ | ❌ |

## Architecture

### Token Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                        LINEAR OAUTH FLOW                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  1. Create OAuth App in Linear                                  │
│     ┌─────────────────────────────────────────────────────┐     │
│     │ Linear Settings → API → OAuth Applications          │     │
│     │ • Name: 5DLabs-{Agent}                              │     │
│     │ • Redirect URI: https://pm.5dlabs.ai/oauth/{agent}  │     │
│     │ • Scopes: read, write, issues:create               │     │
│     └─────────────────────────────────────────────────────┘     │
│                            │                                     │
│                            ▼                                     │
│  2. Store credentials in K8s / 1Password                        │
│     ┌─────────────────────────────────────────────────────┐     │
│     │ linear-app-{agent} secret:                          │     │
│     │ • client_id                                         │     │
│     │ • client_secret                                     │     │
│     │ • webhook_secret                                    │     │
│     └─────────────────────────────────────────────────────┘     │
│                            │                                     │
│                            ▼                                     │
│  3. User authorizes app                                         │
│     ┌─────────────────────────────────────────────────────┐     │
│     │ GET https://pm.5dlabs.ai/oauth/{agent}/authorize    │     │
│     │     → Redirects to Linear OAuth consent             │     │
│     │     → User approves                                 │     │
│     │     → Linear redirects to callback with code        │     │
│     └─────────────────────────────────────────────────────┘     │
│                            │                                     │
│                            ▼                                     │
│  4. PM Server exchanges code for tokens                         │
│     ┌─────────────────────────────────────────────────────┐     │
│     │ POST https://api.linear.app/oauth/token             │     │
│     │     grant_type=authorization_code                   │     │
│     │     code={authorization_code}                       │     │
│     │     → Returns: access_token + refresh_token         │     │
│     └─────────────────────────────────────────────────────┘     │
│                            │                                     │
│                            ▼                                     │
│  5. Tokens stored in K8s secret                                 │
│     ┌─────────────────────────────────────────────────────┐     │
│     │ linear-app-{agent} secret updated:                  │     │
│     │ • access_token                                      │     │
│     │ • refresh_token                                     │     │
│     │ • expires_at                                        │     │
│     └─────────────────────────────────────────────────────┘     │
│                            │                                     │
│                            ▼                                     │
│  6. TokenHealthManager auto-refreshes                           │
│     ┌─────────────────────────────────────────────────────┐     │
│     │ Every 5 minutes, checks all agents:                 │     │
│     │ • If expires_at < now + 1 hour: refresh             │     │
│     │ • POST https://api.linear.app/oauth/token           │     │
│     │     grant_type=refresh_token                        │     │
│     │ • Update K8s secret with new tokens                 │     │
│     └─────────────────────────────────────────────────────┘     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Setup Instructions

### Step 1: Create OAuth Apps in Linear

For each agent, create an OAuth application in Linear:

1. Go to **Linear Settings → API → OAuth Applications**
2. Click **Create OAuth Application**
3. Fill in:
   - **Name**: `5DLabs-{Agent}` (e.g., `5DLabs-Rex`)
   - **Redirect URIs**: `https://pm.5dlabs.ai/oauth/{agent}/callback`
   - **Scopes**: `read`, `write`, `issues:create`
4. Save the **Client ID** and **Client Secret**

### Step 2: Store Credentials

#### Option A: Kubernetes (Production)

```bash
kubectl create secret generic linear-app-{agent} \
  --namespace cto \
  --from-literal=client_id={CLIENT_ID} \
  --from-literal=client_secret={CLIENT_SECRET} \
  --from-literal=webhook_secret={WEBHOOK_SECRET}
```

#### Option B: 1Password (Local Testing)

```bash
op item create \
  --category="Login" \
  --title="Linear {Agent} OAuth" \
  --vault="Automation" \
  "client_id[concealed]={CLIENT_ID}" \
  "client_secret[concealed]={CLIENT_SECRET}"
```

### Step 3: Authorize the App

1. Open the authorization URL:
   ```
   https://pm.5dlabs.ai/oauth/{agent}/authorize
   ```

2. Log in to Linear with the workspace admin account

3. Approve the OAuth request

4. PM Server will:
   - Exchange the code for tokens
   - Store `access_token` + `refresh_token` in K8s secret
   - Start automatic refresh cycle

### Step 4: Verify

```bash
# Check token health
./setup-linear-oauth.sh

# Test specific agent
./setup-linear-oauth.sh bolt
```

## Automated Refresh

The PM Server's `TokenHealthManager` automatically refreshes tokens:

- **Interval**: Every 5 minutes
- **Buffer**: Refresh 1 hour before expiration
- **Concurrency**: 3 parallel refreshes max

### How It Works

```rust
// crates/pm/src/state/token_health.rs

pub async fn refresh_expiring_tokens(&self) {
    for (agent, app) in agents {
        if app.needs_refresh() {
            let new_tokens = refresh_access_token(
                &app.refresh_token,
                &app.client_id,
                &app.client_secret,
            ).await;
            
            store_to_k8s_secret(agent, new_tokens);
        }
    }
}
```

## Troubleshooting

### Token Expired, No Refresh Token

**Symptom**: `developer_token` returns 401, but no `refresh_token` stored.

**Solution**: Re-authorize the app:
```bash
open "https://pm.5dlabs.ai/oauth/{agent}/authorize"
```

### OAuth App Not Found

**Symptom**: `no OAuth app configured`

**Solution**: Create OAuth app in Linear (Step 1)

### Refresh Fails

**Symptom**: `refresh failed: invalid_grant`

**Possible causes**:
- Refresh token revoked (user uninstalled app)
- Token rotated and old refresh token used

**Solution**: Re-authorize the app

## 1Password Items

### Existing Items

| Item | Contents |
|------|----------|
| `Linear Agent Client Secrets (Rotated 2026-01-02)` | client_secret for 12 agents (by section) |
| `Linear Bolt OAuth` | client_id, client_secret, developer_token |
| `Linear Morgan OAuth` | client_id, client_secret, developer_token |

### Required Fields Per Agent

```
Linear {Agent} OAuth:
  - client_id (concealed)
  - client_secret (concealed)
  - developer_token (concealed) - the access token
  - refresh_token (concealed) - for auto-refresh
```

## Scripts

| Script | Purpose |
|--------|---------|
| `setup-linear-oauth.sh` | Audit all agents, test tokens |
| `setup-linear-oauth.sh --refresh` | Attempt token refresh |
| `verify-linear-tokens.sh` | Quick token validation |
