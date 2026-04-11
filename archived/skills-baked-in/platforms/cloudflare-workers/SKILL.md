---
name: cloudflare-workers
description: Deploy and manage Cloudflare Workers using Wrangler CLI - KV, R2, D1, Vectorize, Queues, and more.
---

# Wrangler CLI for Cloudflare Workers

Deploy, develop, and manage Cloudflare Workers and associated resources.

## First: Verify Installation

```bash
wrangler --version  # Requires v4.x+

# If not installed:
npm install -D wrangler@latest
```

## Key Guidelines

- **Use `wrangler.jsonc`**: Prefer JSON config over TOML. Newer features are JSON-only.
- **Set `compatibility_date`**: Use a recent date (within 30 days)
- **Generate types after config changes**: Run `wrangler types` to update TypeScript bindings
- **Local dev defaults to local storage**: Bindings use local simulation unless `remote: true`
- **Use environments for staging/prod**: Define `env.staging` and `env.production` in config

## Quick Start: New Worker

```bash
# Initialize new project
npx wrangler init my-worker

# Or with a framework
npx create-cloudflare@latest my-app
```

## Quick Reference: Core Commands

| Task | Command |
|------|---------|
| Start local dev server | `wrangler dev` |
| Deploy to Cloudflare | `wrangler deploy` |
| Deploy dry run | `wrangler deploy --dry-run` |
| Generate TypeScript types | `wrangler types` |
| Validate configuration | `wrangler check` |
| View live logs | `wrangler tail` |
| Delete Worker | `wrangler delete` |
| Auth status | `wrangler whoami` |

## Configuration (wrangler.jsonc)

### Minimal Config

```jsonc
{
  "$schema": "./node_modules/wrangler/config-schema.json",
  "name": "my-worker",
  "main": "src/index.ts",
  "compatibility_date": "2026-01-01"
}
```

### Full Config with Bindings

```jsonc
{
  "$schema": "./node_modules/wrangler/config-schema.json",
  "name": "my-worker",
  "main": "src/index.ts",
  "compatibility_date": "2026-01-01",
  "compatibility_flags": ["nodejs_compat_v2"],
  
  // Environment variables
  "vars": { "ENVIRONMENT": "production" },
  
  // KV Namespace
  "kv_namespaces": [{ "binding": "KV", "id": "<KV_NAMESPACE_ID>" }],
  
  // R2 Bucket
  "r2_buckets": [{ "binding": "BUCKET", "bucket_name": "my-bucket" }],
  
  // D1 Database
  "d1_databases": [{
    "binding": "DB",
    "database_name": "my-db",
    "database_id": "<DB_ID>"
  }],
  
  // Workers AI (always remote)
  "ai": { "binding": "AI" },
  
  // Vectorize
  "vectorize": [{ "binding": "VECTOR_INDEX", "index_name": "my-index" }],
  
  // Durable Objects
  "durable_objects": {
    "bindings": [{ "name": "COUNTER", "class_name": "Counter" }]
  },
  
  // Environments
  "env": {
    "staging": {
      "name": "my-worker-staging",
      "vars": { "ENVIRONMENT": "staging" }
    }
  }
}
```

## Local Development

```bash
# Local mode (default) - uses local storage simulation
wrangler dev

# With specific environment
wrangler dev --env staging

# Remote mode - runs on Cloudflare edge
wrangler dev --remote

# Custom port
wrangler dev --port 8787

# Test scheduled/cron handlers
wrangler dev --test-scheduled
# Then visit: http://localhost:8787/__scheduled
```

### Local Secrets

Create `.dev.vars` for local development secrets:

```
API_KEY=local-dev-key
DATABASE_URL=postgres://localhost:5432/dev
```

## Deployment

```bash
# Deploy to production
wrangler deploy

# Deploy specific environment
wrangler deploy --env staging

# Dry run (validate without deploying)
wrangler deploy --dry-run

# Keep dashboard-set variables
wrangler deploy --keep-vars
```

### Manage Secrets

```bash
# Set secret interactively
wrangler secret put API_KEY

# Set from stdin
echo "secret-value" | wrangler secret put API_KEY

# List secrets
wrangler secret list

# Delete secret
wrangler secret delete API_KEY
```

## KV (Key-Value Store)

```bash
# Create namespace
wrangler kv namespace create MY_KV

# Put value
wrangler kv key put --namespace-id <ID> "key" "value"

# Put with expiration (seconds)
wrangler kv key put --namespace-id <ID> "key" "value" --expiration-ttl 3600

# Get value
wrangler kv key get --namespace-id <ID> "key"

# List keys
wrangler kv key list --namespace-id <ID>
```

## R2 (Object Storage)

```bash
# Create bucket
wrangler r2 bucket create my-bucket

# Upload object
wrangler r2 object put my-bucket/path/file.txt --file ./local-file.txt

# Download object
wrangler r2 object get my-bucket/path/file.txt

# Delete object
wrangler r2 object delete my-bucket/path/file.txt
```

## D1 (SQL Database)

```bash
# Create database
wrangler d1 create my-database

# Execute SQL command (remote)
wrangler d1 execute my-database --remote --command "SELECT * FROM users"

# Execute SQL file (remote)
wrangler d1 execute my-database --remote --file ./schema.sql

# Create migration
wrangler d1 migrations create my-database create_users_table

# Apply migrations
wrangler d1 migrations apply my-database --remote
```

## Vectorize (Vector Database)

```bash
# Create index with dimensions
wrangler vectorize create my-index --dimensions 768 --metric cosine

# Create with preset
wrangler vectorize create my-index --preset @cf/baai/bge-base-en-v1.5

# Insert vectors
wrangler vectorize insert my-index --file vectors.ndjson

# Query vectors
wrangler vectorize query my-index --vector "[0.1, 0.2, ...]" --top-k 10
```

## Observability

```bash
# Stream live logs
wrangler tail

# Filter by status
wrangler tail --status error

# Filter by search term
wrangler tail --search "error"

# JSON output
wrangler tail --format json
```

## Best Practices

1. **Version control `wrangler.jsonc`**: Source of truth for Worker config
2. **Use automatic provisioning**: Omit resource IDs for auto-creation on deploy
3. **Run `wrangler types` in CI**: Catch binding mismatches early
4. **Use environments**: Separate staging/production with `env.staging`, `env.production`
5. **Set `compatibility_date`**: Update quarterly for new runtime features
6. **Use `.dev.vars` for local secrets**: Never commit secrets to config
7. **Test locally first**: `wrangler dev` before deploying
8. **Use `--dry-run` before major deploys**: Validate changes first

## Attribution

Based on [cloudflare/skills](https://github.com/cloudflare/skills) wrangler skill.
