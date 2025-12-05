# Tools Configuration Analysis

> **Purpose**: Analyze the current Toolman/Tools setup for CTO platform and plan migration to a unified `tools-config.json` approach.

---

## Executive Summary

The current tools architecture involves multiple configuration files and a two-tier system (server-side and client-side). This document analyzes the existing setup to inform a unified configuration approach.

**Current State**: 
- Server config (`servers-config.json`) defines available MCP servers
- Client config (`client-config.json`) filters which tools agents can access
- Tools are auto-discovered from MCP servers via handshake

**Proposed State**:
- Single `tools-config.json` defining servers
- Tools auto-detected from servers
- Filtering happens at CTO controller level (agent templates)

---

## Current Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           TOOLS ARCHITECTURE                                 │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────┐     ┌─────────────────────────┐
│   Helm Chart values.yaml│     │   ConfigMap Generation  │
│                         │     │                         │
│  config:                │────►│  servers-config.json    │
│    servers:             │     │  local-tools-config.json│
│      - brave-search     │     │                         │
│      - context7         │     └──────────┬──────────────┘
│      - openmemory       │                │
│      - kubernetes       │                ▼
│      - ...              │     ┌─────────────────────────┐
│                         │     │   Tools Server (Rust)   │
│  localTools:            │     │   (Kubernetes Pod)      │
│    servers:             │     │                         │
│      - rust-tools       │     │  - Loads servers-config │
│                         │     │  - Spawns MCP servers   │
└─────────────────────────┘     │  - Auto-discovers tools │
                                │  - HTTP proxy endpoint  │
                                └──────────┬──────────────┘
                                           │
                                           │ HTTP /mcp
                                           ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        AGENT CONTAINER (Runtime)                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────┐     ┌─────────────────────────┐                    │
│  │ client-config.json  │     │   MCP Client (tools)    │                    │
│  │ (from ConfigMap)    │────►│   (Rust binary)         │                    │
│  │                     │     │                         │                    │
│  │ {                   │     │  - Loads client-config  │                    │
│  │   "remoteTools": [  │     │  - Filters remote tools │                    │
│  │     "brave_search.."│     │  - Spawns local servers │                    │
│  │   ],                │     │  - Routes tool calls    │                    │
│  │   "localServers": {}│     │                         │                    │
│  │ }                   │     └──────────┬──────────────┘                    │
│  └─────────────────────┘                │                                   │
│                                         │ stdio (MCP protocol)              │
│                                         ▼                                   │
│                              ┌─────────────────────────┐                    │
│                              │   AI Agent (Claude,     │                    │
│                              │   Cursor, Codex, etc.)  │                    │
│                              └─────────────────────────┘                    │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Configuration Files

### 1. Server Configuration (`servers-config.json`)

**Source**: Generated from `values.yaml` → `configmap.yaml` → JSON

**Location**: `/config/servers-config.json` (mounted from ConfigMap)

**Purpose**: Defines all MCP servers available on the Tools proxy

**Structure**:
```json
{
  "servers": {
    "brave-search": {
      "name": "Brave Search",
      "description": "Web search using Brave Search API",
      "transport": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-brave-search"],
      "workingDirectory": "/tmp",
      "env": {}
    },
    "openmemory": {
      "transport": "http",
      "url": "http://openmemory.cto.svc.cluster.local:8080/mcp"
    },
    "context7": {
      "transport": "stdio",
      "command": "npx",
      "args": ["-y", "@upstash/context7-mcp"]
    }
  }
}
```

**Current Servers Defined** (from `values.yaml`):

| Server | Transport | Type | Notes |
|--------|-----------|------|-------|
| `brave-search` | stdio | Search | Brave Search API |
| `openmemory` | http | Memory | Long-term memory for agents |
| `context7` | stdio | Docs | Library documentation |
| `terraform` | stdio/docker | Infra | Terraform Registry API |
| `kubernetes` | sse | Infra | K8s management (separate deployment) |
| `postgres` | stdio | Database | PostgreSQL MCP |
| `redis` | stdio | Database | Redis operations |
| `github` | stdio | Dev Tools | GitHub API |
| `shadcn` | stdio | UI | shadcn/ui components |
| `ai-elements` | http | UI | AI SDK components |
| `solana` | http | Blockchain | Solana tools |
| `reddit` | stdio | Social | Reddit content |

