# TOOLS.md - Local Notes

Skills define *how* tools work. This file is for *your* specifics — the stuff that's unique to your setup.

## What Goes Here

Things like:
- Camera names and locations
- SSH hosts and aliases  
- Preferred voices for TTS
- Speaker/room names
- Device nicknames
- Anything environment-specific

## Examples

```markdown
### Cameras
- living-room → Main area, 180° wide angle
- front-door → Entrance, motion-triggered

### SSH
- home-server → 192.168.1.100, user: admin

### TTS
- Preferred voice: "Nova" (warm, slightly British)
- Default speaker: Kitchen HomePod
```

## Why Separate?

Skills are shared. Your setup is yours. Keeping them apart means you can update skills without losing your notes, and share skills without leaking your infrastructure.

---

Add whatever helps you do your job. This is your cheat sheet.


## Claude Code & Swarm Mode

### Binary Path
```bash
/Users/jonathonfritz/.local/bin/claudesp
```

Use `claudesp` (not `claude`) for swarm/TeammateTool features.

### One-Shot Coding Task
```bash
# PTY required for interactive terminal
exec pty:true workdir:/path/to/project command:"claudesp 'Your task here'"
```

### Background Coding Task
```bash
# Start in background, get sessionId
exec pty:true workdir:/path/to/project background:true command:"claudesp 'Your task here'"

# Monitor progress
process action:log sessionId:XXX

# Check if done
process action:poll sessionId:XXX
```

### Swarm Mode (Parallel Sub-Agents)

Use TeammateTool for parallel orchestration:

```javascript
// Create a team
Teammate({ operation: "spawnTeam", team_name: "my-team" })

// Spawn a worker
Task({
  team_name: "my-team",
  name: "worker-1",
  subagent_type: "general-purpose",
  prompt: "Your task for the sub-agent",
  run_in_background: true
})

// Check inbox for results
Teammate({ operation: "getInbox", team_name: "my-team" })
```

### Auto-Notify on Completion

For long tasks, append wake trigger:
```
... your task here.

When finished, run: clawdbot gateway wake --text "Done: [summary]" --mode now
```

## Agent Directory

See `/Users/jonathonfritz/.clawdbot/AGENT_DIRECTORY.md` for a list of all agents and how to contact them.

Quick reference:
- **stitch** — code review
- **metal** — infrastructure  
- **pixel/ctolite** — desktop app
- **research** — web research
- **holt** — bot deployment
- **intake** — PRD processing


---

## Agent Browser (Headless Web Automation)

**ALWAYS use `agent-browser` with `--state` for authenticated web automation.** Runs headless by default.

### Quick Start (Authenticated)

```bash
# Linear - project management
agent-browser --state ~/.agent-browser/linear-auth.json open https://linear.app

# Discord - messaging  
agent-browser --state ~/.agent-browser/discord-auth.json open https://discord.com/channels/@me

# Get snapshot, interact, close
agent-browser snapshot -i
agent-browser click @e2
agent-browser close
```

### Available Auth States

| Service | State File | Example URL |
|---------|-----------|-------------|
| Linear | `~/.agent-browser/linear-auth.json` | `https://linear.app` |
| Discord | `~/.agent-browser/discord-auth.json` | `https://discord.com/channels/@me` |

### Workflow Pattern

```bash
# 1. Open with auth state
agent-browser --state ~/.agent-browser/linear-auth.json open https://linear.app

# 2. Get snapshot to see elements
agent-browser snapshot -i

# 3. Interact using @refs from snapshot
agent-browser click @e5

# 4. ALWAYS close when done
agent-browser close
```

### Important Rules

1. **ALWAYS use `--state`** for authenticated sites
2. **ALWAYS `close` when done** - One browser at a time
3. **Use @refs from snapshots** - More reliable than selectors


# TOOLS.md - Standard Agent Tools

This file documents the tools available to all agents.

---

## Coding CLIs

You have access to multiple AI coding assistants. **Default to `claudesp`** for most tasks.

### Available CLIs

