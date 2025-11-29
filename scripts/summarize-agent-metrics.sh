#!/bin/bash
# Summarize Agent Metrics
# 
# This script aggregates metrics from /workspace/.metrics/ and generates
# a summary report for cost analysis and prompt optimization insights.
#
# Usage:
#   ./summarize-agent-metrics.sh [metrics_dir]
#
# Arguments:
#   metrics_dir - Directory containing metrics JSON files (default: /workspace/.metrics)
#
# Output:
#   - Summary report to stdout
#   - JSON summary to metrics_dir/summary.json

set -euo pipefail

METRICS_DIR="${1:-/workspace/.metrics}"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if jq is available
if ! command -v jq >/dev/null 2>&1; then
    echo "Error: jq is required but not installed" >&2
    exit 1
fi

# Check if metrics directory exists
if [ ! -d "$METRICS_DIR" ]; then
    echo "Error: Metrics directory not found: $METRICS_DIR" >&2
    echo "No metrics have been collected yet." >&2
    exit 1
fi

# Count metrics files
FILE_COUNT=$(find "$METRICS_DIR" -name "task-*.json" -type f 2>/dev/null | wc -l)

if [ "$FILE_COUNT" -eq 0 ]; then
    echo "No metrics files found in $METRICS_DIR"
    exit 0
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "                 AGENT METRICS SUMMARY                          "
echo "════════════════════════════════════════════════════════════════"
echo ""

# Get date range
FIRST_DATE=$(find "$METRICS_DIR" -name "task-*.json" -type f -exec jq -r '.timestamp' {} \; 2>/dev/null | sort | head -1)
LAST_DATE=$(find "$METRICS_DIR" -name "task-*.json" -type f -exec jq -r '.timestamp' {} \; 2>/dev/null | sort | tail -1)

echo -e "${BLUE}Period:${NC} $FIRST_DATE to $LAST_DATE"
echo -e "${BLUE}Tasks:${NC} $FILE_COUNT"
echo ""

# Aggregate totals
TOTAL_INPUT=0
TOTAL_OUTPUT=0
TOTAL_TOKENS=0
SUCCESS_COUNT=0
FAIL_COUNT=0

# Per-CLI aggregation
declare -A CLI_INPUT
declare -A CLI_OUTPUT
declare -A CLI_COUNT
declare -A CLI_SUCCESS

# Per-Agent aggregation
declare -A AGENT_INPUT
declare -A AGENT_OUTPUT
declare -A AGENT_COUNT
declare -A AGENT_SUCCESS

# Process each metrics file
while IFS= read -r file; do
    # Parse JSON
    TOKENS_IN=$(jq -r '.tokens.input // 0' "$file" 2>/dev/null)
    TOKENS_OUT=$(jq -r '.tokens.output // 0' "$file" 2>/dev/null)
    CLI=$(jq -r '.cli // "unknown"' "$file" 2>/dev/null)
    AGENT=$(jq -r '.agent // "unknown"' "$file" 2>/dev/null)
    SUCCESS=$(jq -r '.success // 0' "$file" 2>/dev/null)
    
    # Skip if parsing failed
    [[ "$TOKENS_IN" =~ ^[0-9]+$ ]] || TOKENS_IN=0
    [[ "$TOKENS_OUT" =~ ^[0-9]+$ ]] || TOKENS_OUT=0
    [[ "$SUCCESS" =~ ^[0-9]+$ ]] || SUCCESS=0
    
    # Update totals
    TOTAL_INPUT=$((TOTAL_INPUT + TOKENS_IN))
    TOTAL_OUTPUT=$((TOTAL_OUTPUT + TOKENS_OUT))
    TOTAL_TOKENS=$((TOTAL_INPUT + TOTAL_OUTPUT))
    
    if [ "$SUCCESS" -eq 1 ]; then
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
    else
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
    
    # Update CLI stats
    CLI_INPUT[$CLI]=$((${CLI_INPUT[$CLI]:-0} + TOKENS_IN))
    CLI_OUTPUT[$CLI]=$((${CLI_OUTPUT[$CLI]:-0} + TOKENS_OUT))
    CLI_COUNT[$CLI]=$((${CLI_COUNT[$CLI]:-0} + 1))
    CLI_SUCCESS[$CLI]=$((${CLI_SUCCESS[$CLI]:-0} + SUCCESS))
    
    # Update Agent stats
    AGENT_INPUT[$AGENT]=$((${AGENT_INPUT[$AGENT]:-0} + TOKENS_IN))
    AGENT_OUTPUT[$AGENT]=$((${AGENT_OUTPUT[$AGENT]:-0} + TOKENS_OUT))
    AGENT_COUNT[$AGENT]=$((${AGENT_COUNT[$AGENT]:-0} + 1))
    AGENT_SUCCESS[$AGENT]=$((${AGENT_SUCCESS[$AGENT]:-0} + SUCCESS))
    
done < <(find "$METRICS_DIR" -name "task-*.json" -type f 2>/dev/null)

# Calculate overall stats
TOTAL_TOKENS=$((TOTAL_INPUT + TOTAL_OUTPUT))

# Cost estimation (Claude Sonnet 4 pricing: $3/1M input, $15/1M output)
INPUT_COST=$(echo "scale=4; $TOTAL_INPUT * 3 / 1000000" | bc 2>/dev/null || echo "0")
OUTPUT_COST=$(echo "scale=4; $TOTAL_OUTPUT * 15 / 1000000" | bc 2>/dev/null || echo "0")
TOTAL_COST=$(echo "scale=2; $INPUT_COST + $OUTPUT_COST" | bc 2>/dev/null || echo "0")