### 2. Client Configuration (`client-config.json`)

**Source**: Generated from Handlebars templates at agent creation time

**Location**: `/task-files/client-config.json` or `/workspace/client-config.json`

**Purpose**: Filters which tools an agent can access (whitelist)

**Structure**:
```json
{
  "remoteTools": [
    "brave_search_brave_web_search",
    "context7_get_library_docs",
    "openmemory_query",
    "openmemory_store"
  ],
  "localServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"],
      "tools": ["read_file", "write_file", "list_directory"],
      "workingDirectory": "project_root"
    }
  }
}
```

**Key Points**:
- `remoteTools`: Array of tool names to include from the Tools proxy
  - Empty array = include ALL tools (no filtering)
  - Non-empty = whitelist mode
- `localServers`: MCP servers spawned directly in the agent container
- Tool names use underscores (Cursor sanitization: `context7_get_library_docs` not `context7_get-library-docs`)

### 3. Local Tools Configuration (`local-tools-config.json`)

**Source**: Generated from `localTools` section in `values.yaml`

**Purpose**: Defines tools that run in agent containers (not proxied)

**Current Configuration**:
```yaml
localTools:
  servers:
    rust-tools:
      name: "Rust Development Tools"
      description: "Rust analyzer integration..."
      transport: "stdio"
      command: "/home/node/.cargo/bin/cursor-rust-tools"
      args: ["--no-ui"]
      workingDirectory: "/workspace"
```

---

## Tool Discovery Flow

```
┌────────────────────────────────────────────────────────────────────────────┐
│                          TOOL DISCOVERY FLOW                                │
└────────────────────────────────────────────────────────────────────────────┘

1. SERVER STARTUP (Tools Pod)
   ┌────────────────────────────────────────────────────────────────────────┐
   │  a. Load servers-config.json                                           │
   │  b. For each server definition:                                        │
   │     - Spawn process (stdio) or connect (http/sse)                      │
   │     - Send MCP "initialize" request                                    │
   │     - Send "notifications/initialized"                                 │
   │     - Send "tools/list" request                                        │
   │     - Cache discovered tools with schemas                              │
   │  c. Publish tools to ConfigMap (tools-tool-catalog)                    │
   └────────────────────────────────────────────────────────────────────────┘

2. CLIENT STARTUP (Agent Container)
   ┌────────────────────────────────────────────────────────────────────────┐
   │  a. Load client-config.json                                            │
   │  b. Spawn local servers (if any)                                       │
   │  c. Perform MCP handshake with local servers                           │
   │  d. Cache local tools                                                  │
   │  e. Connect to Tools proxy (HTTP)                                      │
   │  f. Request "tools/list" from proxy                                    │
   │  g. Filter tools based on remoteTools whitelist                        │
   │  h. Merge local + filtered remote tools                                │
   │  i. Apply Cursor compatibility fixes (name sanitization)               │
   └────────────────────────────────────────────────────────────────────────┘

3. TOOL CALL ROUTING
   ┌────────────────────────────────────────────────────────────────────────┐
   │  Agent calls tool "context7_get_library_docs"                          │
   │                                                                        │
   │  Client receives via stdio:                                            │
   │  {"method": "tools/call", "params": {"name": "context7_get..."}}       │
   │                                                                        │
   │  Client routes:                                                        │
   │  ├─ Is local server tool? → Forward to local server                    │
   │  └─ Is remote tool? → Forward to Tools proxy via HTTP                  │
   │                                                                        │
   │  Tools proxy routes:                                                   │
   │  ├─ Parse prefixed name: "context7_get_library_docs"                   │
   │  ├─ Extract server: "context7"                                         │
   │  ├─ Restore original name: "get-library-docs" (from cache)             │
   │  └─ Forward to correct MCP server                                      │
   └────────────────────────────────────────────────────────────────────────┘
```

---

## Agent Template Integration

### How Agents Get Tools Config

The controller generates agent configurations using Handlebars templates:

**Template Files**:
- `agent-templates/code/client-config.json.hbs` - Code agents
- `agent-templates/docs/claude/client-config.json.hbs` - Docs agents

**Template Variables**:
- `remote_tools` - Array of tool names to include
- `local_tools` - Array of local tool names
- `tool_config` - Config profile: "minimal", "default", "advanced"

