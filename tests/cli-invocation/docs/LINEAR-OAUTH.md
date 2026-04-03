# Linear Tokens for CTO Agents

## Preferred Operating Model

The canonical flow is now:

1. Store `client_id` and `client_secret` in 1Password as `Linear {Agent} OAuth`.
2. Let PM mint runtime access tokens with `grant_type=client_credentials`.
3. Let PM store the minted runtime token in Kubernetes secret `linear-app-{agent}`.
4. Let PM re-mint when the token is missing, expiring, or explicitly requested.

Do **not** treat 1Password as the source of truth for runtime access tokens.

## Source Of Truth

Use per-agent 1Password items for long-lived credentials only:

```text
Linear {Agent} OAuth
  - client_id
  - client_secret
  - webhook_secret   (if you keep it there)
```

If `developer_token` or `refresh_token` still exist on older items, treat them as legacy data. PM should not depend on them for the standard path.

Runtime access tokens belong in Kubernetes only:

```text
Secret: linear-app-{agent}
  - client_id
  - client_secret
  - webhook_secret
  - access_token
  - expires_at
```

## PM Endpoints

PM is the token broker.

Single agent:

```bash
curl -X POST https://pm.5dlabs.ai/oauth/mint/bolt
```

All configured agents:

```bash
curl -X POST https://pm.5dlabs.ai/oauth/mint-all
```

These endpoints mint via `client_credentials`, persist the runtime token to `linear-app-{agent}`, and update PM's in-memory config.

## Standard Setup For A New Agent

### 1. Create the Linear OAuth app

In Linear:

1. Go to `Settings -> API -> OAuth applications`
2. Create `5DLabs-{Agent}`
3. Add redirect URI:

```text
https://pm.5dlabs.ai/oauth/{agent}/callback
```

The redirect URI is still useful for exception flows, but it is not the default operational path.

### 2. Create the 1Password item

Create:

```text
Linear {Agent} OAuth
```

with:

```text
client_id
client_secret
webhook_secret   (optional if stored elsewhere)
```

### 3. Create or update the Kubernetes secret

PM expects `linear-app-{agent}` to exist or be provisioned by the cluster secret pipeline. It may contain blank runtime token fields initially, but it must carry the agent credentials PM needs to mint from scratch.

### 4. Ask PM to mint a runtime token

```bash
./tests/cli-invocation/scripts/refresh-linear-tokens.sh bolt
```

or:

```bash
curl -X POST https://pm.5dlabs.ai/oauth/mint/bolt
```

### 5. Verify the runtime token

```bash
./tests/cli-invocation/verify-linear-tokens.sh bolt
```

This validates the current runtime token from Kubernetes, not a cached token from 1Password.

## Browser Auth

Browser auth is now an exception path only.

Use it only when:

- the app truly requires `authorization_code`
- `client_credentials` is not enabled for that app
- you are repairing a legacy refresh-token app that cannot mint non-interactively

For normal service-style agent apps, browser auth should not be the first repair step.

## Related Scripts

| Script | Purpose |
|--------|---------|
| `tests/cli-invocation/verify-linear-tokens.sh` | Validate runtime tokens from Kubernetes |
| `tests/cli-invocation/scripts/refresh-linear-tokens.sh` | Ask PM to mint runtime tokens |
| `tests/cli-invocation/setup-linear-oauth.sh` | Audit per-agent client credentials and runtime token state |
