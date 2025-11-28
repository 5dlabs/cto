#!/usr/bin/env bash
# Tools Server Validation Script
# Validates that all MCP servers are properly exposing their tools
# and performs smoke tests on representative tools.
#
# Usage: ./scripts/validate-tools-server.sh [--server-url URL]

set -euo pipefail

# Configuration
TOOLS_SERVER_URL="${TOOLS_SERVER_URL:-http://cto-tools.cto.svc.cluster.local:3000}"
TIMEOUT=30

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --server-url)
            TOOLS_SERVER_URL="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [--server-url URL]"
            echo "  --server-url URL  Tools server URL (default: $TOOLS_SERVER_URL)"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}======================================${NC}"
echo -e "${BLUE}Tools Server Validation${NC}"
echo -e "${BLUE}======================================${NC}"
echo ""
echo "Server URL: $TOOLS_SERVER_URL"
echo ""

# Check server health first
echo -e "${BLUE}[1/4] Checking server health...${NC}"
HEALTH_RESPONSE=$(curl -s --connect-timeout 5 "$TOOLS_SERVER_URL/health" 2>&1) || {
    echo -e "${RED}✗ Server health check failed${NC}"
    echo "Response: $HEALTH_RESPONSE"
    echo ""
    echo "Make sure the tools server is running and accessible."
    echo "If using WireGuard, ensure you're connected."
    exit 1
}

if echo "$HEALTH_RESPONSE" | grep -q '"status":"ok"'; then
    echo -e "${GREEN}✓ Server is healthy${NC}"
else
    echo -e "${YELLOW}⚠ Unexpected health response: $HEALTH_RESPONSE${NC}"
fi
echo ""

# Query tools list via MCP protocol
echo -e "${BLUE}[2/4] Discovering tools via MCP protocol...${NC}"

# MCP tools/list request
TOOLS_REQUEST=$(cat <<EOF
{
    "jsonrpc": "2.0",
    "method": "tools/list",
    "params": {},
    "id": 1
}
EOF
)

# Write response to temp file to handle large responses
TEMP_RESPONSE=$(mktemp)
trap "rm -f $TEMP_RESPONSE" EXIT

curl -s --connect-timeout "$TIMEOUT" --max-time 120 \
    -X POST "$TOOLS_SERVER_URL/mcp" \
    -H "Content-Type: application/json" \
    -d "$TOOLS_REQUEST" > "$TEMP_RESPONSE" 2>&1

if [[ ! -s "$TEMP_RESPONSE" ]]; then
    echo -e "${RED}✗ Failed to query tools list (empty response)${NC}"
    exit 1
fi

# Check for error response
if jq -e '.error' "$TEMP_RESPONSE" >/dev/null 2>&1; then
    echo -e "${RED}✗ MCP error response:${NC}"
    jq '.error' "$TEMP_RESPONSE"
    exit 1
fi

# Extract tool names
TOOL_NAMES=$(jq -r '.result.tools[].name' "$TEMP_RESPONSE" 2>/dev/null) || {
    echo -e "${RED}✗ Failed to parse tools response${NC}"
    echo "Response (first 500 chars): $(head -c 500 "$TEMP_RESPONSE")"
    exit 1
}

TOTAL_TOOLS=$(echo "$TOOL_NAMES" | wc -l | tr -d ' ')
echo -e "${GREEN}✓ Found $TOTAL_TOOLS tools${NC}"
echo ""

# Count tools per server prefix
echo -e "${BLUE}Tools by server:${NC}"
declare -A SERVER_COUNTS

# Expected server prefixes based on values.yaml
EXPECTED_SERVERS=(
    "brave_search"
    "openmemory"
    "context7"
    "terraform"
    "kubernetes"
    "github"
    "shadcn"
    "ai_elements"
    "pg_aiguide"
    "solana"
    "firecrawl"
    "cloudflare_docs"
    "cloudflare_bindings"
    "cloudflare_observability"
    "cloudflare_radar"
    "grafana"
    "victoriametrics"
    "argocd"
    "nano_banana"
    "vault"
)

for server in "${EXPECTED_SERVERS[@]}"; do
    count=$(echo "$TOOL_NAMES" | grep -c "^${server}_" 2>/dev/null || true)
    # Handle empty/zero result
    if [[ -z "$count" ]] || [[ "$count" == "0" ]]; then
        count=0
    fi
    SERVER_COUNTS[$server]=$count
    if [[ $count -gt 0 ]]; then
        echo -e "  ${GREEN}✓${NC} $server: $count tools"
    else
        echo -e "  ${RED}✗${NC} $server: 0 tools (missing!)"
    fi
done

