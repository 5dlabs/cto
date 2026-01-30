#!/usr/bin/env bash
# =============================================================================
# Factory (Droid) Container Script - Mirrors Controller's container.sh.hbs
# =============================================================================
#
# This script mirrors the controller's generated container script for Droid.
# It sets up MCP tools and executes the CLI with proper output streaming.
#
# Environment:
#   MCP_CLIENT_CONFIG  - Path to client-config.json for tool filtering
#   TOOLS_URL          - URL of the CTO tools server
#   FACTORY_API_KEY    - Factory API key (required)
#
# =============================================================================

set -euo pipefail

# Configuration
TOOLS_URL="${TOOLS_URL:-http://tools.fra.5dlabs.ai/mcp}"
WORKSPACE="${CLI_WORK_DIR:-/workspace}"

echo "=== Factory (Droid) Container Script ===" >&2
echo "  Workspace: ${WORKSPACE}" >&2
echo "  Tools URL: ${TOOLS_URL}" >&2
echo "  Client Config: ${MCP_CLIENT_CONFIG:-not set}" >&2

# Check for API key
if [[ -z "${FACTORY_API_KEY:-}" ]]; then
    echo "❌ FACTORY_API_KEY not set" >&2
    echo '{"type":"result","subtype":"failure","is_error":true,"result":"FACTORY_API_KEY not set"}' > "${WORKSPACE}/stream.jsonl"
    exit 1
fi

# -----------------------------------------------------------------------------
# 1. Configure MCP Server
# -----------------------------------------------------------------------------
echo "" >&2
echo "--- Configuring MCP Server ---" >&2

# Remove existing server if present (in case of re-run)
droid mcp remove cto-tools >&2 2>&1 || true

# Add MCP server (http type for remote server)
# The tools binary path is used with MCP_CLIENT_CONFIG for tool filtering
droid mcp add --type http cto-tools "${TOOLS_URL}" >&2 2>&1
echo "✓ MCP server configured" >&2

# -----------------------------------------------------------------------------
# 2. List Available Tools (for init info)
# -----------------------------------------------------------------------------
echo "" >&2
echo "--- Listing Available Tools ---" >&2
droid exec --list-tools -o json > "${WORKSPACE}/mcp-tools.txt" 2>&1 || true
TOOL_COUNT=$(cat "${WORKSPACE}/mcp-tools.txt" | jq 'length' 2>/dev/null || echo "0")
echo "→ Available tools: ${TOOL_COUNT}" >&2

# -----------------------------------------------------------------------------
# 3. Execute Droid CLI
# -----------------------------------------------------------------------------
echo "" >&2
echo "--- Executing Droid CLI ---" >&2

# The prompt - can be overridden via DROID_PROMPT env var
PROMPT="${DROID_PROMPT:-Build a small Python project in /workspace with the following structure:

1. First, explore what files exist in /workspace using LS
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

# Run Droid with JSON output for sidecar parsing
# Use --auto medium for development operations (create files, run tests)
# Use --skip-permissions-unsafe in container environment for full access
droid exec \
  --skip-permissions-unsafe \
  --cwd "${WORKSPACE}" \
  -o json \
  "${PROMPT}" 2>&1 | tee "${WORKSPACE}/stream.jsonl"

echo "" >&2
echo "✅ Factory test complete" >&2
