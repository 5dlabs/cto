# Toolman MCP Proxy Architecture

**Date:** 2025-10-31  
**Status:** Production  
**Purpose:** Centralized MCP tool distribution for all agents

---

## Overview

The **Toolman MCP Proxy** is a centralized HTTP→MCP bridge that enables agents to access remote MCP servers without directly connecting to each one. It simplifies agent configuration and provides a single point of control for MCP tool availability.

## Architecture Components

### 1. **Toolman Server (Remote)**
- **Location:** `http://toolman.agent-platform.svc.cluster.local:3000/mcp`
- **Purpose:** Proxy HTTP requests to configured MCP servers
- **Configuration:** Defined in toolman repository
- **Deployment:** Argo CD application in `infra/gitops/applications/toolman.yaml`

### 2. **Toolman CLI (Client)**
- **Binary:** Installed in agent runtime containers
- **Command:** `toolman --url <server-url> --tool <tool-name>`
- **Protocol:** Converts STDIO (MCP) ↔ HTTP (Toolman Server)
- **Usage:** Configured in agent MCP configuration files

### 3. **Local MCP Servers**
- **Execution:** Run directly in agent containers
- **Examples:** `filesystem`, `git`, `memory`, `kubernetes`
- **Protocol:** STDIO MCP (no proxy needed)
- **Configuration:** Defined per-agent in `cto-config.json`

---

## Tool Types

### **Remote Tools** (via Toolman Proxy)

Remote tools are accessed through the toolman HTTP proxy. Agents call `toolman` CLI which proxies to the remote server.

**Configuration:**
```json
{
  "agents": {
    "rex": {
      "tools": {
        "remote": [
          "brave_search_brave_web_search",
          "rustdocs_query_rust_docs",
          "memory_create_entities"
        ]
      }
    }
  }
}
```

**Generated MCP Config:**
```json
{
  "mcpServers": {
    "brave_search_brave_web_search": {
      "command": "toolman",
      "args": ["--url", "http://toolman.agent-platform.svc.cluster.local:3000/mcp", "--tool", "brave_search_brave_web_search"],
      "env": {
        "TOOLMAN_SERVER_URL": "http://toolman.agent-platform.svc.cluster.local:3000/mcp"
      }
    }
  }
}
```

**Available Remote Tools:**
- `brave_search_brave_web_search` - Web search via Brave API
- `rustdocs_query_rust_docs` - Rust documentation search
- `memory_create_entities` - Create memory entities
- `memory_add_observations` - Add observations to memory
- `context7_get_library_docs` - Library documentation retrieval
- _...and others configured in toolman server_

### **Local Servers** (Direct Execution)

Local MCP servers run directly in the agent container without proxying.

**Configuration:**
```json
{
  "agents": {
    "blaze": {
      "tools": {
        "localServers": {
          "filesystem": {
            "enabled": true,
            "command": "npx",
            "args": ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"],
            "tools": ["read_file", "write_file", "list_directory", "search_files", "directory_tree"]
          },
          "git": {
            "enabled": true,
            "command": "mcp-server-git",
            "args": [],
            "tools": ["git_status", "git_diff", "git_log", "git_show"]
          }
        }
      }
    }
  }
}
```

**Generated MCP Config:**
```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"],
      "env": {}
    },
    "git": {
      "command": "mcp-server-git",
      "args": [],
      "env": {}
    }
  }
}
```

**Available Local Servers:**
- `filesystem` - File system operations (read, write, list, search)
- `git` - Git operations (status, diff, log, show)
- `kubernetes` - Kubernetes operations (list, describe, create resources)
- `shadcn` - shadcn/ui component installation _(NEW)_

---

## Configuration Flow

### Agent Configuration (`cto-config.json`)

