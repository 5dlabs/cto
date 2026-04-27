# Tools Client Skill

How the CTO tool server and MCP tool architecture works in this environment.

## Architecture

```
OpenClaw Gateway
  ├── Built-in tools (exec, web_search, web_fetch, browser, etc.)
  ├── Remote tools (CTO tool server via remoteTools globs)
  └── Local MCP servers (spawned alongside CLI sessions)

CLI Backends (Claude Code, Codex, Droid)
  ├── Their own built-in tools
  ├── MCP servers (Claude Code: .claude.json, others: tools-config.json)
  └── Remote tools from tools-config.json
```

## tools-config.json

Located at `/workspace/tools-config.json`. Provides MCP tools to CLI backends.

```json
{
  "remoteTools": ["firecrawl_*"],
  "localServers": {
    "memory": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-memory"],
      "tools": ["create_entities", "search_nodes", ...]
    }
  }
}
```

### remoteTools

Glob patterns matching tools forwarded from the CTO tool server. The tool server runs as a sidecar/service and exposes tools via MCP protocol. Patterns like `"firecrawl_*"` match all Firecrawl tools.

### localServers

MCP servers spawned locally alongside CLI sessions. Each entry defines:
- `command` + `args`: How to start the server
- `tools`: Whitelist of tool names to expose (empty = all)

## Adding New Tools

1. **Remote tool from CTO server:** Add a glob pattern to `remoteTools` in values.yaml under `toolsConfig`
2. **New local MCP server:** Add an entry to `localServers` with the npx command and tool whitelist
3. **Claude Code MCP server:** Add to `claudeCode.mcp.servers` in values.yaml (renders into `.claude.json`)

## Headless OAuth/API-backed tools

- Hugging Face uses `HF_TOKEN`/`HUGGING_FACE_HUB_TOKEN` from the shared API-key secret (`huggingface-token`) and connects to `https://huggingface.co/mcp` through `mcp-remote`.
- Scenario uses project API credentials from the shared API-key secret (`scenario-api-key` and `scenario-api-secret`) and connects to `https://mcp.scenario.com/mcp` with a Basic auth header generated at runtime.
- These local MCP bridges are spawned only when a CLI session requests their tools, so missing optional credentials do not block pod startup.

## Available Tool Groups

The OpenClaw gateway organizes tools into groups:
- `group:runtime` — exec, shell, process management
- `group:fs` — file read/write/search
- `group:sessions` — session management (in-process only)
- `group:memory` — memory read/write/search
- `group:web` — web_search, web_fetch, browser
- `group:ui` — Control UI interactions

The `tools.profile: "full"` setting enables all groups with no restrictions.

## Plugin Tools

Plugin-registered tools are separate from tool groups and must be explicitly added to `tools.allow`:

- `nats` — Cross-pod inter-agent messaging via NATS. Supports publish, request, and discover actions. This is the primary mechanism for agent-to-agent communication since each agent runs in its own pod.
