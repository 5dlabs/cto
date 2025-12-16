#!/bin/bash
# Live Monitor Server - Provides real-time cluster data for the dashboard
# Usage: ./live-monitor-server.sh [port]

PORT="${1:-8765}"
DATA_FILE="/tmp/cto-monitor-data.json"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}║       CTO Platform Live Monitor Server                      ║${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Function to collect cluster data
collect_data() {
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    
    # Get workflows
    local workflows=$(kubectl get workflows -n cto -o json 2>/dev/null | jq -c '[.items[] | {
        name: .metadata.name,
        status: .status.phase,
        startedAt: .metadata.creationTimestamp,
        finishedAt: .status.finishedAt,
        message: .status.message
    }]' 2>/dev/null || echo "[]")
    
    # Get pods in cto namespace
    local pods=$(kubectl get pods -n cto -o json 2>/dev/null | jq -c '[.items[] | {
        name: .metadata.name,
        status: .status.phase,
        ready: (if .status.containerStatuses then (.status.containerStatuses | map(select(.ready)) | length) else 0 end),
        total: (if .status.containerStatuses then (.status.containerStatuses | length) else 0 end),
        age: .metadata.creationTimestamp,
        labels: .metadata.labels
    }]' 2>/dev/null || echo "[]")
    
    # Get coderuns
    local coderuns=$(kubectl get coderuns -n cto -o json 2>/dev/null | jq -c '[.items[] | {
        name: .metadata.name,
        taskId: .spec.taskId,
        status: .status.phase,
        agent: .metadata.labels["agent"] // "unknown",
        startedAt: .metadata.creationTimestamp
    }]' 2>/dev/null || echo "[]")
    
    # Get PR status (if gh is available)
    local pr_status="null"
    if command -v gh &> /dev/null; then
        pr_status=$(gh pr view 1 --repo 5dlabs/alerthub-lite-simplified-notification-system --json state,mergeable,reviews,statusCheckRollup 2>/dev/null || echo "null")
    fi
    
    # Build JSON response
    cat << EOF
{
    "timestamp": "$timestamp",
    "workflows": $workflows,
    "pods": $pods,
    "coderuns": $coderuns,
    "prStatus": $pr_status,
    "metrics": {
        "activeWorkflows": $(echo "$workflows" | jq '[.[] | select(.status == "Running")] | length'),
        "runningPods": $(echo "$pods" | jq '[.[] | select(.status == "Running")] | length'),
        "completedTasks": $(echo "$coderuns" | jq '[.[] | select(.status == "Succeeded")] | length'),
        "totalTasks": 10
    }
}
EOF
}

# Function to serve data
serve_data() {
    # Update data file
    collect_data > "$DATA_FILE"
    
    # Read and return
    cat "$DATA_FILE"
}

echo -e "${GREEN}✓${NC} Checking cluster connection..."
if ! kubectl cluster-info &>/dev/null; then
    echo -e "${RED}✗${NC} Cannot connect to Kubernetes cluster"
    exit 1
fi
echo -e "${GREEN}✓${NC} Connected to cluster"

echo ""
echo -e "${YELLOW}Starting data collection loop...${NC}"
echo -e "${CYAN}Data will be written to: $DATA_FILE${NC}"
echo ""

# Initial collection
echo -e "${GREEN}✓${NC} Initial data collection..."
serve_data > "$DATA_FILE"
echo -e "${GREEN}✓${NC} Data file created"

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}║  To view data: cat $DATA_FILE | jq .           ║${NC}"
echo -e "${CYAN}║  Dashboard: file://$(pwd)/docs/cto-live-monitor.html         ║${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Continuous update loop
echo -e "${GREEN}Starting continuous monitoring (Ctrl+C to stop)...${NC}"
echo ""

while true; do
    timestamp=$(date +"%H:%M:%S")
    
    # Collect data
    data=$(collect_data)
    echo "$data" > "$DATA_FILE"
    
    # Parse metrics for display
    active_wf=$(echo "$data" | jq -r '.metrics.activeWorkflows')
    running_pods=$(echo "$data" | jq -r '.metrics.runningPods')
    completed=$(echo "$data" | jq -r '.metrics.completedTasks')
    total=$(echo "$data" | jq -r '.metrics.totalTasks')
    
    # Display status line
    printf "\r${CYAN}[%s]${NC} Workflows: ${GREEN}%s${NC} | Pods: ${GREEN}%s${NC} | Tasks: ${YELLOW}%s/%s${NC}    " \
        "$timestamp" "$active_wf" "$running_pods" "$completed" "$total"
    
    sleep 5
done