# Success rate
if [ "$FILE_COUNT" -gt 0 ]; then
    SUCCESS_RATE=$(echo "scale=1; $SUCCESS_COUNT * 100 / $FILE_COUNT" | bc 2>/dev/null || echo "0")
else
    SUCCESS_RATE="0"
fi

echo "──────────────────────────────────────────────────────────────"
echo "                     TOKEN SUMMARY                            "
echo "──────────────────────────────────────────────────────────────"
printf "%-20s %15s\n" "Total Input Tokens:" "$(printf "%'d" $TOTAL_INPUT)"
printf "%-20s %15s\n" "Total Output Tokens:" "$(printf "%'d" $TOTAL_OUTPUT)"
printf "%-20s %15s\n" "Total Tokens:" "$(printf "%'d" $TOTAL_TOKENS)"
echo ""
printf "%-20s %15s\n" "Estimated Cost:" "\$$TOTAL_COST"
printf "%-20s %15s\n" "Success Rate:" "${SUCCESS_RATE}%"
echo ""

echo "──────────────────────────────────────────────────────────────"
echo "                     BY CLI                                   "
echo "──────────────────────────────────────────────────────────────"
printf "%-12s %10s %10s %10s %8s\n" "CLI" "Input" "Output" "Tasks" "Success"
printf "%-12s %10s %10s %10s %8s\n" "────" "─────" "──────" "─────" "───────"

for cli in "${!CLI_COUNT[@]}"; do
    if [ "${CLI_COUNT[$cli]}" -gt 0 ]; then
        cli_success_rate=$(echo "scale=0; ${CLI_SUCCESS[$cli]} * 100 / ${CLI_COUNT[$cli]}" | bc 2>/dev/null || echo "0")
    else
        cli_success_rate="0"
    fi
    printf "%-12s %10s %10s %10s %7s%%\n" \
        "$cli" \
        "$(printf "%'d" "${CLI_INPUT[$cli]}")" \
        "$(printf "%'d" "${CLI_OUTPUT[$cli]}")" \
        "${CLI_COUNT[$cli]}" \
        "$cli_success_rate"
done
echo ""

echo "──────────────────────────────────────────────────────────────"
echo "                     BY AGENT                                 "
echo "──────────────────────────────────────────────────────────────"
printf "%-12s %10s %10s %10s %8s\n" "Agent" "Input" "Output" "Tasks" "Success"
printf "%-12s %10s %10s %10s %8s\n" "─────" "─────" "──────" "─────" "───────"

for agent in "${!AGENT_COUNT[@]}"; do
    if [ "${AGENT_COUNT[$agent]}" -gt 0 ]; then
        agent_success_rate=$(echo "scale=0; ${AGENT_SUCCESS[$agent]} * 100 / ${AGENT_COUNT[$agent]}" | bc 2>/dev/null || echo "0")
    else
        agent_success_rate="0"
    fi
    printf "%-12s %10s %10s %10s %7s%%\n" \
        "$agent" \
        "$(printf "%'d" "${AGENT_INPUT[$agent]}")" \
        "$(printf "%'d" "${AGENT_OUTPUT[$agent]}")" \
        "${AGENT_COUNT[$agent]}" \
        "$agent_success_rate"
done
echo ""

echo "──────────────────────────────────────────────────────────────"
echo "                CONTEXT UTILIZATION                           "
echo "──────────────────────────────────────────────────────────────"

# Calculate average context utilization
AVG_CONTEXT=$(find "$METRICS_DIR" -name "task-*.json" -type f -exec jq -r '.context_pct // 0' {} \; 2>/dev/null | awk '{sum+=$1; count++} END {if(count>0) printf "%.1f", sum/count; else print "0"}')
MAX_CONTEXT=$(find "$METRICS_DIR" -name "task-*.json" -type f -exec jq -r '.context_pct // 0' {} \; 2>/dev/null | sort -n | tail -1)

printf "%-25s %10s%%\n" "Average Context Used:" "$AVG_CONTEXT"
printf "%-25s %10s%%\n" "Maximum Context Used:" "$MAX_CONTEXT"

if (( $(echo "$AVG_CONTEXT > 70" | bc -l 2>/dev/null || echo 0) )); then
    echo -e "\n${YELLOW}⚠️  Warning: Average context utilization > 70%${NC}"
    echo "   Consider optimizing prompts to reduce context consumption."
fi

echo ""
echo "════════════════════════════════════════════════════════════════"

# Generate JSON summary
SUMMARY_FILE="$METRICS_DIR/summary.json"
cat > "$SUMMARY_FILE" <<EOF
{
  "generated_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "period": {
    "start": "$FIRST_DATE",
    "end": "$LAST_DATE"
  },
  "totals": {
    "tasks": $FILE_COUNT,
    "input_tokens": $TOTAL_INPUT,
    "output_tokens": $TOTAL_OUTPUT,
    "total_tokens": $TOTAL_TOKENS,
    "success_count": $SUCCESS_COUNT,
    "fail_count": $FAIL_COUNT,
    "success_rate": $SUCCESS_RATE,
    "estimated_cost_usd": $TOTAL_COST
  },
  "context": {
    "average_pct": $AVG_CONTEXT,
    "max_pct": ${MAX_CONTEXT:-0}
  }
}
EOF

echo -e "${GREEN}✓ Summary saved to: $SUMMARY_FILE${NC}"

