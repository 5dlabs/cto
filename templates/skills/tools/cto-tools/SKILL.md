---
name: cto-tools
description: Dynamic MCP tool discovery and invocation via TypeScript code execution in Deno sandbox.
agents: [morgan, rex, grizz, nova, viper, blaze, tap, spark, cleo, cipher, tess, stitch, atlas, bolt, block, vex, angie, glitch, lex, hype, tally, chase]
triggers: [tool, mcp, escalate, discover tools, capability, search tools]
---

# CTO Tools — Dynamic MCP Tool Access

You have access to a large catalog of MCP tools via the `cto-tools` HTTP proxy. Rather than having all tools pre-loaded, you can discover and invoke them on demand by writing short TypeScript scripts.

## Quick Reference

### List available tools
```bash
cto-tools mcp list
```

### Search for a tool by keyword
```bash
cto-tools mcp list | grep -i github
```

### Get full schema for a tool
```bash
cto-tools mcp describe <tool_name>
```

### Call a tool
```bash
cto-tools mcp call <tool_name> --json '{"param": "value"}'
```

### Request access to a tool outside your prewarm set
```bash
cto-tools mcp escalate <tool_name> --reason "Need to query Grafana dashboards for deployment metrics"
```

## How It Works

1. **Your prewarm set** contains the tools loaded at session start. These work immediately.
2. **The full catalog** has many more tools. Use `cto-tools mcp list` to see them all.
3. **Escalation**: If you need a tool outside your prewarm set, call `tools_request_capability` (or `cto-tools mcp escalate`). If the policy allows it, you'll get access for the rest of this session.

## TypeScript Code Execution (Advanced)

For complex multi-tool workflows, write a TypeScript script and run it with Deno:

```typescript
// save as /tmp/workflow.ts
import { callTool } from "/.cto-tools/mcp.ts";

// Search for relevant code
const results = await callTool("github_search_code", {
  q: "authentication middleware",
  repo: "5dlabs/myapp"
});

// Use results to query metrics
const dashboard = await callTool("grafana_get_dashboard_by_uid", {
  uid: "api-latency"
});

console.log(JSON.stringify({ code: results, metrics: dashboard }, null, 2));
```

Run it:
```bash
deno run --allow-net=localhost:3000 /tmp/workflow.ts
```

### Available imports from `/.cto-tools/mcp.ts`

| Function | Description |
|----------|-------------|
| `callTool(name, args)` | Invoke any MCP tool by name. Returns the tool's response. |
| `listTools()` | Get the full tool catalog with schemas. |
| `describeTool(name)` | Get the JSON schema for a specific tool. |

## When to Use What

- **Single tool call**: Use the MCP tool directly (it's in your prewarm set) or `cto-tools mcp call`
- **Multi-step workflow**: Write a TypeScript script with `callTool()` for orchestrated tool chains
- **Tool discovery**: `cto-tools mcp list` to browse, `cto-tools mcp describe` for schemas
- **Access denied**: `cto-tools mcp escalate` with a reason — the policy may grant it