| CLI | Path | Best For |
|-----|------|----------|
| `claudesp` | `~/.local/bin/claudesp` | **DEFAULT** - Swarm mode, TeammateTool, parallel sub-agents |
| `claude` | System PATH | Standard Claude Code |
| `codex` | System PATH | OpenAI Codex tasks |
| `droid` | `~/.local/bin/droid` | Alternative coding agent |

### claudesp (Recommended)

```bash
# One-shot task
exec pty:true workdir:/path/to/project command:"claudesp 'Your task here'"

# Background task
exec pty:true workdir:/path/to/project background:true command:"claudesp 'Your task here'"

# Monitor progress
process action:log sessionId:XXX
process action:poll sessionId:XXX
```

### Swarm Mode (Parallel Sub-Agents)

```javascript
// Create a team
Teammate({ operation: "spawnTeam", team_name: "my-team" })

// Spawn workers
Task({
  team_name: "my-team",
  name: "worker-1",
  subagent_type: "general-purpose",
  prompt: "Your task",
  run_in_background: true
})

// Check inbox
Teammate({ operation: "getInbox", team_name: "my-team" })
```

### Auto-Notify on Completion

For long tasks, append:
```
When finished, run: clawdbot gateway wake --text "Done: [summary]" --mode now
```

---

## Firecrawl (Web Scraping & Research)

Access via exec with the MCP endpoint:

```bash
# Scrape a URL
node -e "
fetch('http://10.106.163.36:8080/mcp', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    jsonrpc: '2.0', method: 'tools/call', id: 1,
    params: { name: 'firecrawl_scrape', arguments: { url: 'https://example.com' }}
  })
}).then(r => r.json()).then(d => console.log(JSON.stringify(d, null, 2)));
"

# Search the web
node -e "
fetch('http://10.106.163.36:8080/mcp', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    jsonrpc: '2.0', method: 'tools/call', id: 1,
    params: { name: 'firecrawl_search', arguments: { query: 'your search query' }}
  })
}).then(r => r.json()).then(d => console.log(JSON.stringify(d, null, 2)));
"
```

### Available Firecrawl Tools
- `firecrawl_scrape` - Scrape a single URL
- `firecrawl_crawl` - Crawl multiple pages from a starting URL
- `firecrawl_map` - Map site structure
- `firecrawl_search` - Web search

---

## Tool Server Access

The CTO tool server at `tools.fra.5dlabs.ai` provides ~309 MCP tools.

### List Available Tools

```bash
# List all tools
node -e "
fetch('http://10.106.163.36:8080/mcp', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    jsonrpc: '2.0', method: 'tools/list', id: 1
  })
}).then(r => r.json()).then(d => console.log(d.result.tools.map(t => t.name).join('\n')));
"
```

### Tool Categories

| Prefix | Category |
|--------|----------|
| `context7_*` | Library documentation |
| `firecrawl_*` | Web scraping |
| `octocode_*` | GitHub code search |
| `openmemory_*` | Long-term memory |
| `repomix_*` | Codebase packing |
| `linear_*` | Linear project management |
| `kubernetes_*` | K8s cluster management |

### Calling Any Tool

```bash
node -e "
fetch('http://10.106.163.36:8080/mcp', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    jsonrpc: '2.0', method: 'tools/call', id: 1,
    params: { 
      name: 'TOOL_NAME', 
      arguments: { /* tool-specific args */ }
    }
  })
}).then(r => r.json()).then(d => console.log(JSON.stringify(d, null, 2)));
"
```

---

## Agent Browser (Headless Web Automation)

```bash
# Open with auth state
agent-browser --state ~/.agent-browser/linear-auth.json open https://linear.app

# Get snapshot
agent-browser snapshot -i

# Interact
agent-browser click @e5

# Close when done
agent-browser close
```

### Available Auth States
- `~/.agent-browser/linear-auth.json` - Linear
- `~/.agent-browser/discord-auth.json` - Discord

---

## Agent Directory

