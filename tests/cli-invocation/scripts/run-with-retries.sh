#!/usr/bin/env bash
# =========================================================================
# Run With Retries - Controller Simulator
# Mirrors: controller retry behavior with model rotation
# =========================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
CLI_INVOCATION_DIR="$SCRIPT_DIR/.."

# Configuration
AGENT_NAME="${1:-rex}"
CTO_CONFIG="${CTO_CONFIG:-$REPO_ROOT/cto-config.json}"

# Read configuration from cto-config.json
get_config_value() {
    local path="$1"
    local default="$2"
    jq -r "$path // \"$default\"" "$CTO_CONFIG" 2>/dev/null || echo "$default"
}

# Get model rotation config
MODEL_ROTATION_ENABLED=$(get_config_value ".agents.$AGENT_NAME.modelRotation.enabled" "false")
MODEL_ROTATION_MODELS=$(get_config_value ".agents.$AGENT_NAME.modelRotation.models | @json" "[]")

# Get max retries (agent-specific or default)
MAX_RETRIES=$(get_config_value ".agents.$AGENT_NAME.maxRetries" "")
if [ -z "$MAX_RETRIES" ] || [ "$MAX_RETRIES" = "null" ]; then
    MAX_RETRIES=$(get_config_value ".defaults.play.implementationMaxRetries" "3")
fi

# Default models for rotation (haiku → sonnet → opus)
DEFAULT_MODELS='["claude-haiku-3-5-20241022","claude-sonnet-4-5-20250929","claude-opus-4-5-20251101"]'
if [ "$MODEL_ROTATION_MODELS" = "[]" ] || [ "$MODEL_ROTATION_MODELS" = "null" ]; then
    MODEL_ROTATION_MODELS="$DEFAULT_MODELS"
fi

# Parse models into array (portable approach)
MODELS=()
while IFS= read -r model; do
    [ -n "$model" ] && MODELS+=("$model")
done < <(echo "$MODEL_ROTATION_MODELS" | jq -r '.[]' 2>/dev/null)

# Fallback if no models parsed
if [ ${#MODELS[@]} -eq 0 ]; then
    MODELS=("claude-sonnet-4-5-20250929")
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "║ Run With Retries - Controller Simulator"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "Agent: $AGENT_NAME"
echo "Max Retries: $MAX_RETRIES"
echo "Model Rotation: $MODEL_ROTATION_ENABLED"
echo "Models: ${MODELS[*]}"
echo ""

# Track attempt history
declare -a ATTEMPT_HISTORY
TOTAL_START_TIME=$(date +%s)

# Retry loop
for ((attempt=1; attempt<=MAX_RETRIES; attempt++)); do
    echo ""
    echo "────────────────────────────────────────────────────────────────"
    echo "║ Attempt $attempt of $MAX_RETRIES"
    echo "────────────────────────────────────────────────────────────────"
    
    # Select model for this attempt (rotate through models array)
    MODEL_INDEX=$(( (attempt - 1) % ${#MODELS[@]} ))
    CURRENT_MODEL="${MODELS[$MODEL_INDEX]}"
    
    echo "Model: $CURRENT_MODEL"
    echo ""
    
    ATTEMPT_START_TIME=$(date +%s)
    
    # Set environment variables for the container
    export CURRENT_ATTEMPT="$attempt"
    export CURRENT_MODEL="$CURRENT_MODEL"
    export MAX_RETRIES="$MAX_RETRIES"
    
    # Run the container
    cd "$CLI_INVOCATION_DIR"
    
    # Determine which service to run
    SERVICE="${AGENT_NAME}"
    if [ "$AGENT_NAME" = "rex" ]; then
        SERVICE="claude"
    fi
    
    # Run docker compose with the model
    docker compose run --rm \
        -e "CURRENT_ATTEMPT=$attempt" \
        -e "CURRENT_MODEL=$CURRENT_MODEL" \
        -e "MAX_RETRIES=$MAX_RETRIES" \
        "$SERVICE" || true
    
    ATTEMPT_END_TIME=$(date +%s)
    ATTEMPT_DURATION=$((ATTEMPT_END_TIME - ATTEMPT_START_TIME))
    
    # Check acceptance result
    RESULT_FILE="$CLI_INVOCATION_DIR/workspaces/${SERVICE}/.acceptance_result"
    ACCEPTANCE_RESULT="unknown"
    
    if [ -f "$RESULT_FILE" ]; then
        ACCEPTANCE_RESULT=$(cat "$RESULT_FILE")
    fi
    
    # Record attempt in history
    if [ "$ACCEPTANCE_RESULT" = "passed" ]; then
        ATTEMPT_HISTORY+=("✅ Attempt $attempt: $CURRENT_MODEL (passed) - ${ATTEMPT_DURATION}s")
    else
        ATTEMPT_HISTORY+=("❌ Attempt $attempt: $CURRENT_MODEL ($ACCEPTANCE_RESULT) - ${ATTEMPT_DURATION}s")
    fi
    
    echo ""
    echo "Acceptance Result: $ACCEPTANCE_RESULT"
    
    # Check if passed
    if [ "$ACCEPTANCE_RESULT" = "passed" ]; then
        break
    fi
    
    # If not last attempt, continue
    if [ "$attempt" -lt "$MAX_RETRIES" ]; then
        echo "Retrying with next model..."
        sleep 2
    fi
done

# Final summary
TOTAL_END_TIME=$(date +%s)
TOTAL_DURATION=$((TOTAL_END_TIME - TOTAL_START_TIME))

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "║ EXECUTION COMPLETE"
echo "════════════════════════════════════════════════════════════════"

if [ "$ACCEPTANCE_RESULT" = "passed" ]; then
    echo "║ Result: PASSED"
else
    echo "║ Result: FAILED (after $MAX_RETRIES attempts)"
fi

echo "║ Total Iterations: $attempt of $MAX_RETRIES max"
echo "║"
echo "║ Attempt History:"

for entry in "${ATTEMPT_HISTORY[@]}"; do
    echo "║   $entry"
done

echo "║"
echo "║ Total Duration: ${TOTAL_DURATION}s"
echo "════════════════════════════════════════════════════════════════"
echo ""

# Exit with appropriate code
if [ "$ACCEPTANCE_RESULT" = "passed" ]; then
    exit 0
else
    exit 1
fi