**Default Tool Profiles**:

| Profile | Remote Tools | Local Servers |
|---------|--------------|---------------|
| `minimal` | `[]` (empty) | None |
| `default` | `brave_search`, `openmemory_query/store` | None |
| `advanced` | + `github`, `kubernetes`, `terraform` | `filesystem` |

### Environment Variable Flow

```bash
# Set in container
export MCP_CLIENT_CONFIG="/task-files/client-config.json"

# Client loads from this path
fn load_client_config() {
    let path = env::var("MCP_CLIENT_CONFIG")
        .unwrap_or("client-config.json");
    // ...
}
```

---

## Tool Name Handling

### Sanitization (Cursor Compatibility)

MCP tool names often contain hyphens, but Cursor requires underscores:

```
Original Server Name → Sanitized Client Name
─────────────────────────────────────────────
context7_get-library-docs → context7_get_library_docs
brave-search_web_search   → brave_search_brave_web_search
```

**Client-side** (`client.rs`):
```rust
fn apply_cursor_compatibility(&self, tools: Vec<Value>) -> Vec<Value> {
    tools.map(|tool| {
        let name = tool["name"].as_str();
        if name.contains('-') {
            tool["name"] = name.replace('-', "_");
        }
        tool
    })
}
```

**Server-side** (`http_server.rs`):
```rust
// Reverse lookup to find original name
fn parse_tool_name_with_servers(
    tool_name: &str,  // "context7_get_library_docs" (sanitized)
    available_tools: &HashMap<String, Tool>,
) -> ParsedTool {
    // Returns original_tool_name: "get-library-docs"
}
```

---

## Current Configuration Locations

| File | Source | Location at Runtime | Purpose |
|------|--------|---------------------|---------|
| `values.yaml` | Helm chart | N/A (build time) | Define servers |
| `servers-config.json` | ConfigMap | `/config/servers-config.json` | Server definitions |
| `local-tools-config.json` | ConfigMap | `/config/local-tools-config.json` | Local server defs |
| `client-config.json` | Agent ConfigMap | `/task-files/client-config.json` | Tool filtering |

---

## Proposed Simplification