# Check for any tools not matching expected prefixes
OTHER_TOOLS=$(echo "$TOOL_NAMES" | grep -v -E "^($(IFS=\|; echo "${EXPECTED_SERVERS[*]}"))_" 2>/dev/null | head -5 || true)
if [[ -n "$OTHER_TOOLS" ]]; then
    echo -e "  ${YELLOW}?${NC} Other tools found:"
    echo "$OTHER_TOOLS" | sed 's/^/      /'
fi
echo ""

# Smoke tests
echo -e "${BLUE}[3/4] Running smoke tests on representative tools...${NC}"
echo ""

# Function to call a tool and check for success
call_tool() {
    local tool_name="$1"
    local params="$2"
    local description="$3"
    
    echo -n "  Testing $tool_name ($description)... "
    
    local request=$(cat <<EOF
{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
        "name": "$tool_name",
        "arguments": $params
    },
    "id": 1
}
EOF
)
    
    local response
    response=$(curl -s --connect-timeout "$TIMEOUT" --max-time 60 \
        -X POST "$TOOLS_SERVER_URL/mcp" \
        -H "Content-Type: application/json" \
        -d "$request" 2>&1)
    
    # Check for MCP error
    if echo "$response" | jq -e '.error' >/dev/null 2>&1; then
        local error_msg
        error_msg=$(echo "$response" | jq -r '.error.message // .error' 2>/dev/null)
        echo -e "${RED}✗ Error: $error_msg${NC}"
        return 1
    fi
    
    # Check for result
    if echo "$response" | jq -e '.result' >/dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
        return 0
    fi
    
    echo -e "${YELLOW}? Unexpected response${NC}"
    return 1
}

SMOKE_PASS=0
SMOKE_FAIL=0

# Test 1: kubernetes_pods_list (internal stdio server)
if call_tool "kubernetes_pods_list" '{}' "internal stdio"; then
    ((SMOKE_PASS++))
else
    ((SMOKE_FAIL++))
fi

# Test 2: argocd_list_applications (internal stdio with secrets)
if call_tool "argocd_list_applications" '{}' "internal stdio + secrets"; then
    ((SMOKE_PASS++))
else
    ((SMOKE_FAIL++))
fi

# Test 3: context7_resolve_library_id (external stdio via npx)
if call_tool "context7_resolve_library_id" '{"libraryName": "react"}' "external stdio (npx)"; then
    ((SMOKE_PASS++))
else
    ((SMOKE_FAIL++))
fi

# Test 4: cloudflare_docs_search_cloudflare_documentation (external HTTP)
if call_tool "cloudflare_docs_search_cloudflare_documentation" '{"query": "workers"}' "external HTTP"; then
    ((SMOKE_PASS++))
else
    ((SMOKE_FAIL++))
fi

# Test 5: vault_list_mounts (internal HTTP, separate pod)
# Note: This may fail if vault-mcp-server isn't deployed
if echo "$TOOL_NAMES" | grep -q "^vault_"; then
    if call_tool "vault_kv_list" '{"path": "secret"}' "internal HTTP (vault pod)"; then
        ((SMOKE_PASS++))
    else
        ((SMOKE_FAIL++))
    fi
else
    echo -e "  ${YELLOW}⚠ Skipping vault test (no vault tools found)${NC}"
fi

echo ""
echo -e "${BLUE}Smoke test results: ${GREEN}$SMOKE_PASS passed${NC}, ${RED}$SMOKE_FAIL failed${NC}"
echo ""

# Summary
echo -e "${BLUE}[4/4] Summary${NC}"
echo ""

# Count servers with at least one tool
SERVERS_WITH_TOOLS=0
SERVERS_MISSING=0
for server in "${EXPECTED_SERVERS[@]}"; do
    if [[ ${SERVER_COUNTS[$server]} -gt 0 ]]; then
        ((SERVERS_WITH_TOOLS++))
    else
        ((SERVERS_MISSING++))
    fi
done

echo "Server Coverage:"
echo "  - Servers with tools: $SERVERS_WITH_TOOLS / ${#EXPECTED_SERVERS[@]}"
echo "  - Total tools discovered: $TOTAL_TOOLS"
echo ""

if [[ $SERVERS_MISSING -gt 0 ]]; then
    echo -e "${YELLOW}Missing servers:${NC}"
    for server in "${EXPECTED_SERVERS[@]}"; do
        if [[ ${SERVER_COUNTS[$server]} -eq 0 ]]; then
            echo "  - $server"
        fi
    done
    echo ""
fi

# Exit code based on results
if [[ $SERVERS_MISSING -gt 0 ]] || [[ $SMOKE_FAIL -gt 0 ]]; then
    echo -e "${YELLOW}⚠ Validation completed with warnings${NC}"
    exit 1
else
    echo -e "${GREEN}✓ All validations passed${NC}"
    exit 0
fi

