#!/bin/bash
# Run the Remediation Agent (Claude) for PR merge failure remediation
# Usage: ./run-remediation.sh [--no-wait]
#
# By default waits for merger to be running before starting
# Use --no-wait to skip the wait

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PROMPT_FILE="$SCRIPT_DIR/remediation-prompt.md"
COORD_FILE="$SCRIPT_DIR/ralph-coordination.json"
NO_WAIT=false
POLL_INTERVAL=10  # Check for issues every 10 seconds

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

log() {
  echo -e "${CYAN}[$(date '+%Y-%m-%d %H:%M:%S')]${NC} $1"
}

error() {
  echo -e "${RED}[ERROR]${NC} $1" >&2
}

success() {
  echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
  echo -e "${YELLOW}[WARN]${NC} $1"
}

# Parse arguments
parse_args() {
  while [[ $# -gt 0 ]]; do
    case $1 in
      --no-wait)
        NO_WAIT=true
        shift
        ;;
      *)
        error "Unknown option: $1"
        exit 1
        ;;
    esac
  done
}

# Check prerequisites
check_prereqs() {
  log "Checking prerequisites..."
  
  # Check Claude CLI
  if ! command -v claude &> /dev/null; then
    error "Claude CLI not found. Install with: npm install -g @anthropic-ai/claude-code"
    exit 1
  fi
  
  # Check GitHub CLI
  if ! command -v gh &> /dev/null; then
    error "GitHub CLI not found. Install with: brew install gh"
    exit 1
  fi
  
  # Check GitHub authentication
  if ! gh auth status &> /dev/null; then
    error "GitHub not authenticated. Run: gh auth login"
    exit 1
  fi
  
  # Check git
  if ! command -v git &> /dev/null; then
    error "Git not found"
    exit 1
  fi
  
  # Check jq
  if ! command -v jq &> /dev/null; then
    error "jq not found. Install with: brew install jq"
    exit 1
  fi
  
  # Check coordination file
  if [[ ! -f "$COORD_FILE" ]]; then
    error "Coordination file not found: $COORD_FILE"
    error "Run the merger first to initialize state"
    exit 1
  fi
  
  success "All prerequisites met"
}

# Wait for merger to be running
wait_for_merger() {
  if [ "$NO_WAIT" = true ]; then
    log "Skipping merger wait (--no-wait)"
    return 0
  fi
  
  log "Waiting for merger agent to start..."
  local max_wait=300  # 5 minutes
  local waited=0
  
  while [ $waited -lt $max_wait ]; do
    local status=$(jq -r '.merger.status' "$COORD_FILE" 2>/dev/null || echo "unknown")
    
    if [ "$status" = "running" ]; then
      success "Merger is running, starting remediation"
      return 0
    fi
    
    log "Merger status: $status (waiting...)"
    sleep 5
    waited=$((waited + 5))
  done
  
  error "Timeout waiting for merger to start"
  exit 1
}

# Update coordination state
update_coord() {
  local key="$1"
  local value="$2"
  local tmp=$(mktemp)
  jq "$key = $value" "$COORD_FILE" > "$tmp" && mv "$tmp" "$COORD_FILE"
}

# Initialize remediation
init_remediation() {
  local now=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  
  # Initialize issueQueue if it doesn't exist
  if ! jq -e '.issueQueue' "$COORD_FILE" > /dev/null 2>&1; then
    update_coord '.issueQueue' '[]'
  fi
  
  update_coord '.remediation.status' '"running"'
  update_coord '.remediation.lastCheck' "\"$now\""
  update_coord '.remediation.pid' "$$"
  
  log "Remediation initialized"
}

# Get next pending issue
get_next_pending_issue() {
  jq -r '[.issueQueue[]? | select(.status == "pending")] | sort_by(.timestamp) | first | .id // empty' "$COORD_FILE" 2>/dev/null || echo ""
}

# Claim an issue
claim_issue() {
  local issue_id="$1"
  local now=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  local tmp=$(mktemp)
  
  jq --arg id "$issue_id" \
     --arg now "$now" \
     '(.issueQueue[]? | select(.id == $id)).status = "claimed" |
      (.issueQueue[]? | select(.id == $id)).claimedAt = $now |
      (.issueQueue[]? | select(.id == $id)).claimedBy = "remediation"' \
     "$COORD_FILE" > "$tmp" && mv "$tmp" "$COORD_FILE"
  
  log "Claimed issue: $issue_id"
}

# Resolve an issue
resolve_issue() {
  local issue_id="$1"
  local resolution="$2"
  local now=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  local tmp=$(mktemp)
  
  jq --arg id "$issue_id" \
     --arg now "$now" \
     --arg resolution "$resolution" \
     '(.issueQueue[]? | select(.id == $id)).status = "resolved" |
      (.issueQueue[]? | select(.id == $id)).resolvedAt = $now |
      (.issueQueue[]? | select(.id == $id)).resolution = $resolution' \
     "$COORD_FILE" > "$tmp" && mv "$tmp" "$COORD_FILE"
  
  log "Resolved issue: $issue_id"
}

