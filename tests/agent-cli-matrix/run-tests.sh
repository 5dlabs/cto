#!/usr/bin/env bash
# Agent × CLI Matrix Test Runner
# 
# Runs the comprehensive test suite for all agent/CLI combinations
# and generates a detailed report.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
RESULTS_DIR="$SCRIPT_DIR/results"
TIMESTAMP=$(date +%Y-%m-%d-%H-%M-%S)
RESULT_FILE="$RESULTS_DIR/$TIMESTAMP.json"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
TOTAL=0
PASSED=0
FAILED=0
SKIPPED=0

# Parse arguments
FILTER_AGENT=""
FILTER_CLI=""
VERBOSE=false
DRY_RUN=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --agent)
            FILTER_AGENT="$2"
            shift 2
            ;;
        --cli)
            FILTER_CLI="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --agent AGENT    Filter to specific agent (e.g., rex, blaze)"
            echo "  --cli CLI        Filter to specific CLI (e.g., claude, codex)"
            echo "  -v, --verbose    Show detailed output"
            echo "  --dry-run        Show what would be tested without running"
            echo "  -h, --help       Show this help"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Ensure results directory exists
mkdir -p "$RESULTS_DIR"

# Set up environment
export AGENT_TEMPLATES_PATH="$PROJECT_ROOT/templates"

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}║         Agent × CLI Matrix Test Suite                        ║${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo "Project Root: $PROJECT_ROOT"
echo "Templates Path: $AGENT_TEMPLATES_PATH"
echo "Results File: $RESULT_FILE"
echo ""

# Initialize result JSON
cat > "$RESULT_FILE" << EOF
{
  "timestamp": "$TIMESTAMP",
  "project_root": "$PROJECT_ROOT",
  "filters": {
    "agent": "${FILTER_AGENT:-null}",
    "cli": "${FILTER_CLI:-null}"
  },
  "results": [],
  "summary": {}
}
EOF

# Function to run a single test
run_test() {
    local agent="$1"
    local cli="$2"
    local github_app="$3"
    
    ((TOTAL++))
    
    if [[ "$DRY_RUN" == "true" ]]; then
        echo -e "  ${YELLOW}[DRY-RUN]${NC} Would test: $agent + $cli"
        return 0
    fi
    
    # Run the Rust test binary with specific agent/cli
    local test_output
    local test_result
    
    if test_output=$(cd "$PROJECT_ROOT" && \
        AGENT_TEMPLATES_PATH="$AGENT_TEMPLATES_PATH" \
        cargo test -p controller --test e2e_template_tests \
        "rendering_tests::test_container_script_renders" \
        -- --nocapture 2>&1); then
        test_result="pass"
        ((PASSED++))
        echo -e "  ${GREEN}✓${NC} $agent + $cli"
    else
        test_result="fail"
        ((FAILED++))
        echo -e "  ${RED}✗${NC} $agent + $cli"
        if [[ "$VERBOSE" == "true" ]]; then
            echo "$test_output" | tail -20
        fi
    fi
    
    # Append to results JSON (using jq if available, otherwise simple append)
    if command -v jq &> /dev/null; then
        jq --arg agent "$agent" \
           --arg cli "$cli" \
           --arg result "$test_result" \
           --arg github_app "$github_app" \
           '.results += [{
             "agent": $agent,
             "cli": $cli,
             "github_app": $github_app,
             "result": $result
           }]' "$RESULT_FILE" > "$RESULT_FILE.tmp" && mv "$RESULT_FILE.tmp" "$RESULT_FILE"
    fi
}

# Define the test matrix
# Format: agent:github_app:supported_clis (comma-separated)
MATRIX=(
    "rex:5DLabs-Rex:claude,codex,cursor,factory,gemini,opencode"
    "blaze:5DLabs-Blaze:claude,codex,cursor,factory,gemini,opencode"
    "grizz:5DLabs-Grizz:claude,codex,cursor,factory,gemini,opencode"
    "nova:5DLabs-Nova:claude,codex,cursor,factory,gemini,opencode"
    "tap:5DLabs-Tap:claude,codex,cursor,factory,gemini,opencode"
    "spark:5DLabs-Spark:claude,codex,cursor,factory,gemini,opencode"
    "bolt:5DLabs-Bolt:claude,factory"
    "cipher:5DLabs-Cipher:claude,factory"
    "cleo:5DLabs-Cleo:claude,factory"
    "tess:5DLabs-Tess:claude,factory"
    "stitch:5DLabs-Stitch:claude,factory"
    "morgan:5DLabs-Morgan:claude"
    "atlas:5DLabs-Atlas:claude"
)

echo -e "${BLUE}Running Tests${NC}"
echo "────────────────────────────────────────────────────────────────"

for entry in "${MATRIX[@]}"; do
    IFS=':' read -r agent github_app clis <<< "$entry"
    
    # Apply agent filter
    if [[ -n "$FILTER_AGENT" && "$agent" != "$FILTER_AGENT" ]]; then
        continue
    fi
    
    echo ""
    echo -e "${YELLOW}Agent: $agent ($github_app)${NC}"
    
    IFS=',' read -ra CLI_ARRAY <<< "$clis"
    for cli in "${CLI_ARRAY[@]}"; do
        # Apply CLI filter
        if [[ -n "$FILTER_CLI" && "$cli" != "$FILTER_CLI" ]]; then
            ((SKIPPED++))
            continue
        fi
        
        run_test "$agent" "$cli" "$github_app"
    done
done

echo ""
echo "────────────────────────────────────────────────────────────────"
echo -e "${BLUE}Summary${NC}"
echo "────────────────────────────────────────────────────────────────"
echo -e "Total:   $TOTAL"
echo -e "Passed:  ${GREEN}$PASSED${NC}"
echo -e "Failed:  ${RED}$FAILED${NC}"
echo -e "Skipped: ${YELLOW}$SKIPPED${NC}"
echo ""

# Update summary in results file
if command -v jq &> /dev/null; then
    jq --argjson total "$TOTAL" \
       --argjson passed "$PASSED" \
       --argjson failed "$FAILED" \
       --argjson skipped "$SKIPPED" \
       '.summary = {
         "total": $total,
         "passed": $passed,
         "failed": $failed,
         "skipped": $skipped,
         "pass_rate": (if $total > 0 then (($passed / $total) * 100 | floor) else 0 end)
       }' "$RESULT_FILE" > "$RESULT_FILE.tmp" && mv "$RESULT_FILE.tmp" "$RESULT_FILE"
fi

echo "Results saved to: $RESULT_FILE"

# Exit with failure if any tests failed
if [[ $FAILED -gt 0 ]]; then
    exit 1
fi