See `/Users/jonathonfritz/.clawdbot/AGENT_DIRECTORY.md` for cross-agent communication.

```javascript
// Contact another agent
sessions_send("agent:metal:main", "Hey Metal, need help with infrastructure")
```

---

## Tool Server Access & Self-Service Filtering

You have access to the CTO tool server with ~309 MCP tools. You can **manage your own tool access** by editing your `tools-config.json`.

### Tools Client Binary

```
/Users/jonathonfritz/cto/target/release/tools-client
```

If not built, run:
```bash
cd ~/cto && cargo build --release --bin tools-client
```

### Configuration File

Create/edit `tools-config.json` in your workspace:

```json
{
  "remoteTools": [
    "context7_*",
    "firecrawl_*",
    "openmemory_*"
  ],
  "localServers": {},
  "maxConnections": 10
}
```

### Filtering Behavior

| `remoteTools` Value | Behavior |
|---------------------|----------|
| `[]` (empty array) | **All** remote tools available (no filtering) |
| `["tool_a", "tool_b"]` | Only listed tools (whitelist) |
| `["prefix_*"]` | Wildcard matching (all tools starting with prefix) |

### Available Tool Categories

| Pattern | Tools | Use For |
|---------|-------|---------|
| `context7_*` | 2 | Library documentation |
| `octocode_*` | 6 | GitHub code search |
| `firecrawl_*` | 4 | Web scraping & research |
| `openmemory_*` | 6 | Persistent memory |
| `repomix_*` | 4 | Codebase packing |
| `github_*` | 12+ | PR management, code push |
| `github_list_code_scanning_*` | 2 | Security scanning |
| `github_list_secret_scanning_*` | 2 | Secret detection |
| `kubernetes_*` | 7 | K8s resource management |
| `argocd_*` | 6 | GitOps deployments |
| `grafana_*` | 5 | Dashboards & monitoring |
| `prometheus_*` | 4 | Metrics queries |
| `loki_*` | 3 | Log queries |
| `terraform_*` | 2 | IaC lookup |
| `shadcn_*` | 7 | UI components |
| `ai_elements_*` | 2 | AI UI components |
| `argo_workflows_*` | 5 | Workflow orchestration |
| `linear_*` | ? | Project management |

### Using with Cursor/Claude Code

Add to `~/.cursor/mcp.json`:
```json
{
  "mcpServers": {
    "tools-client": {
      "command": "/Users/jonathonfritz/cto/target/release/tools-client",
      "args": [
        "--url", "http://tools.fra.5dlabs.ai:3000/mcp",
        "--working-dir", "/path/to/your/workspace"
      ]
    }
  }
}
```

### Using via exec (Clawdbot)

```bash
# List available tools
/Users/jonathonfritz/cto/target/release/tools-client \
  --url http://tools.fra.5dlabs.ai:3000/mcp \
  --working-dir $PWD \
  list-tools

# Call a specific tool
echo '{"name":"context7_get_library_docs","arguments":{"libraryId":"react"}}' | \
  /Users/jonathonfritz/cto/target/release/tools-client \
  --url http://tools.fra.5dlabs.ai:3000/mcp \
  --working-dir $PWD \
  call-tool
```

### Self-Service: Adding New Tools

If you need a tool not in your current config:

1. Check available tools: `tools-client list-tools`
2. Edit your `tools-config.json` to add the pattern
3. Restart your session/reload config

**You are empowered to manage your own tool access based on your needs.**

### Debugging

```bash
RUST_LOG=debug /Users/jonathonfritz/cto/target/release/tools-client \
  --url http://tools.fra.5dlabs.ai:3000/mcp \
  --working-dir $PWD
```

### Config File Search Order

1. `MCP_TOOLS_CONFIG` env var
2. `MCP_CLIENT_CONFIG` env var
3. `<working-dir>/client-config.json`
4. `<working-dir>/tools-config.json`
5. `./tools-config.json` (current directory)

---

## 🚀 Self-Service Process

