#!/bin/bash
# =============================================================================
# Ralph Runner Entrypoint
# =============================================================================
# Persistent loop runner for CTO lifecycle tests.
# Runs ralph-cto.sh in a loop until play completes, then notifies Linear.
# =============================================================================

set -euo pipefail

# Configuration from environment
RALPH_CONFIG="${RALPH_CONFIG:-/ralph/config/ralph-cto.json}"
RALPH_STATE="${RALPH_STATE:-/ralph/state/ralph-cto.state.json}"
RALPH_PROGRESS="${RALPH_PROGRESS:-/ralph/state/progress.txt}"
RALPH_REPORT="${RALPH_REPORT:-/ralph/state/report.json}"
SLEEP_INTERVAL="${SLEEP_INTERVAL:-60}"
MAX_FAILURES="${MAX_FAILURES:-10}"
LINEAR_API_KEY="${LINEAR_API_KEY:-}"
LINEAR_ISSUE_ID="${LINEAR_ISSUE_ID:-}"
NOTIFY_ON_SUCCESS="${NOTIFY_ON_SUCCESS:-true}"
NOTIFY_ON_FAILURE="${NOTIFY_ON_FAILURE:-true}"

# Working directory for ralph (must have git repo access)
WORKSPACE="${WORKSPACE:-/workspace}"

log() {
  echo "[$(date -u '+%Y-%m-%dT%H:%M:%SZ')] $*"
}

# Post a comment to Linear
post_linear_comment() {
  local body="$1"
  local issue_id="${2:-$LINEAR_ISSUE_ID}"
  
  if [[ -z "$LINEAR_API_KEY" || -z "$issue_id" ]]; then
    log "WARNING: Cannot post to Linear - missing API key or issue ID"
    return 0
  fi
  
  # Escape the body for JSON
  local escaped_body
  escaped_body=$(echo "$body" | jq -Rs .)
  
  local query
  query=$(cat <<EOF
{
  "query": "mutation CreateComment(\$issueId: String!, \$body: String!) { commentCreate(input: { issueId: \$issueId, body: \$body }) { success comment { id } } }",
  "variables": {
    "issueId": "$issue_id",
    "body": $escaped_body
  }
}
EOF
)
  
  local response
  response=$(curl -s -X POST https://api.linear.app/graphql \
    -H "Authorization: $LINEAR_API_KEY" \
    -H "Content-Type: application/json" \
    -d "$query")
  
  if echo "$response" | jq -e '.data.commentCreate.success == true' >/dev/null 2>&1; then
    log "Posted comment to Linear issue $issue_id"
    return 0
  else
    log "WARNING: Failed to post Linear comment: $response"
    return 1
  fi
}

# Check if play is complete
check_completion() {
  if [[ ! -f "$RALPH_STATE" ]]; then
    return 1
  fi
  
  local phase
  phase=$(jq -r '.phase // "unknown"' "$RALPH_STATE" 2>/dev/null || echo "unknown")
  
  if [[ "$phase" == "done" || "$phase" == "complete" ]]; then
    return 0
  fi
  
  return 1
}

# Get current phase from state
get_phase() {
  if [[ -f "$RALPH_STATE" ]]; then
    jq -r '.phase // "unknown"' "$RALPH_STATE" 2>/dev/null || echo "unknown"
  else
    echo "unknown"
  fi
}

# Get completed objectives
get_completed_objectives() {
  if [[ -f "$RALPH_STATE" ]]; then
    jq -r '.completedObjectives // [] | join(", ")' "$RALPH_STATE" 2>/dev/null || echo "none"
  else
    echo "none"
  fi
}

# Build completion summary
build_summary() {
  local status="$1"
  local phase
  local objectives
  
  phase=$(get_phase)
  objectives=$(get_completed_objectives)
  
  cat <<EOF
## Ralph Runner - $status

**Phase**: $phase
**Completed Objectives**: $objectives
**Timestamp**: $(date -u '+%Y-%m-%dT%H:%M:%SZ')

### Progress Summary
$(tail -50 "$RALPH_PROGRESS" 2>/dev/null || echo "No progress file found")
EOF
}

