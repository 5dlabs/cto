# Agent-Specific Client Configuration

## Overview

The CTO platform now supports agent-specific MCP (Model Context Protocol) client configurations. Instead of using a single static `client-config.json` for all agents, each agent can now have its own tailored tool configuration based on its specific role and requirements.

## How It Works

### 1. Agent Tool Configuration

Each agent in the CTO configuration now has a `tools` section that defines:

- **Remote Tools**: External MCP tools (e.g., `memory_create_entities`, `brave_web_search`, `rustdocs_query_rust_docs`)
- **Local Servers**: Local MCP servers (e.g., filesystem, git) with specific tool access

### 2. Automatic Configuration Generation

When a CodeRun is created:
1. The system identifies the agent based on the `githubApp` field
2. Looks up the agent's tool configuration in the CTO config
3. Generates a customized `client-config.json` for that agent
4. The configuration is mounted in the agent's workspace at `/workspace/client-config.json`

### 3. Container Script Integration

All agent container scripts now automatically:
- Copy `client-config.json` from the ConfigMap to the workspace
- Set up MCP client environment variables
- Configure the agent to use its specific tool set

## Configuration Example

### CTO Config (`cto-config.json`)

```json
{
  "agents": {
    "rex": {
      "githubApp": "5DLabs-Rex",
      "tools": {
        "remote": ["memory_create_entities", "memory_add_observations", "rustdocs_query_rust_docs"],
        "localServers": {
          "filesystem": {
            "enabled": true,
            "tools": ["read_file", "write_file", "list_directory", "search_files", "directory_tree"]
          },
          "git": {
            "enabled": true,
            "tools": ["git_status", "git_diff", "git_log", "git_show"]
          }
        }
      }
    },
    "cleo": {
      "githubApp": "5DLabs-Cleo",
      "tools": {
        "remote": ["memory_create_entities", "memory_add_observations", "rustdocs_query_rust_docs"],
        "localServers": {
          "filesystem": {
            "enabled": true,
            "tools": ["read_file", "write_file", "list_directory", "search_files", "directory_tree"]
          },
          "git": {
            "enabled": true,
            "tools": ["git_status", "git_diff", "git_log", "git_show"]
          }
        }
      }
    },
    "cipher": {
      "githubApp": "5DLabs-Cipher",
      "tools": {
        "remote": ["memory_create_entities", "memory_add_observations", "brave_web_search"],
        "localServers": {
          "filesystem": {
            "enabled": false,
            "tools": []
          },
          "git": {
            "enabled": false,
            "tools": []
          }
        }
      }
    }
  }
}
```

### Generated Client Config

For Rex, the system generates:

```json
{
  "remoteTools": ["memory_create_entities", "memory_add_observations", "rustdocs_query_rust_docs"],
  "localServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"],
      "tools": ["read_file", "write_file", "list_directory", "search_files", "directory_tree"],
      "workingDirectory": "project_root"
    },
    "git": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-git", "/workspace"],
      "tools": ["git_status", "git_diff", "git_log", "git_show"],
      "workingDirectory": "project_root"
    }
  }
}
```

For Cipher (security-focused), the system generates:

```json
{
  "remoteTools": ["memory_create_entities", "memory_add_observations", "brave_web_search"],
  "localServers": {}
}
```

## Agent-Specific Tool Assignments

### Morgan (Documentation Specialist)
- **Full filesystem access**: Needs to read/write documentation files
- **Full git access**: Needs to track changes and commit documentation
- **Web search**: Research capabilities for documentation
- **Rust docs**: Access to Rust documentation for technical writing

### Rex (Backend Architect)
- **Full filesystem access**: Code architecture and implementation
- **Full git access**: Version control for code changes
- **Rust docs**: Access to Rust documentation for architecture decisions
- **Memory tools**: Context management for complex implementations

### Cleo (Code Quality Specialist)
- **Full filesystem access**: Code analysis and formatting
- **Full git access**: Committing quality improvements
- **Rust docs**: Understanding code patterns and best practices

### Tess (QA Specialist)
- **Limited filesystem access**: Reading code for testing
- **Limited git access**: Checking changes for testing
- **Memory tools**: Test case management

### Cipher (Security Specialist)
- **No filesystem access**: Security isolation
- **No git access**: Prevent direct repository modifications
- **Web search**: Security research and vulnerability databases
- **Memory tools**: Security analysis context

### Blaze (Performance Specialist)
- **Basic filesystem access**: Performance analysis
- **Basic git access**: Performance change tracking
- **Web search**: Performance research and benchmarks

## Benefits

### 1. **Security by Design**
- Cipher has no direct filesystem access, preventing accidental security issues
- Blaze has limited access appropriate for performance analysis
- Each agent gets exactly the tools it needs

### 2. **Role-Based Tool Access**
- Documentation agents get web search and extensive file access
- Code quality agents get full code analysis tools
- QA agents get appropriate testing tools
- Security agents get research tools without direct access

### 3. **Automatic Configuration**
- No manual configuration required
- Agent tools are automatically configured based on role
- Consistent tool access across all tasks for each agent

### 4. **Workspace Isolation**
- Each agent gets its own `client-config.json` in the workspace
- Tools are configured per agent, not per task
- Consistent behavior across all tasks for each agent

## Implementation Details

### Controller Changes
- Added `AgentTools` configuration structures to `ControllerConfig`
- Extended `CodeTemplateGenerator` with `generate_client_config()` method
- Agent identification via GitHub app name mapping
- Tool configuration lookup and JSON generation

### Container Script Updates
- All agent container scripts updated to copy `client-config.json`
- Automatic MCP client configuration setup
- Environment variable configuration for tool access

### Configuration Flow
1. CodeRun specifies `githubApp` (e.g., "5DLabs-Rex")
2. Controller extracts agent name ("rex") from GitHub app
3. Looks up agent tools in CTO configuration
4. Generates `client-config.json` with agent's specific tools
5. Mounts configuration in agent's workspace
6. Container script copies to workspace root for MCP client access

## Future Extensions

This foundation enables:
- Dynamic tool assignment based on task requirements
- Tool access control policies
- Audit logging of tool usage by agent
- Performance monitoring of tool usage patterns
- Integration with external tool registries