### Goal
Single `tools-config.json` in CTO repo that:
1. Defines available servers (what's in `values.yaml.config.servers`)
2. Auto-discovers tools (existing behavior)
3. Tool filtering happens at agent template level (existing `remoteTools` array)

### Proposed Structure

```json
{
  "$schema": "./tools-config.schema.json",
  "version": "1.0",
  
  "servers": {
    "brave-search": {
      "name": "Brave Search",
      "description": "Web search using Brave Search API",
      "transport": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-brave-search"],
      "workingDirectory": "/tmp",
      "secrets": {
        "envFrom": "tools-brave-search-secrets",
        "keys": ["BRAVE_API_KEY"]
      }
    },
    "openmemory": {
      "name": "OpenMemory",
      "description": "Long-term memory system",
      "transport": "http",
      "url": "http://openmemory.cto.svc.cluster.local:8080/mcp"
    },
    "context7": {
      "name": "Context7",
      "description": "Library documentation",
      "transport": "stdio",
      "command": "npx",
      "args": ["-y", "@upstash/context7-mcp"]
    }
  },
  
  "localServers": {
    "rust-tools": {
      "name": "Rust Development Tools",
      "description": "Rust analyzer integration",
      "command": "/home/node/.cargo/bin/cursor-rust-tools",
      "args": ["--no-ui"],
      "workingDirectory": "/workspace"
    }
  }
}
```

### What Changes

| Aspect | Current | Proposed |
|--------|---------|----------|
| Server definition | `values.yaml` + Helm templating | `tools-config.json` |
| Tools discovery | Auto (via MCP handshake) | Same (no change) |
| Tool filtering | `client-config.json` per agent | Agent templates (same mechanism) |
| Local servers | Separate `localTools` section | Part of unified config |

### What Stays the Same

- **Tool auto-discovery**: MCP handshake discovers tools from servers
- **Agent-level filtering**: `remoteTools` array in client-config.json
- **Name sanitization**: Cursor compatibility layer
- **HTTP proxy architecture**: Tools pod proxies to MCP servers

---

## Migration Path

1. **Create `tools-config.json`** in CTO repo root
2. **Update Helm chart** to read from JSON instead of values.yaml
3. **Consolidate** `config.servers` and `localTools.servers`
4. **Retain agent templates** for per-agent tool filtering
5. **Update documentation**

---

## Questions to Resolve

1. **Secret management**: How to handle `secretRef` in JSON vs Helm values?
2. **Environment-specific servers**: Different URLs for dev/staging/prod?
3. **Local servers location**: Keep in unified config or separate for agent-specific needs?
4. **Validation**: JSON schema for tools-config.json?

---

## Current Issues (Post-Migration)

### Issue 1: Image Mismatch - `toolman` vs `tools`

**Symptom**: 
```
⚠️ Local tools ConfigMap not found: ApiError: configmaps "toolman-local-tools" not found
```

**Root Cause**:
The deployed image is the **old standalone toolman** image, not the new integrated tools image:

| Component | Expected | Actual (Deployed) |
|-----------|----------|-------------------|
| Image | `ghcr.io/5dlabs/tools:latest` | `ghcr.io/5dlabs/toolman:latest` |
| ConfigMap prefix | `tools-` | Old code expects `toolman-` |

**Evidence**:
```bash
# Deployed image (wrong)
$ kubectl get deployment cto-tools -n cto -o jsonpath='{.spec.template.spec.containers[0].image}'
ghcr.io/5dlabs/toolman:latest

# Helm chart expects (values.yaml)
image:
  repository: ghcr.io/5dlabs/tools
```

**ConfigMaps in cluster**:
```
cto-tools-config        1      25h   ← Main config (correct name)
tools-local-tools       1      25h   ← Local tools (Helm creates this)
toolman-tool-catalog    1      12h   ← Created by OLD toolman code
```

The old `toolman` code hardcodes `toolman-` prefix for ConfigMap lookups, but the Helm chart creates ConfigMaps with `tools-` prefix.

### Issue 2: Brave Search Server Connection Failure

**Symptom**:
```
⚠️ [brave-search] Failed to initialize server: Server connection closed
```

**Root Cause**: 
The Kubernetes secret `tools-brave-search-secrets` does not exist.

**Evidence**:
```bash
$ kubectl get secrets -n cto | grep brave
# No results

$ kubectl describe deployment cto-tools -n cto | grep BRAVE
# Environment variable is empty
```

**Expected Secret** (from `values.yaml`):
```yaml
brave-search:
  secretRef:
    name: "tools-brave-search-secrets"
    keys:
      - "BRAVE_API_KEY"
```

The secret should be created via Vault Secrets Operator but hasn't been configured/synced.

### Issue 3: Separate Local/Remote ConfigMaps

**Current State**: Two separate ConfigMaps for tools configuration:

1. `cto-tools-config` → Contains `servers-config.json` (remote servers)
2. `tools-local-tools` → Contains `local-tools-config.json` (local servers)

**Problem**: 
- Adds complexity
- Rust code has hardcoded ConfigMap name `"tools-local-tools"` (line 1177 in `http_server.rs`)
- Name doesn't follow Helm release naming convention

**Desired State**: Single unified ConfigMap containing both remote and local server definitions.

---

## Current Deployed State

### Kubernetes Resources

```
Namespace: cto

Deployments:
  - cto-tools              (main tools proxy - WRONG IMAGE)
  - cto-tools-k8s-mcp      (kubernetes MCP server)

ConfigMaps:
  - cto-tools-config       (servers-config.json)
  - tools-local-tools      (local-tools-config.json) 
  - toolman-tool-catalog   (auto-generated tool catalog - OLD NAME)

Secrets (Missing):
  - tools-brave-search-secrets  ❌ NOT FOUND
  - tools-reddit-secrets        (status unknown)
  - tools-context7-secrets      (status unknown)
  - tools-github-secrets        (status unknown)
```

### Code vs Deployment Mismatch

| Aspect | CTO Repo Code | Deployed |
|--------|---------------|----------|
| Image name | `ghcr.io/5dlabs/tools` | `ghcr.io/5dlabs/toolman` |
| Local tools ConfigMap | `"tools-local-tools"` | Old code looks for `"toolman-local-tools"` |
| Tool catalog ConfigMap | `"tools-tool-catalog"` | Old code creates `"toolman-tool-catalog"` |

---

## What Needs to Change

### 1. Unified Tools Configuration

**Goal**: Single `tools-config.json` that includes both remote and local servers.

**Current** (Two separate sections in `values.yaml`):
```yaml
# Remote servers
config:
  servers:
    brave-search: { ... }
    context7: { ... }

# Local servers (separate section)
localTools:
  servers:
    rust-tools: { ... }
```

**Proposed** (Single unified config):
```json
{
  "servers": {
    "brave-search": { "transport": "stdio", ... },
    "context7": { "transport": "stdio", ... },
    "rust-tools": { "transport": "stdio", "local": true, ... }
  }
}
```

### 2. Single ConfigMap

**Current**: 
- `configmap.yaml` → `servers-config.json`
- `local-tools-configmap.yaml` → `local-tools-config.json`

**Proposed**:
- Single `configmap.yaml` → `tools-config.json` containing everything

### 3. Fix Image Reference

**Root Cause Found**: ArgoCD application override

The `values.yaml` has the correct image:
```yaml
image:
  repository: ghcr.io/5dlabs/tools
```

But **ArgoCD is overriding it** in `infra/gitops/applications/cto/tools.yaml`:
```yaml
helm:
  values: |
    # Image configuration - temporarily using toolman image until tools release workflow runs
    image:
      repository: ghcr.io/5dlabs/toolman
      tag: "latest"
```

**Action Required**:
- Update ArgoCD application to use `ghcr.io/5dlabs/tools` image
- Verify the tools CI/CD pipeline is building and pushing images
- Remove the "temporary" override once confirmed

### 4. Create Missing Secrets

Secrets that need to be created in Vault:
- `tools-brave-search-secrets` with `BRAVE_API_KEY`
- Verify other secrets exist: `tools-context7-secrets`, `tools-github-secrets`, etc.

### 5. Update Rust Code for Unified Config

**Current** (`http_server.rs` line 1177):
```rust
match api.get("tools-local-tools").await {
```

**Proposed**: Read from unified config or make ConfigMap name configurable via environment variable.

---

## Migration Checklist

### Phase 1: Fix Immediate Issues
- [ ] Update `infra/gitops/applications/cto/tools.yaml` to remove `toolman` image override
- [ ] Verify `ghcr.io/5dlabs/tools` image exists and is being built by CI
- [ ] Create `tools-brave-search-secrets` in Vault with `BRAVE_API_KEY`
- [ ] Trigger ArgoCD sync to deploy correct image
- [ ] Verify tools pod starts with correct image

### Phase 2: Consolidate Configuration
- [ ] Create unified `tools-config.json` schema
- [ ] Merge `config.servers` and `localTools.servers` in values.yaml
- [ ] Update `configmap.yaml` to generate single config
- [ ] Delete `local-tools-configmap.yaml`
- [ ] Update Rust code to read unified config

### Phase 3: Update Rust Code
- [ ] Remove hardcoded ConfigMap name
- [ ] Add environment variable for ConfigMap name (or use release name)
- [ ] Update tool catalog generation to use correct names

### Phase 4: Cleanup
- [ ] Remove old `toolman-tool-catalog` ConfigMap
- [ ] Update documentation
- [ ] Test full tool discovery flow

---

## References

### Rust Code
- `tools/src/config.rs` - Rust config structures
- `tools/src/client.rs` - Client-side tool filtering
- `tools/src/server/http_server.rs` - Server-side tool discovery (lines 1169-1212 for local tools)

### Helm Chart
- `infra/charts/tools/values.yaml` - Current server definitions
- `infra/charts/tools/templates/configmap.yaml` - Main ConfigMap generation
- `infra/charts/tools/templates/local-tools-configmap.yaml` - Local tools ConfigMap (to be removed)
- `infra/charts/tools/templates/role.yaml` - RBAC (has both `tools-` and `toolman-` ConfigMap names)

### GitOps / ArgoCD
- `infra/gitops/applications/cto/tools.yaml` - **ArgoCD app with `toolman` image override** ⚠️

### Agent Templates
- `infra/charts/controller/agent-templates/code/client-config.json.hbs` - Agent tool filtering

### Docker Image
- `infra/images/tools/` - Dockerfile for tools image