# Fail an issue
fail_issue() {
  local issue_id="$1"
  local reason="$2"
  local now=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  local tmp=$(mktemp)
  
  jq --arg id "$issue_id" \
     --arg now "$now" \
     --arg reason "$reason" \
     '(.issueQueue[]? | select(.id == $id)).status = "failed" |
      (.issueQueue[]? | select(.id == $id)).failedAt = $now |
      (.issueQueue[]? | select(.id == $id)).failureReason = $reason |
      (.issueQueue[]? | select(.id == $id)).retryCount = ((.issueQueue[]? | select(.id == $id)).retryCount // 0) + 1' \
     "$COORD_FILE" > "$tmp" && mv "$tmp" "$COORD_FILE"
  
  log "Failed issue: $issue_id - $reason"
}

# Get issue details
get_issue_details() {
  local issue_id="$1"
  jq -c --arg id "$issue_id" '.issueQueue[]? | select(.id == $id)' "$COORD_FILE" 2>/dev/null || echo "{}"
}

# Run remediation for an issue
remediate_issue() {
  local issue_id="$1"
  local issue_details=$(get_issue_details "$issue_id")
  
  if [ -z "$issue_details" ] || [ "$issue_details" = "{}" ]; then
    warn "Issue $issue_id not found, skipping"
    return 1
  fi
  
  log "Remediating issue: $issue_id"
  
  # Build the prompt
  local prompt_content=$(cat "$PROMPT_FILE")
  local prompt="$prompt_content

---

## Current Issue

\`\`\`json
$issue_details
\`\`\`

### Coordination State
\`\`\`json
$(cat "$COORD_FILE" | jq .)
\`\`\`

### Your Task

1. **Investigate** the issue described above
2. **Fix** the root cause
3. **Verify** the fix works
4. **Resolve** the issue in the coordination file
5. **Log** your actions to progress.txt

CRITICAL: You must actually FIX the issue, not just document it. The Merger Agent is blocked until you resolve this.

START NOW - investigate and fix the issue."

  # Run Claude to fix the issue
  log "Running Claude to fix issue..."
  
  if claude --dangerously-skip-permissions "$prompt" 2>&1; then
    # Check if issue was resolved (Claude should have updated the coordination file)
    local status=$(jq -r --arg id "$issue_id" '.issueQueue[]? | select(.id == $id) | .status' "$COORD_FILE" 2>/dev/null || echo "unknown")
    
    if [ "$status" = "resolved" ]; then
      success "Issue $issue_id resolved"
      return 0
    else
      # If Claude didn't mark it resolved, we'll mark it based on outcome
      # For now, assume it's resolved if Claude completed successfully
      resolve_issue "$issue_id" "Fixed by remediation agent"
      return 0
    fi
  else
    fail_issue "$issue_id" "Claude remediation failed"
    return 1
  fi
}

# Cleanup on exit
cleanup() {
  log "Cleaning up..."
  update_coord '.remediation.status' '"stopped"'
  update_coord '.remediation.pid' 'null'
  log "Remediation agent stopped"
}

# Main remediation loop
remediation_loop() {
  log "Starting remediation loop (poll interval: ${POLL_INTERVAL}s)"
  
  while true; do
    # Check if merger is still running
    local merger_status=$(jq -r '.merger.status' "$COORD_FILE" 2>/dev/null || echo "unknown")
    
    if [ "$merger_status" = "stopped" ] || [ "$merger_status" = "not_started" ]; then
      warn "Merger is not running (status: $merger_status). Waiting..."
      sleep 30
      continue
    fi
    
    # Get next pending issue
    local next_issue=$(get_next_pending_issue)
    
    if [ -n "$next_issue" ]; then
      claim_issue "$next_issue"
      remediate_issue "$next_issue"
    else
      # No issues, just update last check time
      local now=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
      update_coord '.remediation.lastCheck' "\"$now\""
    fi
    
    # Wait before next check
    sleep $POLL_INTERVAL
  done
}

# Main
main() {
  parse_args "$@"
  
  log "=== PR Merge Remediation Agent (Claude) ==="
  log "Repo root: $REPO_ROOT"
  log "Prompt file: $PROMPT_FILE"
  log "Poll interval: ${POLL_INTERVAL}s"
  
  check_prereqs
  wait_for_merger
  init_remediation
  
  # Set up cleanup trap
  trap cleanup EXIT
  
  warn "Using --dangerously-skip-permissions - all operations will be auto-approved"
  log "Press Ctrl+C to stop"
  
  cd "$REPO_ROOT"
  
  # Run the remediation loop
  remediation_loop
}

main "$@"
