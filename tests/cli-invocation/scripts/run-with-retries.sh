#!/usr/bin/env bash
# =============================================================================
# Run With Retries - Controller Simulator
# =============================================================================
#
# This script mimics the Kubernetes controller's retry behavior for local testing.
# It runs the container multiple times with model rotation until acceptance criteria
# pass or max retries is reached.
#
# Usage:
#   ./scripts/run-with-retries.sh [agent]
#
# Examples:
#   ./scripts/run-with-retries.sh          # Default: bolt
#   ./scripts/run-with-retries.sh bolt     # Bolt agent
#   ./scripts/run-with-retries.sh claude   # Claude/Rex agent
#
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_DIR="${SCRIPT_DIR}/.."
CTO_CONFIG="${TEST_DIR}/../../cto-config.json"
START_TIME=$(date +%s)

AGENT="${1:-bolt}"

echo "════════════════════════════════════════════════════════════════"
echo "║ Controller Simulator - ${AGENT} Agent"
echo "════════════════════════════════════════════════════════════════"

# -----------------------------------------------------------------------------
# Read Configuration from cto-config.json (mirrors MCP server behavior)
# -----------------------------------------------------------------------------
if [[ ! -f "${CTO_CONFIG}" ]]; then
  echo "❌ Config not found: ${CTO_CONFIG}" >&2
  exit 1
fi

MAX_RETRIES=$(jq -r ".agents.${AGENT}.maxRetries // .defaults.play.infrastructureMaxRetries // 3" "$CTO_CONFIG")
MODEL_ROTATION_ENABLED=$(jq -r ".agents.${AGENT}.modelRotation.enabled // false" "$CTO_CONFIG")
MODEL_ROTATION=$(jq -c ".agents.${AGENT}.modelRotation.models // []" "$CTO_CONFIG")
DEFAULT_MODEL=$(jq -r ".agents.${AGENT}.model // \"claude-sonnet-4-20250514\"" "$CTO_CONFIG")

echo "║ Config source: ${CTO_CONFIG}"
echo "║ Max retries: ${MAX_RETRIES}"
echo "║ Model rotation: ${MODEL_ROTATION_ENABLED}"
if [ "$MODEL_ROTATION_ENABLED" = "true" ]; then
  echo "║ Models: ${MODEL_ROTATION}"
else
  echo "║ Model: ${DEFAULT_MODEL}"
fi
echo "════════════════════════════════════════════════════════════════"

# -----------------------------------------------------------------------------
# Helper Functions
# -----------------------------------------------------------------------------
get_model_for_attempt() {
  local attempt=$1
  if [ "$MODEL_ROTATION_ENABLED" = "true" ]; then
    local model_count
    model_count=$(echo "$MODEL_ROTATION" | jq length)
    local index=$(( (attempt - 1) % model_count ))
    echo "$MODEL_ROTATION" | jq -r ".[$index]"
  else
    echo "$DEFAULT_MODEL"
  fi
}

# Tracking arrays for summary
declare -a ATTEMPT_MODELS=()
declare -a ATTEMPT_RESULTS=()
declare -a ATTEMPT_DURATIONS=()
declare -a ATTEMPT_COSTS=()

# -----------------------------------------------------------------------------
# Main Retry Loop
# -----------------------------------------------------------------------------
CURRENT_ATTEMPT=1
FINAL_RESULT="FAILED"