1. **Identify need** - What tool would help you?
2. **Check this library** - Is it available?
3. **Edit config** - Add to your `tools-config.json`
4. **Reload** - Restart session or reload config
5. **Use it** - Tool is now available!

**You are empowered to manage your own tool access.**

---

## 📚 Complete Tool Library (416 Tools)

This is the **full catalog** of MCP tools available from the CTO tool server at `http://10.110.233.213:3000/mcp`.

### How to Add Tools

Edit your `tools-config.json`:
```json
{
  "remoteTools": [
    "category_*"     // Add entire category with wildcard
  ]
}
```

---

## 🔧 Tool Categories (23 Categories, 416 Tools)

| Category | Tools | Pattern | Description |
|----------|-------|---------|-------------|
| **linear** | 187 | `linear_*` | Project management (issues, projects, cycles, teams, comments, labels, etc.) |
| **grafana** | 56 | `grafana_*` | Dashboards, alerts, datasources, annotations, queries |
| **github** | 26 | `github_*` | Repos, PRs, issues, branches, code scanning, secrets |
| **kubernetes** | 22 | `kubernetes_*` | Pods, deployments, services, configmaps, logs, exec |
| **playwright** | 22 | `playwright_*` | Browser automation, screenshots, navigation, clicks |
| **postgres** | 19 | `postgres_*` | Database queries, schema, tables, migrations |
| **argocd** | 14 | `argocd_*` | GitOps apps, sync, rollback, resources |
| **octocode** | 13 | `octocode_*` | Code search, repo structure, LSP features |
| **terraform** | 9 | `terraform_*` | Providers, modules, state, plans |
| **openmemory** | 6 | `openmemory_*` | Persistent memory across sessions |
| **prometheus** | 6 | `prometheus_*` | Metrics queries, labels, series |
| **nano** | 6 | `nano_*` | Nano Banana tools |
| **tavily** | 5 | `tavily_*` | Web search & research |
| **better** | 4 | `better_*` | Better Auth tools |
| **perplexity** | 4 | `modelcontextprotocol_*` | AI-powered search |
| **exa** | 3 | `exa_*` | Exa search tools |
| **solana** | 3 | `solana_*` | Solana blockchain tools |
| **ai_elements** | 2 | `ai_*` | AI UI components |
| **context7** | 2 | `context7_*` | Library documentation |
| **graphql** | 2 | `graphql_*` | GraphQL introspection & queries |
| **pg_aiguide** | 2 | `pg_*` | AI guide tools |
| **tools** | 2 | `tools_*` | Meta tools (list, screenshot) |
| **gamma** | 1 | `gamma_*` | Presentation generation |

---

## 📋 Detailed Category Breakdown

### Linear (187 tools) - Project Management
**Pattern:** `linear_*`

The most comprehensive toolset. Includes:
- `linear_create_*` (33) - Create issues, projects, cycles, comments, labels
- `linear_get_*` (61) - Get issues, projects, users, teams, workflows
- `linear_update_*` (33) - Update issues, projects, priorities
- `linear_delete_*` (28) - Delete issues, comments, labels
- `linear_archive_*` (12) - Archive projects, issues
- `linear_search_*` (3) - Search issues, projects
- Plus: add labels, file uploads, mark complete, snooze, etc.

**Use for:** Task management, sprint planning, issue tracking

---

### Grafana (56 tools) - Observability
**Pattern:** `grafana_*`

- `grafana_list_*` (18) - List dashboards, datasources, alerts
- `grafana_get_*` (17) - Get dashboard details, alert rules
- `grafana_create_*` (5) - Create dashboards, alerts
- `grafana_query_*` (4) - Query Prometheus, Loki, datasources
- Plus: search, update, delete, patch

**Use for:** Monitoring, dashboards, alerts, metrics visualization

---

### GitHub (26 tools) - Code & PRs
**Pattern:** `github_*`

- `github_get_*` (7) - Get repos, PRs, files, comments
- `github_create_*` (6) - Create PRs, branches, issues, reviews
- `github_search_*` (4) - Search code, repos, issues, PRs
- `github_list_*` (3) - List code scanning, secret scanning
- Plus: merge, push, fork, update