# Main loop
main() {
  log "Starting Ralph Runner"
  log "Config: $RALPH_CONFIG"
  log "State: $RALPH_STATE"
  log "Workspace: $WORKSPACE"
  log "Sleep interval: ${SLEEP_INTERVAL}s"
  log "Max failures: $MAX_FAILURES"
  
  # Verify workspace exists
  if [[ ! -d "$WORKSPACE" ]]; then
    log "ERROR: Workspace directory not found: $WORKSPACE"
    exit 1
  fi
  
  cd "$WORKSPACE"
  
  # Ensure state directory is writable
  mkdir -p "$(dirname "$RALPH_STATE")"
  
  # Fix permissions on state files (may have restrictive perms from git)
  if [[ -f "$RALPH_STATE" ]]; then
    chmod u+w "$RALPH_STATE" 2>/dev/null || sudo chmod u+w "$RALPH_STATE" 2>/dev/null || true
  fi
  if [[ -f "$RALPH_PROGRESS" ]]; then
    chmod u+w "$RALPH_PROGRESS" 2>/dev/null || sudo chmod u+w "$RALPH_PROGRESS" 2>/dev/null || true
  fi
  
  # Initialize state if needed
  if [[ ! -f "$RALPH_STATE" ]]; then
    log "Initializing state file"
    echo '{"phase": "intake", "attempts": {}, "completedObjectives": [], "attendedCompleted": 2}' > "$RALPH_STATE"
  fi
  
  # Initialize progress if needed
  if [[ ! -f "$RALPH_PROGRESS" ]]; then
    mkdir -p "$(dirname "$RALPH_PROGRESS")"
    echo "# Ralph Runner Progress Log" > "$RALPH_PROGRESS"
    echo "# Started: $(date -u '+%Y-%m-%dT%H:%M:%SZ')" >> "$RALPH_PROGRESS"
  fi
  
  local consecutive_failures=0
  local iteration=0
  
  while true; do
    iteration=$((iteration + 1))
    log "=== Iteration $iteration ==="
    
    # Check if already complete
    if check_completion; then
      log "Play lifecycle COMPLETE!"
      
      if [[ "$NOTIFY_ON_SUCCESS" == "true" ]]; then
        local summary
        summary=$(build_summary "Play Completed Successfully")
        post_linear_comment "$summary" || true
      fi
      
      log "Exiting successfully"
      exit 0
    fi
    
    # Run ralph-cto.sh
    log "Running ralph-cto.sh..."
    local ralph_exit=0
    
    # Export config paths for ralph
    export RALPH_CONFIG
    export RALPH_STATE
    export RALPH_PROGRESS
    export RALPH_REPORT
    
    if /usr/local/bin/ralph-cto.sh 2>&1 | tee -a "$RALPH_PROGRESS"; then
      ralph_exit=0
      consecutive_failures=0
      log "Ralph iteration completed successfully"
    else
      ralph_exit=$?
      consecutive_failures=$((consecutive_failures + 1))
      log "Ralph iteration failed with exit code $ralph_exit (consecutive failures: $consecutive_failures)"
    fi
    
    # Check for too many failures
    if [[ $consecutive_failures -ge $MAX_FAILURES ]]; then
      log "ERROR: Too many consecutive failures ($consecutive_failures >= $MAX_FAILURES)"
      
      if [[ "$NOTIFY_ON_FAILURE" == "true" ]]; then
        local summary
        summary=$(build_summary "FAILED - Too Many Consecutive Failures")
        post_linear_comment "$summary" || true
      fi
      
      log "Exiting with failure"
      exit 1
    fi
    
    # Check completion after run
    if check_completion; then
      log "Play lifecycle COMPLETE after iteration $iteration!"
      
      if [[ "$NOTIFY_ON_SUCCESS" == "true" ]]; then
        local summary
        summary=$(build_summary "Play Completed Successfully")
        post_linear_comment "$summary" || true
      fi
      
      log "Exiting successfully"
      exit 0
    fi
    
    # Sleep before next iteration
    log "Sleeping for ${SLEEP_INTERVAL}s before next iteration..."
    sleep "$SLEEP_INTERVAL"
  done
}

# Handle signals gracefully
trap 'log "Received SIGTERM, exiting..."; exit 0' SIGTERM
trap 'log "Received SIGINT, exiting..."; exit 0' SIGINT

main "$@"