while [ "$CURRENT_ATTEMPT" -le "$MAX_RETRIES" ]; do
  ATTEMPT_START=$(date +%s)
  CURRENT_MODEL=$(get_model_for_attempt "$CURRENT_ATTEMPT")
  
  echo ""
  echo "═══════════════════════════════════════════════"
  echo "║ Attempt ${CURRENT_ATTEMPT}/${MAX_RETRIES} | Model: ${CURRENT_MODEL}"
  echo "═══════════════════════════════════════════════"
  
  # Clean workspace for fresh attempt
  rm -rf "${TEST_DIR}/workspaces/${AGENT}"/*
  rm -f "${TEST_DIR}/workspaces/${AGENT}/.acceptance_result"
  
  # Run container with current attempt parameters
  # The sidecar should already be running or will be started separately
  cd "${TEST_DIR}"
  docker compose run --rm \
    -e CURRENT_ATTEMPT="$CURRENT_ATTEMPT" \
    -e EXECUTION_MAX_RETRIES="$MAX_RETRIES" \
    -e MODEL_ROTATION_MODELS="$MODEL_ROTATION" \
    -e CURRENT_MODEL="$CURRENT_MODEL" \
    "${AGENT}" || true
  
  ATTEMPT_END=$(date +%s)
  ATTEMPT_DURATION=$((ATTEMPT_END - ATTEMPT_START))
  ATTEMPT_DURATIONS+=("$ATTEMPT_DURATION")
  ATTEMPT_MODELS+=("$CURRENT_MODEL")
  
  # Check acceptance result
  RESULT_FILE="${TEST_DIR}/workspaces/${AGENT}/.acceptance_result"
  if [[ -f "$RESULT_FILE" ]]; then
    RESULT=$(cat "$RESULT_FILE")
  else
    RESULT="failed"
  fi
  ATTEMPT_RESULTS+=("$RESULT")
  
  # Try to extract cost from stream (if available)
  COST="unknown"
  if [[ -f "${TEST_DIR}/workspaces/${AGENT}/stream.jsonl" ]]; then
    # Look for cost in the last line or summary
    COST=$(tail -1 "${TEST_DIR}/workspaces/${AGENT}/stream.jsonl" 2>/dev/null | jq -r '.cost // "unknown"' 2>/dev/null || echo "unknown")
  fi
  ATTEMPT_COSTS+=("$COST")
  
  echo ""
  echo "║ Attempt ${CURRENT_ATTEMPT} Result: ${RESULT} (${ATTEMPT_DURATION}s)"
  
  if [ "$RESULT" = "passed" ]; then
    FINAL_RESULT="PASSED"
    echo "║ ✅ Acceptance criteria met!"
    break
  fi
  
  if [ "$CURRENT_ATTEMPT" -lt "$MAX_RETRIES" ]; then
    echo "║ ❌ Failed, scheduling retry..."
  else
    echo "║ ❌ Failed, no more retries."
  fi
  
  CURRENT_ATTEMPT=$((CURRENT_ATTEMPT + 1))
done

# -----------------------------------------------------------------------------
# Final Summary
# -----------------------------------------------------------------------------
END_TIME=$(date +%s)
TOTAL_DURATION=$((END_TIME - START_TIME))
TOTAL_ATTEMPTS=${#ATTEMPT_RESULTS[@]}

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "║ EXECUTION COMPLETE"
echo "════════════════════════════════════════════════════════════════"
echo "║ Result: ${FINAL_RESULT}"
echo "║ Total Iterations: ${TOTAL_ATTEMPTS} of ${MAX_RETRIES} max"
echo "║"
echo "║ Attempt History:"
for i in "${!ATTEMPT_RESULTS[@]}"; do
  status="${ATTEMPT_RESULTS[$i]}"
  model="${ATTEMPT_MODELS[$i]}"
  duration="${ATTEMPT_DURATIONS[$i]}"
  cost="${ATTEMPT_COSTS[$i]}"
  icon=$( [ "$status" = "passed" ] && echo "✅" || echo "❌" )
  echo "║   ${icon} Attempt $((i+1)): ${model}"
  echo "║      Status: ${status} | Duration: ${duration}s | Cost: ${cost}"
done
echo "║"
echo "║ Total Duration: $((TOTAL_DURATION / 60))m $((TOTAL_DURATION % 60))s"
echo "════════════════════════════════════════════════════════════════"

# Exit with appropriate code
[ "$FINAL_RESULT" = "PASSED" ] && exit 0 || exit 1
