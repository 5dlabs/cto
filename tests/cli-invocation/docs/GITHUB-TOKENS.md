# GitHub App OAuth Tokens

## Overview

Each CTO agent (except `vex`) has a dedicated GitHub App for authentication. The apps use private keys stored in 1Password to generate installation tokens.

## Token Types

### 1. Private Key (PEM)
- **Stored in:** 1Password `GitHub-App-{Agent}` items
- **Field:** `private-key`
- **Purpose:** Used to create JWT for GitHub App authentication
- **Lifetime:** Does not expire (unless rotated)

### 2. Installation Token
- **Generated:** From private key + app ID
- **Lifetime:** 1 hour
- **Usage:** Actual API calls to GitHub

### 3. User Access Token (OAuth)
- **Stored in:** 1Password `credential` field (if available)
- **Purpose:** User-context API access
- **Lifetime:** Varies (usually 8 hours, with refresh token)

## Token Generation Flow

```
┌─────────────────┐
│ Private Key     │  1. Read from 1Password
│ (in 1Password)  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Create JWT      │  2. Sign JWT with app ID + private key
│ (10 min TTL)    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Get Installation│  3. POST /app/installations/:id/access_tokens
│ Token           │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Use Token       │  4. Include in Authorization header
│ (1 hour TTL)    │
└─────────────────┘
```

## 1Password Structure

Each GitHub App item contains:

| Field | Description |
|-------|-------------|
| `app-id` | GitHub App ID |
| `client-id` | OAuth client ID |
| `private-key` | PEM-encoded private key |
| `credential` | User access token (if OAuth flow completed) |
| `filename` | Usually `private-key.pem` |
| `valid from` | When key was created |
| `expires` | Token expiration (if applicable) |
| `hostname` | Usually `github.com` |

## Agents and Their Apps

| Agent | 1Password Item | Status |
|-------|---------------|--------|
| atlas | GitHub-App-Atlas | ✓ Has private-key |
| blaze | GitHub-App-Blaze | ✓ Has private-key |
| bolt | GitHub-App-Bolt | ✓ Has private-key |
| cipher | GitHub-App-Cipher | ✓ Has private-key |
| cleo | GitHub-App-Cleo | ✓ Has private-key |
| grizz | GitHub-App-Grizz | ✓ Has private-key |
| morgan | GitHub-App-Morgan | ✓ Has private-key |
| nova | GitHub-App-Nova | ✓ Has private-key |
| rex | GitHub-App-Rex | ✓ Has private-key |
| spark | GitHub-App-Spark | ✓ Has private-key |
| stitch | GitHub-App-Stitch | ✓ Has private-key |
| tap | GitHub-App-Tap | ✓ Has private-key |
| tess | GitHub-App-Tess | ✓ Has private-key |
| vex | (none) | ⚠️ Needs GitHub App creation |

## Token Refresh Process

### For Installation Tokens (Automatic)

The controller automatically refreshes installation tokens:

1. Reads private key from Kubernetes secret (sourced from 1Password)
2. Creates JWT signed with private key
3. Exchanges JWT for installation token
4. Token is valid for 1 hour
5. Refresh ~5 minutes before expiry

### For User Access Tokens (Manual)

If using OAuth user tokens:

1. User initiates OAuth flow via CTO web app
2. Token is stored in 1Password `credential` field
3. Refresh token used to get new access token when needed
4. Manual re-authentication required if refresh token expires

## Verification

Run the verification script:

```bash
cd tests/cli-invocation
./verify-github-tokens.sh           # All agents
./verify-github-tokens.sh bolt      # Single agent
```

## Creating a New GitHub App (for vex)

1. Go to: https://github.com/organizations/5dlabs/settings/apps/new
2. Name: `cto-vex` (or similar)
3. Homepage URL: `https://cto.5dlabs.io`
4. Webhook: Disable (or configure later)
5. Permissions: Contents (read/write), Pull requests (read/write), etc.
6. Generate private key
7. Store in 1Password:
   - Vault: `Automation`
   - Item name: `GitHub-App-Vex`
   - Add fields: `app-id`, `client-id`, `private-key`

## Troubleshooting

### Token Expired
- Installation tokens: Controller auto-refreshes
- User tokens: Re-run OAuth flow

### Private Key Not Found
- Check 1Password item has `private-key` field
- Verify service account has access to Automation vault

### API Returns 401
- Token may be expired
- Check GitHub App is still installed on target repos

### API Returns 403
- App may not have required permissions
- Check app installation permissions in GitHub settings
