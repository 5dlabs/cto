# Intake Agent

A compiled TypeScript binary that wraps the official Claude Agent SDK for PRD parsing, task generation, and research capabilities.

## Overview

This agent provides JSON-based communication with Claude for:
- **parse_prd**: Generate tasks from a Product Requirements Document
- **expand_task**: Break down tasks into subtasks
- **analyze_complexity**: Analyze task complexity and recommend subtask counts
- **generate**: Generic text generation
- **research**: Gather context using MCP tools (Firecrawl, OctoCode, Context7)
- **research_capabilities**: Check available MCP research servers

## Architecture

```
Rust Intake CLI
      │
      ▼ JSON/stdin
┌─────────────────────┐
│   intake-agent      │  (Bun-compiled TypeScript)
│   Claude Agent SDK  │
│         │           │
│    ┌────┴────┐      │
│    │   MCP   │      │
│    │ Servers │      │
│    └─────────┘      │
└─────────────────────┘
      │ JSON/stdout
      ▼
   AI Response
```

## Building

```bash
# Install dependencies
bun install

# Build the binary
bun run build

# The binary is output to dist/intake-agent
```

## Usage

The agent reads JSON from stdin and writes JSON to stdout:

```bash
# Health check
echo '{"operation":"ping"}' | ./dist/intake-agent

# Check research capabilities
echo '{"operation":"research_capabilities"}' | ./dist/intake-agent

# Parse PRD
echo '{
  "operation": "parse_prd",
  "model": "claude-sonnet-4-20250514",
  "payload": {
    "prd_content": "Build a todo app with...",
    "num_tasks": 10,
    "next_id": 1
  }
}' | ./dist/intake-agent

# Research a topic (requires MCP credentials)
echo '{
  "operation": "research",
  "model": "claude-sonnet-4-20250514",
  "payload": {
    "topic": "Best practices for React authentication with Better Auth",
    "focus_areas": ["session management", "token refresh", "security"]
  }
}' | ./dist/intake-agent
```

## JSON Protocol

### Request Format

```json
{
  "operation": "parse_prd" | "expand_task" | "analyze_complexity" | "generate" | "research" | "research_capabilities" | "ping",
  "model": "claude-sonnet-4-20250514",
  "options": {
    "temperature": 0.7,
    "max_tokens": 64000,
    "mcp_enabled": false
  },
  "payload": { /* operation-specific */ }
}
```

### Response Format (Success)

```json
{
  "success": true,
  "data": { /* operation-specific result */ },
  "usage": {
    "input_tokens": 1234,
    "output_tokens": 5678,
    "total_tokens": 6912
  },
  "model": "claude-sonnet-4-20250514",
  "provider": "claude-agent-sdk"
}
```

### Response Format (Error)

```json
{
  "success": false,
  "error": "Error message",
  "error_type": "api_error" | "parse_error" | "validation_error" | "mcp_error",
  "details": "Optional additional context"
}
```

## Operations

### parse_prd

Generate tasks from a PRD.

**Payload:**
```json
{
  "prd_content": "string (required)",
  "prd_path": "string",
  "num_tasks": 10,
  "next_id": 1,
  "research": false,
  "default_task_priority": "medium"
}
```

### expand_task

Break down a task into subtasks.

**Payload:**
```json
{
  "task": {
    "id": "1",
    "title": "Task title",
    "description": "Task description",
    "details": "Implementation details"
  },
  "subtask_count": 5,
  "next_subtask_id": 1,
  "use_research": false,
  "enable_subagents": false
}
```

### analyze_complexity

Analyze task complexity.

**Payload:**
```json
{
  "tasks": [
    { "id": "1", "title": "...", "description": "...", "details": "..." }
  ],
  "threshold": 5,
  "use_research": false
}
```

### generate

Generic text generation.

**Payload:**
```json
{
  "system_prompt": "string",
  "user_prompt": "string (required)",
  "prefill": "string (optional, to start assistant response)"
}
```

### research

Research a topic using MCP tools.

**Payload:**
```json
{
  "topic": "string (required)",
  "focus_areas": ["array", "of", "topics"],
  "max_turns": 5,
  "enable_servers": {
    "firecrawl": true,
    "octocode": true,
    "context7": true,
    "websearch": true
  }
}
```

**Response Data:**
```json
{
  "summary": "Executive summary of findings",
  "findings": [
    {
      "topic": "Topic area",
      "content": "Detailed findings",
      "source": "URL or reference"
    }
  ],
  "recommendations": ["List of recommendations"],
  "servers_used": ["firecrawl", "context7"]
}
```

### research_capabilities

Check which MCP servers are available.

**Response:**
```json
{
  "available": true,
  "servers": [
    { "name": "firecrawl", "available": false, "reason": "FIRECRAWL_API_KEY not set" },
    { "name": "octocode", "available": true },
    { "name": "context7", "available": true },
    { "name": "websearch", "available": false, "reason": "TAVILY_API_KEY not set" }
  ]
}
```

## MCP Server Configuration

The agent supports multiple MCP servers for research capabilities. Configure them via environment variables:

| Server | Environment Variable | Description |
|--------|---------------------|-------------|
| Firecrawl | `FIRECRAWL_API_KEY` | Web scraping and crawling |
| OctoCode | `GITHUB_TOKEN` or `OCTOCODE_API_KEY` | GitHub code search |
| Context7 | None (always available) | Library documentation |
| Web Search | `TAVILY_API_KEY` or `SERPER_API_KEY` | Web search |

## Development

```bash
# Type check
bun run typecheck

# Run directly (without building)
echo '{"operation":"ping"}' | bun run src/index.ts

# Clean build artifacts
bun run clean
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `ANTHROPIC_API_KEY` | Required for Claude API access |
| `FIRECRAWL_API_KEY` | Optional: Enables Firecrawl MCP server |
| `GITHUB_TOKEN` | Optional: Enables OctoCode MCP server |
| `TAVILY_API_KEY` | Optional: Enables Tavily web search |
| `SERPER_API_KEY` | Optional: Enables Serper web search |

## Integration with Rust

The Rust `intake` crate calls this binary via subprocess:

```rust
// In crates/intake/src/ai/sdk_provider.rs
let child = Command::new("intake-agent")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()?;

// Write JSON request to stdin
serde_json::to_writer(stdin, &request)?;

// Read JSON response from stdout
let response: AgentResponse = serde_json::from_slice(&output.stdout)?;
```

Binary discovery order:
1. `INTAKE_AGENT_PATH` environment variable
2. `tools/intake-agent/dist/intake-agent` relative to workspace
3. `intake-agent` in PATH

## License

MIT