```json
{
  "agents": {
    "blaze": {
      "githubApp": "5DLabs-Blaze",
      "cli": "codex",
      "model": "gpt-5-codex",
      "tools": {
        "remote": [
          "brave_search_brave_web_search",
          "context7_get_library_docs"
        ],
        "localServers": {
          "filesystem": {
            "enabled": true,
            "tools": ["read_file", "write_file", "list_directory", "search_files"]
          },
          "git": {
            "enabled": true,
            "tools": ["git_status", "git_diff", "git_log"]
          },
          "shadcn": {
            "enabled": true,
            "command": "npx",
            "args": ["shadcn@latest", "mcp"],
            "tools": ["list_components", "add_component", "init_project"]
          }
        }
      }
    }
  }
}
```

### Controller Processing

1. **Controller** reads agent configuration
2. **Extracts** `tools.remote` and `tools.localServers`
3. **Generates** MCP configuration JSON
4. **Templates** agent-specific configuration files
5. **Mounts** configuration as ConfigMap in agent pod

### Runtime Execution

1. **Agent starts** (Claude Code, Codex, Factory, etc.)
2. **Reads** `.mcp.json` from working directory
3. **Launches** MCP servers defined in configuration
4. **Toolman CLI** proxies remote tool requests to toolman server
5. **Local servers** execute directly in container
6. **Agent** uses tools via MCP protocol

---

## Adding New Tools

### Remote Tools (Toolman Server)

1. **Add server** to toolman repository configuration
2. **Deploy** toolman with updated config
3. **Add tool name** to agent's `remote` array in `cto-config.json`
4. **Agent** will automatically have access

**Example:**
```json
{
  "agents": {
    "blaze": {
      "tools": {
        "remote": [
          "brave_search_brave_web_search",
          "new_tool_name"  // ← Add here
        ]
      }
    }
  }
}
```

### Local Servers

1. **Ensure binary/package** is available in runtime container
2. **Add server configuration** to agent's `localServers` in `cto-config.json`
3. **Define tools** that server provides
4. **Agent** will automatically launch server

**Example:**
```json
{
  "agents": {
    "blaze": {
      "tools": {
        "localServers": {
          "shadcn": {  // ← Add new server
            "enabled": true,
            "command": "npx",
            "args": ["shadcn@latest", "mcp"],
            "tools": ["list_components", "add_component", "init_project"]
          }
        }
      }
    }
  }
}
```

---

## Tool Selection Strategy

### By Agent Role

**Morgan (Documentation):**
- Remote: `brave_search_brave_web_search`, `memory_create_entities`, `rustdocs_query_rust_docs`
- Local: `filesystem`, `git`

**Rex (Backend Implementation):**
- Remote: `memory_create_entities`, `rustdocs_query_rust_docs`
- Local: `filesystem`, `git`

**Blaze (Frontend Implementation):**
- Remote: `brave_search_brave_web_search`, `context7_get_library_docs`
- Local: `filesystem`, `git`, `shadcn`

**Cleo (Code Quality):**
- Remote: `memory_create_entities`, `rustdocs_query_rust_docs`
- Local: `filesystem`, `git`

**Tess (QA/Testing):**
- Remote: `memory_add_observations`
- Local: `filesystem`, `git`, `kubernetes`

### By Task Type

**Research/Discovery Tasks:**
- Add `brave_search_brave_web_search`
- Add `context7_get_library_docs`

**Infrastructure Tasks:**
- Add `kubernetes` local server

**Frontend Tasks:**
- Add `shadcn` local server
- Add `context7_get_library_docs` for component docs

**Backend/API Tasks:**
- Add `rustdocs_query_rust_docs`
- Add database-specific tools if available

---

## Benefits

### 1. **Centralized Management**
- Single point to add/remove MCP servers
- No need to update every agent configuration
- Consistent tool availability across agents

### 2. **Network Efficiency**
- One HTTP connection to toolman (multiplexed)
- vs. multiple direct MCP server connections
- Reduced network overhead

