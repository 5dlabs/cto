#!/usr/bin/env bash
# Validate that each agent can access their configured tools via MCP
#
# This script tests the tool routing for all 12 agents by attempting
# to list available tools for each agent's configuration.
set -euo pipefail

echo "=== Agent Tool Routing Validation ==="
echo ""

# Agent configurations from cto-config.json
declare -A AGENT_TOOLS=(
    ["morgan"]="context7_resolve_library_id firecrawl_scrape openmemory_query"
    ["rex"]="github_create_pull_request github_push_files context7_get_library_docs"
    ["blaze"]="github_create_pull_request shadcn_get_component context7_get_library_docs"
    ["grizz"]="github_create_pull_request github_push_files context7_get_library_docs"
    ["nova"]="github_create_pull_request github_push_files context7_get_library_docs"
    ["tap"]="github_create_pull_request github_push_files context7_get_library_docs"
    ["spark"]="github_create_pull_request github_push_files context7_get_library_docs"
    ["cleo"]="github_get_pull_request github_create_pull_request_review"
    ["cipher"]="github_list_code_scanning_alerts github_list_secret_scanning_alerts"
    ["tess"]="kubernetes_get_pods github_get_pull_request_status"
    ["atlas"]="github_merge_pull_request github_update_pull_request_branch"
    ["bolt"]="kubernetes_get_pods argocd_list_applications grafana_query_prometheus"
)

PASS_COUNT=0
FAIL_COUNT=0

for agent in "${!AGENT_TOOLS[@]}"; do
    echo "--- Validating $agent ---"
    tools="${AGENT_TOOLS[$agent]}"
    
    for tool in $tools; do
        # Check if tool exists in tool catalog
        if grep -q "\"$tool\"" /tools-catalog/tool-catalog.json 2>/dev/null; then
            echo "  ✓ $tool"
            ((PASS_COUNT++))
        else
            echo "  ✗ $tool (not in catalog)"
            ((FAIL_COUNT++))
        fi
    done
    echo ""
done

echo "=== Summary ==="
echo "Passed: $PASS_COUNT"
echo "Failed: $FAIL_COUNT"

if [ "$FAIL_COUNT" -gt 0 ]; then
    echo ""
    echo "WARNING: Some tools are missing from the catalog."
    echo "This may indicate routing issues for affected agents."
    exit 1
else
    echo ""
    echo "All agent tools validated successfully!"
fi
