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
# 3. Check for Custom Agents (auto-discovered from ~/.claude/agents/)
# -----------------------------------------------------------------------------
echo "" >&2
echo "--- Checking Custom Agents ---" >&2
AGENTS_DIR="${HOME}/.claude/agents"

if [[ -d "${AGENTS_DIR}" ]]; then
  echo "📂 Agents directory: ${AGENTS_DIR}" >&2
  for agent_file in "${AGENTS_DIR}"/*.md; do
    if [[ -f "$agent_file" ]]; then
      agent_name=$(basename "$agent_file" .md)
      echo "  ✓ Found agent: ${agent_name}" >&2
    fi
  done
else
  echo "  No custom agents directory found" >&2
fi

# -----------------------------------------------------------------------------
# 4. Execute Claude CLI
# -----------------------------------------------------------------------------
echo "" >&2
echo "--- Executing Claude CLI ---" >&2

# The prompt - read from prompt.md file or use CLAUDE_PROMPT env var
if [[ -f "${WORKSPACE}/prompt.md" ]]; then
  echo "📋 Reading prompt from ${WORKSPACE}/prompt.md" >&2
  PROMPT="$(cat "${WORKSPACE}/prompt.md")"
elif [[ -n "${CLAUDE_PROMPT:-}" ]]; then
  PROMPT="${CLAUDE_PROMPT}"
else
  echo "❌ Error: No prompt found. Either create ${WORKSPACE}/prompt.md or set CLAUDE_PROMPT env var" >&2
  exit 1
fi

# Run Claude with streaming output for sidecar parsing
# Custom agents are auto-discovered from ~/.claude/agents/*.md
# The init message in stream-json contains tools, skills, agents, and mcp_servers
(echo "n" | claude --print --output-format stream-json --verbose \
  --dangerously-skip-permissions \
  "${PROMPT}" 2>&1) | tee "${WORKSPACE}/stream.jsonl"

echo "" >&2
echo "✅ Claude test complete" >&2