### 3. **Security**
- Agent containers only need access to toolman URL
- Toolman handles authentication to backend servers
- Simplified network policies

### 4. **Flexibility**
- Mix remote (proxied) and local (direct) tools
- Per-agent tool customization
- Easy to enable/disable tools

### 5. **Observability**
- Centralized logging of tool usage
- Metrics collection at proxy layer
- Easier to debug tool issues

---

## Implementation Details

### Toolman Server URL

**Environment Variable:**
```bash
TOOLMAN_SERVER_URL=http://toolman.agent-platform.svc.cluster.local:3000/mcp
```

**Default (if not set):**
```rust
// controller/src/cli/adapters/claude.rs
let toolman_url = std::env::var("TOOLMAN_SERVER_URL").unwrap_or_else(|_| {
    "http://toolman.agent-platform.svc.cluster.local:3000/mcp".to_string()
});
```

### MCP Configuration Generation

**Controller Code:**
```rust
// controller/src/cli/adapters/claude.rs
fn generate_mcp_config(tools: Option<&ToolConfiguration>) -> Value {
    let mut mcp_servers = json!({});
    
    if let Some(tool_config) = tools {
        // Add remote tools (via toolman)
        for tool_name in &tool_config.remote {
            let server_config = json!({
                "command": "toolman",
                "args": ["--url", toolman_url.clone(), "--tool", tool_name],
                "env": {
                    "TOOLMAN_SERVER_URL": toolman_url.clone()
                }
            });
            mcp_servers[tool_name] = server_config;
        }
        
        // Add local servers
        if let Some(local_servers) = &tool_config.local_servers {
            for (server_name, server_config) in local_servers {
                if server_config.enabled {
                    let local_server_config = json!({
                        "command": server_config.command.as_deref().unwrap_or(&format!("mcp-server-{}", server_name)),
                        "args": server_config.args.clone().unwrap_or_default(),
                        "env": {}
                    });
                    mcp_servers[server_name] = local_server_config;
                }
            }
        }
    }
    
    mcp_servers
}
```

---

## Troubleshooting

### Remote Tool Not Working

**Check:**
1. Toolman server is running: `kubectl get pods -n agent-platform | grep toolman`
2. Tool is configured in toolman server
3. Agent has tool in `remote` array
4. Network connectivity to toolman server

**Debug:**
```bash
# Test toolman directly
toolman --url http://toolman.agent-platform.svc.cluster.local:3000/mcp --tool brave_search_brave_web_search

# Check toolman logs
kubectl logs -n agent-platform deployment/toolman
```

### Local Server Not Starting

**Check:**
1. Binary/package available in container: `which npx`, `which mcp-server-git`
2. Server configuration correct in `localServers`
3. Server `enabled: true`

**Debug:**
```bash
# Test server directly
npx -y @modelcontextprotocol/server-filesystem /workspace

# Check MCP configuration
cat /workspace/.mcp.json | jq .
```

### Tool Not Available to Agent

**Check:**
1. Tool listed in agent's configuration
2. MCP config generated correctly
3. Agent CLI supports MCP (Claude Code, Codex, Factory do)

**Debug:**
```bash
# Verify MCP configuration
cat $CLAUDE_WORK_DIR/.mcp.json

# Check agent logs
kubectl logs -n agent-platform <agent-pod-name>
```

---

## Future Enhancements

1. **Dynamic Tool Discovery** - Agent queries available tools at runtime
2. **Tool Usage Metrics** - Track which tools are used most
3. **Tool Authentication** - Per-agent API key management
4. **Tool Rate Limiting** - Prevent abuse of expensive APIs
5. **Tool Caching** - Cache responses for idempotent tools

---

## References

- [Toolman Repository](https://github.com/5dlabs/toolman)
- [MCP Specification](https://modelcontextprotocol.io)
- [Controller MCP Integration](../../controller/src/cli/adapters/)
- [Agent Configuration Examples](../../cto-config-example.json)

