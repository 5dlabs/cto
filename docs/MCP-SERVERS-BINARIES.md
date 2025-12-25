# MCP Servers Binary Status

This document tracks which MCP servers from `mcp-servers.json` require binaries and their build status.

## Summary

| Server | Transport | Binary Required | Status | Notes |
|--------|-----------|----------------|--------|-------|
| **openmemory** | HTTP | ❌ No | ✅ Ready | Remote service at `http://localhost:8765/mcp` |
| **context7** | stdio (npx) | ❌ No | ✅ Ready | Uses `npx -y @upstash/context7-mcp` |
| **github** | stdio (npx) | ❌ No | ✅ Ready | Uses `npx -y @modelcontextprotocol/server-github` |
| **shadcn** | stdio (npx) | ❌ No | ✅ Ready | Uses `npx -y shadcn@latest mcp` |
| **firecrawl** | stdio (npx) | ❌ No | ✅ Ready | Uses `npx -y firecrawl-mcp` |
| **filesystem** | stdio (npx) | ❌ No | ✅ Ready | Uses `npx -y @modelcontextprotocol/server-filesystem` |
| **memory** | stdio (npx) | ❌ No | ✅ Ready | Uses `npx -y @modelcontextprotocol/server-memory` |
| **playwright** | stdio (npx) | ❌ No | ✅ Ready | Uses `npx -y @playwright/mcp@latest` |
| **solana** | HTTP | ❌ No | ✅ Ready | Remote service at `https://mcp.solana.com/mcp` |
| **cloudflare** | HTTP | ❌ No | ✅ Ready | Remote service at `https://bindings.mcp.cloudflare.com/mcp` |
| **ai-elements** | HTTP | ❌ No | ✅ Ready | Remote service at `https://registry.ai-sdk.dev/api/mcp` |
| **pg-aiguide** | HTTP | ❌ No | ✅ Ready | Remote service at `https://mcp.tigerdata.com/docs` |
| **minimax** | stdio (npx) | ✅ Yes | ⚠️ Needs Build | Uses `npx -y @5dlabs/minimax-mcp` (local package) |

## Binary Requirements

### HTTP Transport Servers
These servers don't require binaries - they connect to remote HTTP endpoints:
- `openmemory` - Local service (must be running separately)
- `solana` - Remote service
- `cloudflare` - Remote service
- `ai-elements` - Remote service
- `pg-aiguide` - Remote service

### NPX-Based Servers
These servers use `npx` to download and run npm packages on-demand:
- `context7` - `@upstash/context7-mcp`
- `github` - `@modelcontextprotocol/server-github`
- `shadcn` - `shadcn@latest`
- `firecrawl` - `firecrawl-mcp`
- `filesystem` - `@modelcontextprotocol/server-filesystem`
- `memory` - `@modelcontextprotocol/server-memory`
- `playwright` - `@playwright/mcp@latest`

**No binaries required** - `npx` downloads these packages automatically when invoked.

### Local Package: minimax-mcp

The `minimax` server uses `@5dlabs/minimax-mcp`, which is a **local package** in this repository at `packages/minimax-mcp/`.

#### Build Requirements

1. **TypeScript compilation**: Source code must be compiled to JavaScript in `dist/` directory
2. **NPM publishing**: Package must be published to npm registry for `npx -y @5dlabs/minimax-mcp` to work

#### Build Instructions

**Local Build:**
```bash
cd packages/minimax-mcp
npm install
npm run build
```

Or use the provided script:
```bash
./scripts/build-minimax-mcp.sh
```

**Publish to npm:**
```bash
cd packages/minimax-mcp
npm publish --access public
```

#### GitHub Workflow

A GitHub Actions workflow (`.github/workflows/minimax-mcp-publish.yaml`) automatically:
- Builds the package when `packages/minimax-mcp/` changes
- Publishes to npm when version is bumped
- Uses `NPM_TOKEN` secret for authentication

#### Current Status

