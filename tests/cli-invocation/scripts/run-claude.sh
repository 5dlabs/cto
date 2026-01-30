#!/usr/bin/env bash
# =============================================================================
# Claude Container Script - Mirrors Controller's container.sh.hbs
# =============================================================================
#
# This script mirrors the controller's generated container script for Claude.
# It sets up MCP tools and executes the CLI with proper output streaming.
#
# Environment:
#   MCP_CLIENT_CONFIG  - Path to client-config.json for tool filtering
#   TOOLS_URL          - URL of the CTO tools server
#
# =============================================================================

set -euo pipefail

# Configuration
TOOLS_URL="${TOOLS_URL:-http://tools.fra.5dlabs.ai/mcp}"
WORKSPACE="${CLI_WORK_DIR:-/workspace}"

echo "=== Claude Container Script ===" >&2
echo "  Workspace: ${WORKSPACE}" >&2
echo "  Tools URL: ${TOOLS_URL}" >&2
echo "  Client Config: ${MCP_CLIENT_CONFIG:-not set}" >&2

# -----------------------------------------------------------------------------
# 1. Configure MCP Server (tool filtering via client-config.json)
# -----------------------------------------------------------------------------
echo "" >&2
echo "--- Configuring MCP Server ---" >&2

# Remove existing server if present (in case of re-run)
claude mcp remove cto-tools >&2 2>&1 || true

# The tools binary reads MCP_CLIENT_CONFIG for tool filtering
claude mcp add cto-tools -- tools "${TOOLS_URL}" "${WORKSPACE}" >&2 2>&1
echo "✓ MCP server configured" >&2

# -----------------------------------------------------------------------------
# 2. Verify MCP Server Connection
# -----------------------------------------------------------------------------
echo "" >&2
echo "--- Verifying MCP Server ---" >&2
claude mcp list >&2 2>&1 || true

# -----------------------------------------------------------------------------
# 3. Execute Claude CLI
# -----------------------------------------------------------------------------
echo "" >&2
echo "--- Executing Claude CLI ---" >&2

# The prompt - can be overridden via CLAUDE_PROMPT env var
# Default prompt is designed to exercise MCP tools (context7, firecrawl, etc.)
PROMPT="${CLAUDE_PROMPT:-You have access to CTO MCP tools. Please complete these tasks to verify the tools work:

## Task 1: Use Context7 to look up documentation
Use the context7 MCP tools to:
1. Resolve the library ID for 'effect' (the TypeScript Effect library)
2. Get documentation about Effect's Schema module

## Task 2: Use Firecrawl to research
Use the firecrawl MCP tools to:
1. Search for 'Rust axum framework best practices 2025'
2. Summarize the top result

## Task 3: Create a summary file
Create a file at /workspace/mcp-test-results.md that summarizes:
- Which MCP tools you called
- What results you got from each
- Any errors encountered

Be explicit about which tools you're calling so we can verify the MCP integration is working.}"

# Run Claude with streaming output for sidecar parsing
# The init message in stream-json contains tools, skills, and mcp_servers
(echo "n" | claude --print --output-format stream-json --verbose \
  --dangerously-skip-permissions \
  "${PROMPT}" 2>&1) | tee "${WORKSPACE}/stream.jsonl"

echo "" >&2
echo "✅ Claude test complete" >&2