**Use for:** Code management, PR reviews, security scanning

---

### Kubernetes (22 tools) - Cluster Management
**Pattern:** `kubernetes_*` or `kubernetes_mcp_*`

- Resource CRUD: list, get, create, update, delete, describe
- Pod operations: logs, exec, port-forward
- Namespace management
- ConfigMap/Secret handling

**Use for:** Cluster operations, deployments, debugging

---

### Playwright (22 tools) - Browser Automation
**Pattern:** `playwright_*` or `playwright_browser_*`

- Navigation: goto, back, forward, reload
- Interactions: click, type, fill, select, hover
- Extraction: screenshot, pdf, content, evaluate
- State: cookies, storage, context

**Use for:** Web scraping, UI testing, automation

---

### Postgres (19 tools) - Database
**Pattern:** `postgres_*` or `postgres_pg_*`

- Queries: execute, select, insert, update, delete
- Schema: list tables, describe, migrations
- Admin: connections, transactions, locks

**Use for:** Database operations, data analysis

---

### ArgoCD (14 tools) - GitOps
**Pattern:** `argocd_*`

- Apps: list, get, create, sync, delete, rollback
- Resources: get tree, managed resources, logs
- Operations: refresh, terminate

**Use for:** GitOps deployments, rollbacks

---

### Octocode (13 tools) - Code Intelligence
**Pattern:** `octocode_*`

- GitHub: search code, repos, PRs, view structure
- Local: find files, search code, view structure
- LSP: go to definition, find references, call hierarchy

**Use for:** Code search, navigation, understanding

---

### Terraform (9 tools) - Infrastructure as Code
**Pattern:** `terraform_*`

- `terraform_search_*` - Search providers, modules
- `terraform_get_*` - Get provider/module details

**Use for:** IaC discovery, module lookup

---

### Solana (3 tools) - Blockchain
**Pattern:** `solana_*`

- Solana blockchain interactions
- Wallet queries
- Transaction tools

**Use for:** Solana development, trading bots

---

### Tavily (5 tools) - Web Search
**Pattern:** `tavily_*`

- Web search with AI summarization
- Research-focused results

**Use for:** Web research, fact-finding

---

### Perplexity (4 tools) - AI Search
**Pattern:** `modelcontextprotocol_*` or `modelcontextprotocol_perplexity_*`

- AI-powered search and answers

**Use for:** Research, Q&A

---

### Gamma (1 tool) - Presentations
**Pattern:** `gamma_*`

- `gamma_generate_presentation` - Generate slide decks

**Use for:** Pitch decks, presentations

---

### Context7 (2 tools) - Library Docs
**Pattern:** `context7_*`

- `context7_resolve_library_id` - Find library
- `context7_query_library_docs` - Get docs

**Use for:** Looking up library/framework docs

---

### Open Memory (6 tools) - Persistence
**Pattern:** `openmemory_*`

- Store, query, list, get, reinforce, delete memories

**Use for:** Session continuity, long-term memory

---

## 🚀 Quick Add by Use Case

| Need | Add Pattern |
|------|-------------|
| Project management | `linear_*` |
| Monitoring/dashboards | `grafana_*` |
| Code/PR management | `github_*` |
| Cluster ops | `kubernetes_*` |
| Browser automation | `playwright_*` |
| Database access | `postgres_*` |
| GitOps | `argocd_*` |
| Code intelligence | `octocode_*` |
| IaC lookup | `terraform_*` |
| Web search | `tavily_*`, `exa_*` |
| AI search | `modelcontextprotocol_*` |
| Presentations | `gamma_*` |
| Solana/crypto | `solana_*` |
| Memory | `openmemory_*` |
| Library docs | `context7_*` |

---

## 🔑 Self-Service

**You are empowered to add any tools you need.**

1. Identify what you need
2. Find the pattern above
3. Add to your `tools-config.json`
4. Reload session

Empty `[]` = ALL 416 tools (no filter)