- ✅ Source code exists at `packages/minimax-mcp/src/`
- ✅ Build script created: `scripts/build-minimax-mcp.sh`
- ✅ GitHub workflow created: `.github/workflows/minimax-mcp-publish.yaml`
- ⚠️ **Package needs to be built and published to npm** before `npx -y @5dlabs/minimax-mcp` will work

## Verification

To verify all MCP servers are ready:

```bash
# Check if minimax-mcp is built
test -d packages/minimax-mcp/dist && echo "✅ Built" || echo "❌ Not built"

# Check if minimax-mcp is published (requires npm CLI)
npm view @5dlabs/minimax-mcp version && echo "✅ Published" || echo "❌ Not published"
```

## API Key Requirements

Several MCP servers require API keys that should be retrieved from **1Password**:

| Server | Environment Variable | 1Password Item | Field Name |
|--------|---------------------|----------------|------------|
| **github** | `GITHUB_TOKEN` | `GitHub PAT - Tools MCP Server` | `credential` |
| **firecrawl** | `FIRECRAWL_API_KEY` | `Firecrawl API Key` | `credential` |
| **minimax** | `MINIMAX_API_KEY` | `MiniMax API Key` | `credential` |
| **minimax** | `MINIMAX_GROUP_ID` | `MiniMax API Key` | `Group ID` (optional) |
| **latitude** | `LATITUDE_API_KEY` | `Latitude.sh API` | `credential` |

### Retrieving API Keys from 1Password

**Prerequisites:**
1. Install 1Password CLI: https://developer.1password.com/docs/cli/get-started
2. Sign in: `eval $(op signin)`

**Manual Retrieval:**

```bash
# GitHub PAT
op item get "GitHub PAT - Tools MCP Server" --fields credential --reveal

# Firecrawl API Key
op item get "Firecrawl API Key" --fields credential --reveal

# MiniMax API Key
op item get "MiniMax API Key" --fields credential --reveal

# MiniMax Group ID (optional)
op item get "MiniMax API Key" --fields "Group ID" --reveal

# Latitude API Key
op item get "Latitude.sh API" --fields credential --reveal
```

**Automated Sync (Local Development):**

The `sync-credentials-from-1password.sh` script automatically syncs credentials from 1Password to OpenBao (for cluster use):

```bash
./scripts/sync-credentials-from-1password.sh
```

**For Local MCP Server Testing:**

Export environment variables before running MCP servers:

```bash
# Set environment variables
export GITHUB_TOKEN=$(op item get "GitHub PAT - Tools MCP Server" --fields credential --reveal)
export FIRECRAWL_API_KEY=$(op item get "Firecrawl API Key" --fields credential --reveal)
export MINIMAX_API_KEY=$(op item get "MiniMax API Key" --fields credential --reveal)
export MINIMAX_GROUP_ID=$(op item get "MiniMax API Key" --fields "Group ID" --reveal)
export LATITUDE_API_KEY=$(op item get "Latitude.sh API" --fields credential --reveal)

# Or use op run for automatic injection
op run --env-file=.env.local -- your-mcp-command
```

**1Password Item Structure:**

Each API key should be stored in 1Password with:
- **Item Title**: Descriptive name (e.g., "GitHub PAT - Tools MCP Server")
- **Field Type**: `credential` (for API keys/tokens) or custom field labels
- **Vault**: Usually `Private` or `5DLabs` (configurable via `OP_VAULT` env var)

### Other Services with API Keys

Some services used by the platform (not MCP servers) also store credentials in 1Password:

| Service | 1Password Item | Usage |
|---------|----------------|-------|
| **Latitude.sh** | `Latitude.sh API` | Bare metal provider credentials (`api_key`, `project_id`) |
| **Cloudflare** | `CloudFlare API` | DNS and tunnel management |
| **Linear** | `Linear API Credentials` | Issue sync and project management |

## Notes

- All npx-based servers will download packages automatically on first use
- HTTP transport servers must be accessible at their configured URLs
- The minimax-mcp package is the only one requiring local build/publish steps
- API keys are synced from 1Password to OpenBao for cluster deployments
- For local development, use `op run` or export env vars manually

