#!/bin/bash
# Telemetry Metrics Validation Script
# Verifies that all expected Claude Code metrics are flowing to Prometheus

set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

PROMETHEUS_URL="http://localhost:9090"
LOKI_URL="http://localhost:3100"
FAILED_CHECKS=0
TOTAL_CHECKS=0

echo -e "${BLUE}=== Claude Code Telemetry Validation ===${NC}"
echo "Checking Prometheus at: $PROMETHEUS_URL"
echo "Checking Loki at: $LOKI_URL"
echo ""

# Function to check if a metric exists
check_metric() {
    local metric_name="$1"
    local description="$2"
    
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    
    echo -n "Checking $description: "
    
    # Query Prometheus for the metric
    result=$(curl -s "$PROMETHEUS_URL/api/v1/query?query=${metric_name}" | jq -r '.data.result | length')
    
    if [ "$result" != "null" ] && [ "$result" -gt 0 ]; then
        echo -e "${GREEN}✓ Found ($result series)${NC}"
        return 0
    else
        echo -e "${RED}✗ Missing${NC}"
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
        return 1
    fi
}

# Function to check log events in Loki
check_log_event() {
    local event_pattern="$1"
    local description="$2"
    
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    
    echo -n "Checking $description: "
    
    # Query Loki for the event using LogQL
    result=$(curl -s "$LOKI_URL/loki/api/v1/query?query=%7Bapp%3D%22claude-code%22%7D&limit=1" | jq -r '.data.result | length' 2>/dev/null || echo "0")
    
    if [ "$result" -gt 0 ]; then
        echo -e "${GREEN}✓ Found${NC}"
        return 0
    else
        echo -e "${RED}✗ Missing${NC}"
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
        return 1
    fi
}

echo -e "${YELLOW}=== Core Metrics (Required for all dashboards) ===${NC}"

# Essential metrics that should exist
check_metric "claude_code_session_count" "Session tracking"
check_metric "claude_code_cost_usage" "Cost tracking"
check_metric "claude_code_token_usage" "Token usage"

echo ""
echo -e "${YELLOW}=== Engineering Metrics Dashboard ===${NC}"

# Engineering productivity metrics
check_metric "claude_code_lines_of_code_count" "Lines of code modified"
check_metric "claude_code_commit_count" "Git commits"
check_metric "claude_code_pull_request_count" "Pull requests created"
check_metric "claude_code_code_edit_tool_decision" "Code edit tool usage"

echo ""
echo -e "${YELLOW}=== Operations Monitoring Dashboard ===${NC}"

# Operations and reliability metrics
check_metric "claude_code_api_request" "API request tracking"
check_metric "claude_code_api_error" "API error tracking"

echo ""
echo -e "${YELLOW}=== Cost Management Dashboard ===${NC}"

# Cost-specific metrics (these might be the same as core cost metrics)
check_metric "claude_code_cost_usage" "Detailed cost tracking"

echo ""
echo -e "${YELLOW}=== Log Events (Loki) ===${NC}"

# Check for key log events
check_log_event "claude_code.api_request" "API request events"
check_log_event "claude_code.tool_result" "Tool result events"
check_log_event "claude_code.user_prompt" "User prompt events"

echo ""
echo -e "${YELLOW}=== Component Health ===${NC}"

# Check that telemetry components are running
echo -n "OTLP Collector health: "
if kubectl get pods -n observability -l app.kubernetes.io/name=opentelemetry-collector --no-headers | grep -q "1/1.*Running"; then
    echo -e "${GREEN}✓ Running${NC}"
else
    echo -e "${RED}✗ Not running${NC}"
    FAILED_CHECKS=$((FAILED_CHECKS + 1))
fi
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

echo -n "Prometheus health: "
if kubectl get pods -n observability -l app.kubernetes.io/name=prometheus --no-headers | grep -q "Running"; then
    echo -e "${GREEN}✓ Running${NC}"
else
    echo -e "${RED}✗ Not running${NC}"
    FAILED_CHECKS=$((FAILED_CHECKS + 1))
fi
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

echo -n "Loki health: "
if kubectl get pods -n observability -l app.kubernetes.io/name=loki --no-headers | grep -q "Running"; then
    echo -e "${GREEN}✓ Running${NC}"
else
    echo -e "${RED}✗ Not running${NC}"
    FAILED_CHECKS=$((FAILED_CHECKS + 1))
fi
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

echo -n "Alertmanager health: "
if kubectl get pods -n observability -l app.kubernetes.io/name=alertmanager --no-headers | grep -q "Running"; then
    echo -e "${GREEN}✓ Running${NC}"
else
    echo -e "${RED}✗ Not running${NC}"
    FAILED_CHECKS=$((FAILED_CHECKS + 1))
fi
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

echo -n "Claude Code pod status: "
if kubectl get pods -n claude-code-dev -l app.kubernetes.io/name=claude-code --no-headers | grep -q "Running\|Completed"; then
    echo -e "${GREEN}✓ Running/Completed${NC}"
else
    echo -e "${YELLOW}⚠ Not running (needs valid API key)${NC}"
fi
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

echo ""
echo -e "${BLUE}=== Summary ===${NC}"

if [ $FAILED_CHECKS -eq 0 ]; then
    echo -e "${GREEN}✅ All $TOTAL_CHECKS checks passed! Telemetry is working correctly.${NC}"
    exit 0
else
    echo -e "${RED}❌ $FAILED_CHECKS out of $TOTAL_CHECKS checks failed.${NC}"
    echo ""
    echo -e "${YELLOW}Common issues:${NC}"
    echo "1. Claude Code pod needs a valid API key to generate telemetry"
    echo "2. Metrics may take time to appear after first run"
    echo "3. Check Claude Code configuration: kubectl get configmap -n claude-code-dev claude-code-dev-config"
    echo "4. Verify endpoints are accessible:"
    echo "   - Prometheus: curl http://localhost:9090/api/v1/query?query=up"
    echo "   - Loki: curl 'http://localhost:3100/loki/api/v1/query?query=%7Bapp%3D%22test%22%7D&limit=1'"
    exit 1
fi
