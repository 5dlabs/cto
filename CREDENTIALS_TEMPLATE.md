# CTO Platform - Local Credentials Reference

> **⚠️ SECURITY**: This file contains sensitive credentials. Do NOT commit to version control.
> Add to `.gitignore` if not already ignored.

## Location
Store at: `~/.config/cto/credentials.md` or `/Users/jonathonfritz/agents/conductor/local/CREDENTIALS.md`

---

## GitHub

### GitHub App (Morgan - Intake)
```env
GITHUB_APP_ID=your_app_id
GITHUB_PRIVATE_KEY="-----BEGIN RSA PRIVATE KEY-----\n...\n-----END RSA PRIVATE KEY-----"
GITHUB_WEBHOOK_SECRET=your_webhook_secret
```

### GitHub App (Rex - Implementation)
```env
GITHUB_APP_ID_REX=your_rex_app_id
GITHUB_PRIVATE_KEY_REX="-----BEGIN RSA PRIVATE KEY-----\n...\n-----END RSA PRIVATE KEY-----"
```

---

## Linear OAuth Apps

### Morgan (Intake Agent)
```env
LINEAR_APP_MORGAN_CLIENT_ID=your_client_id
LINEAR_APP_MORGAN_CLIENT_SECRET=your_client_secret
LINEAR_APP_MORGAN_WEBHOOK_SECRET=your_webhook_secret
LINEAR_REDIRECT_URI=http://localhost:8080/oauth/callback
```

### Rex (Implementation Agent)
```env
LINEAR_APP_REX_CLIENT_ID=your_client_id
LINEAR_APP_REX_CLIENT_SECRET=your_client_secret
LINEAR_APP_REX_WEBHOOK_SECRET=your_webhook_secret
```

### Other Agents (Blaze, Grizz, Nova, etc.)
```env
LINEAR_APP_BLAZE_CLIENT_ID=...
LINEAR_APP_GRIZZ_CLIENT_ID=...
LINEAR_APP_NOVA_CLIENT_ID=...
LINEAR_APP_CLEO_CLIENT_ID=...
LINEAR_APP_CIPHER_CLIENT_ID=...
LINEAR_APP_TESS_CLIENT_ID=...
LINEAR_APP_BOLT_CLIENT_ID=...
```

---

## API Keys

### Anthropic
```env
ANTHROPIC_API_KEY=your_anthropic_key
```

### OpenAI
```env
OPENAI_API_KEY=your_openai_key
```

---

## Cloudflare (Tunnels)
```env
CLOUDFLARE_API_TOKEN=your_api_token
CLOUDFLARE_ZONE_ID=your_zone_id
CLOUDFLARE_ACCOUNT_ID=your_account_id
```

---

## Database

### PostgreSQL (Development)
```env
DATABASE_URL=postgresql://user:password@localhost:5432/cto
```

---

## Kubernetes

### Kind Cluster
```env
KIND_CLUSTER_NAME=cto-lite
KUBECONFIG=~/.kube/config
```

---

## 1Password CLI (Alternative)

If you prefer using 1Password, use these commands:

```bash
# Get a specific credential
op item get "GitHub App Private Key" --fields private_key

# List items in a vault
op vault list --vault Personal

# Get environment variable from 1Password
eval $(op signin)
op run --env-file=.env -- your_command_here
```

---

## Quick Reference: Loading Credentials

### Option 1: Source this file (if using shell exports)
```bash
source ~/.config/cto/credentials.md
```

### Option 2: Load into environment with direnv
Create `~/.config/cto/.envrc`:
```bash
source_env_if_exists ~/.config/cto/credentials.md
```

Then install direnv: `brew install direnv`

### Option 3: Use with cargo dotenvy
The pm-lite crate already supports `dotenvy` - create `.env` in the project root.

---

## Adding New Credentials

When you need to add a new credential:

1. Add to this document in the appropriate section
2. Also add the `.env` variable name so tools can pick it up automatically
3. Update any tool configurations that need these values

---

## Notes

- All `LINEAR_APP_*` variables support automatic OAuth token refresh
- Tokens are stored in `~/.config/cto-lite/pm-lite.json` after OAuth flow
- GitHub App private keys should be in PEM format (not JSON)
- Cloudflare tokens should have "Zone:Read" and "Workers:Edit" permissions
