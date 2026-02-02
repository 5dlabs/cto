#!/usr/bin/env bash
# Smoke test for MCP tools server - validates tool discovery
# Usage: ./smoke-test-tools.sh [expected_minimum_tools]
# Exit codes:
#   0 - Success (all servers provided tools, count meets minimum)
#   1 - Failure (some servers failed or count too low)

set -euo pipefail

TOOLS_URL="${TOOLS_URL:-http://localhost:3000}"
EXPECTED_MIN_TOOLS="${1:-300}"  # Default: expect at least 300 tools
TIMEOUT="${TIMEOUT:-180}"       # 3 minutes timeout for server to be ready

echo "🔧 MCP Tools Smoke Test"
echo "   URL: $TOOLS_URL"
echo "   Expected minimum tools: $EXPECTED_MIN_TOOLS"
echo ""

# Wait for server to be ready
echo "⏳ Waiting for server to be ready (timeout: ${TIMEOUT}s)..."
start_time=$(date +%s)
while ! curl -sf "$TOOLS_URL/health" > /dev/null 2>&1; do
    current_time=$(date +%s)
    elapsed=$((current_time - start_time))
    
    if [ $elapsed -ge $TIMEOUT ]; then
        echo "❌ Server did not become ready within ${TIMEOUT}s"
        exit 1
    fi
    
    echo "   Waiting... (${elapsed}s elapsed)"
    sleep 5
done

echo "✅ Server is responding"
echo ""

# Fetch tools list
echo "📡 Fetching tools list..."
response=$(curl -sf "$TOOLS_URL/mcp" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' 2>&1) || {
    echo "❌ Failed to fetch tools list"
    echo "Response: $response"
    exit 1
}

# Parse tool count
tool_count=$(echo "$response" | jq -r '.result.tools | length' 2>/dev/null) || {
    echo "❌ Failed to parse tools response"
    echo "Response: $response"
    exit 1
}

echo "📊 Tool Count: $tool_count"
echo ""

# Validate tool count
if [ "$tool_count" -lt "$EXPECTED_MIN_TOOLS" ]; then
    echo "❌ FAIL: Only $tool_count tools discovered (expected at least $EXPECTED_MIN_TOOLS)"
    echo ""
    echo "This indicates a regression in tool discovery."
    echo "Check server logs for failed/timed out servers."
    exit 1
fi

echo "✅ PASS: Tool count meets minimum requirement ($tool_count >= $EXPECTED_MIN_TOOLS)"

# Group tools by server to check for failures
echo ""
echo "📋 Tools by server:"
echo "$response" | jq -r '.result.tools[] | .name' | \
    sed 's/_[^_]*$//' | sort | uniq -c | sort -rn | head -20

echo ""
echo "✅ Smoke test passed!"
exit 0
