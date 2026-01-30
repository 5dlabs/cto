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
PROMPT="${CLAUDE_PROMPT:-Build a small Python project in /workspace with the following structure:

1. First, explore what files exist in /workspace using Glob
2. Create a project structure:
   - /workspace/src/calculator.py - A Calculator class with add, subtract, multiply, divide methods
   - /workspace/src/__init__.py - Package init  
   - /workspace/tests/test_calculator.py - Unit tests using unittest
   - /workspace/README.md - Documentation with usage examples

3. Run the tests using: python3 -m pytest tests/ -v (or unittest if pytest not available)
4. Show me the test results and a summary of what you created

Make sure to:
- Add proper docstrings to all functions
- Handle division by zero gracefully
- Include at least 5 test cases}"

# Run Claude with streaming output for sidecar parsing
# The init message in stream-json contains tools, skills, and mcp_servers
(echo "n" | claude --print --output-format stream-json --verbose \
  --dangerously-skip-permissions \
  "${PROMPT}" 2>&1) | tee "${WORKSPACE}/stream.jsonl"

echo "" >&2
echo "✅ Claude test complete" >&2
