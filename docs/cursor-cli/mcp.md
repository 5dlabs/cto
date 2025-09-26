# MCP

## Overview

The Cursor CLI supports [Model Context Protocol (MCP)](/docs/context/mcp) servers, allowing you to connect external tools and data sources to `cursor-agent`. **MCP in the CLI uses the same configuration as the editor** - any MCP servers you've configured will work with both.

[Learn about MCP

New to MCP? Read the complete guide on configuration, authentication, and
available servers](/docs/context/mcp)
## CLI commands

Use the `cursor-agent mcp` command to manage MCP servers:

### List configured servers

View all configured MCP servers and their current status:

```
cursor-agent mcp list
```

This shows:

- Server names and identifiers
- Connection status (connected/disconnected)
- Configuration source (project or global)
- Transport method (stdio, HTTP, SSE)

### List available tools

View tools provided by a specific MCP server:

```
cursor-agent mcp list-tools <identifier>
```

This displays:

- Tool names and descriptions
- Required and optional parameters
- Parameter types and constraints

### Login to MCP server

Authenticate with an MCP server configured in your `mcp.json`:

```
cursor-agent mcp login <identifier>
```

### Disable MCP server

Remove an MCP server from the local approved list:

```
cursor-agent mcp disable <identifier>
```

## Using MCP with Agent

Once you have MCP servers configured (see the [main MCP guide](/docs/context/mcp) for setup), `cursor-agent` automatically discovers and uses available tools when relevant to your requests.

```
# Check what MCP servers are available
cursor-agent mcp list

# See what tools a specific server provides
cursor-agent mcp list-tools playwright

# Use cursor-agent - it automatically uses MCP tools when helpful
cursor-agent --prompt "Navigate to google.com and take a screenshot of the search page"
```

The CLI follows the same configuration precedence as the editor (project → global → nested), automatically discovering configurations from parent directories.

## Related

- [MCP Overview](/docs/context/mcp): Complete MCP guide: setup, configuration, and authentication
- [Available MCP Tools](/docs/context/mcp/directory): Browse pre-built MCP servers you can use