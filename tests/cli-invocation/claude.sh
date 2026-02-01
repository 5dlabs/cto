#!/usr/bin/env bash
# =========================================================================
# Claude Code CLI Invocation Test Script
# Combined from: templates/clis/claude.sh.hbs + partials
# 
# This is a standalone shell script for local testing that mirrors
# exactly what runs in production CodeRun containers.
#
# Usage:
#   PROMPT="your prompt" ./tests/cli-invocation/claude.sh
#
# Environment:
#   PROMPT              - The prompt to send (required, or set in script)
#   CLAUDE_WORK_DIR     - Working directory (default: /tmp/claude-test)
#   TOOLS_SERVER_URL    - MCP tools server (default: cluster service)
#   MCP_CONFIG          - Path to MCP config (auto-created if not set)
# =========================================================================
set -euo pipefail

echo "═══════════════════════════════════════════════════════════════"
echo "║               CLAUDE CODE CLI INVOCATION                     ║"
echo "═══════════════════════════════════════════════════════════════"

# Configuration
CLAUDE_WORK_DIR="${CLAUDE_WORK_DIR:-/tmp/claude-test}"
TOOLS_SERVER_URL="${TOOLS_SERVER_URL:-http://cto-tools.cto.svc.cluster.local:3000}"
mkdir -p "$CLAUDE_WORK_DIR"

# =========================================================================
# MCP Tools Connectivity Check (from mcp-check.sh.hbs)
# =========================================================================
echo ""
echo "🔧 MCP Tools Diagnostics:"
echo "───────────────────────────────────────────────────────────────"

# Step 1: Check if tools-client binary exists
echo ""
echo "📦 MCP Client Binary:"
if command -v tools >/dev/null 2>&1; then
    TOOLS_PATH=$(command -v tools)
    echo "  ✅ tools binary found: $TOOLS_PATH"
    TOOLS_VERSION_OUTPUT=$(tools --version 2>&1 || echo "unknown")
    echo "  → Version: $TOOLS_VERSION_OUTPUT"
elif command -v tools-client >/dev/null 2>&1; then
    TOOLS_PATH=$(command -v tools-client)
    echo "  ✅ tools-client binary found: $TOOLS_PATH"
    TOOLS_VERSION_OUTPUT=$(tools-client --version 2>&1 || echo "unknown")
    echo "  → Version: $TOOLS_VERSION_OUTPUT"
else
    echo "  ⚠️ tools/tools-client binary NOT FOUND (not needed for HTTP transport)"
fi

# Step 2: Test connectivity to tools server
echo ""
echo "🌐 Tools Server Connectivity:"
echo "  → Server URL: $TOOLS_SERVER_URL"

if command -v curl >/dev/null 2>&1; then
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout 5 "${TOOLS_SERVER_URL}/health" 2>/dev/null || echo "000")
    if [ "$HTTP_CODE" = "200" ]; then
        echo "  ✅ Health check: OK (HTTP $HTTP_CODE)"
    elif [ "$HTTP_CODE" = "000" ]; then
        echo "  ❌ Server unreachable (connection failed/timeout)"
    else
        echo "  ⚠️ Health returned HTTP $HTTP_CODE"
    fi
    
    # Test MCP endpoint
    MCP_RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" \
        --connect-timeout 5 \
        -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"mcp-check","version":"1.0.0"}}}' \
        "${TOOLS_SERVER_URL}/mcp" 2>/dev/null || echo "")
    if echo "$MCP_RESPONSE" | grep -q '"result"'; then
        echo "  ✅ MCP endpoint: OK (JSON-RPC working)"
    else
        echo "  ⚠️ MCP endpoint: Issue - $MCP_RESPONSE"
    fi
fi

# Step 3: Create or use MCP config
echo ""
echo "📋 MCP Configuration:"

MCP_CONFIG_FILE="${MCP_CONFIG:-$CLAUDE_WORK_DIR/.mcp.json}"

if [ ! -f "$MCP_CONFIG_FILE" ]; then
    echo "  → Creating MCP config: $MCP_CONFIG_FILE"
    cat > "$MCP_CONFIG_FILE" << EOF
{
  "mcpServers": {
    "cto-tools": {
      "url": "${TOOLS_SERVER_URL}/mcp",
      "transport": {
        "type": "sse"
      }
    }
  }
}
EOF
fi

echo "  ✅ Config: $MCP_CONFIG_FILE"
if command -v jq >/dev/null 2>&1; then
    jq empty "$MCP_CONFIG_FILE" 2>/dev/null && echo "  ✅ Valid JSON" || echo "  ❌ Invalid JSON"
fi

echo ""
echo "───────────────────────────────────────────────────────────────"

# =========================================================================
# Claude Code CLI Execution
# =========================================================================

# Claude Code CLI - use binary (same as production runtime image)
CLAUDE_CMD="claude"
echo "✓ Using Claude CLI binary"

# Output format: stream-json for programmatic parsing by sidecar
CLAUDE_CMD="$CLAUDE_CMD --output-format stream-json"

# Verbose mode for debugging
CLAUDE_CMD="$CLAUDE_CMD --verbose"

# MCP configuration
if [ -f "$MCP_CONFIG_FILE" ]; then
    echo "✓ Adding MCP configuration: $MCP_CONFIG_FILE"
    CLAUDE_CMD="$CLAUDE_CMD --mcp-config $MCP_CONFIG_FILE --strict-mcp-config"
fi

# Permission mode for automation
CLAUDE_CMD="$CLAUDE_CMD --dangerously-skip-permissions"

# Determine prompt
USER_PROMPT="${PROMPT:-List available MCP tools and report the count.}"
echo "✓ Prompt: $USER_PROMPT"

# Stream output file for sidecar to parse
STREAM_OUTPUT="$CLAUDE_WORK_DIR/claude-stream.jsonl"
rm -f "$STREAM_OUTPUT" 2>/dev/null || true
touch "$STREAM_OUTPUT"

echo "✓ Claude stream output: $STREAM_OUTPUT"

# Add --print flag for non-interactive mode
CLAUDE_CMD="$CLAUDE_CMD --print"

echo ""
echo "🚀 Claude Command: $CLAUDE_CMD \"<prompt>\""
echo ""
echo "✓ Prompt: $(echo "$USER_PROMPT" | wc -c | tr -d ' ') bytes"

# Execute Claude with prompt via stdin
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "║               EXECUTION START                                ║"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Note: Claude CLI may block waiting for stdin confirmation in some environments
# Piping 'n' to stdin bypasses any interactive prompts
# Use background+wait pattern for better signal handling
echo "✓ Executing Claude CLI..."

# Execute and capture output
echo "n" | $CLAUDE_CMD "$USER_PROMPT" 2>&1 | tee "$STREAM_OUTPUT"
CLAUDE_EXIT_CODE=${PIPESTATUS[1]}

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "║               EXECUTION COMPLETE                             ║"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "✓ Claude execution completed (exit code: $CLAUDE_EXIT_CODE)"
echo "✓ Stream output: $STREAM_OUTPUT"
echo "✓ Lines: $(wc -l < "$STREAM_OUTPUT")"

exit $CLAUDE_EXIT_CODE
