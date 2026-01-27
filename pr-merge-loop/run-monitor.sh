#!/bin/bash
# Run the Monitor Agent (Droid) for PR merge monitoring
# Usage: ./run-monitor.sh [--no-wait]
#
# By default waits for merger to be running before starting
# Use --no-wait to skip the wait

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PROMPT_FILE="$SCRIPT_DIR/monitor-prompt.md"
COORD_FILE="$SCRIPT_DIR/ralph-coordination.json"
NO_WAIT=false
CHECK_INTERVAL=300  # 5 minutes between monitor checks

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
  
  # Check Droid CLI
  if ! command -v droid &> /dev/null; then
    error "Droid CLI not found. Install from Factory.ai"
    exit 1
  fi
  
  # LESSON LEARNED: Check jq prerequisite - used by update_coord
  # See: pr-merge-loop/lessons-learned.md#ISSUE-007
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
      success "Merger is running, starting monitor"
      return 0
    fi
    
    log "Merger status: $status (waiting...)"
    sleep 5
    waited=$((waited + 5))
  done
  
  error "Timeout waiting for merger to start"
  exit 1
}

# Update coordination state with file locking
# LESSON LEARNED: Use flock to prevent race conditions with concurrent updates
# See: pr-merge-loop/lessons-learned.md#ISSUE-008
update_coord() {
  local key="$1"
  local value="$2"
  local lock_file="$COORD_FILE.lock"
  local tmp=$(mktemp)
  
  # Use flock for atomic updates - prevents race condition with merger
  (
    flock -x 200
    jq "$key = $value" "$COORD_FILE" > "$tmp" && mv "$tmp" "$COORD_FILE"
  ) 200>"$lock_file"
}

# Initialize monitor
init_monitor() {
  local now=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  
  update_coord '.monitor.status' '"running"'
  update_coord '.monitor.lastCheck' "\"$now\""
  update_coord '.monitor.pid' "$$"
  
  log "Monitor initialized"
}

# Cleanup on exit
cleanup() {
  log "Cleaning up..."
  update_coord '.monitor.status' '"stopped"'
  update_coord '.monitor.pid' 'null'
  log "Monitor agent stopped"
}

# Run a single monitor check
run_monitor_check() {
  local check_num=$1
  local now=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  
  update_coord '.monitor.lastCheck' "\"$now\""
  update_coord '.monitor.checkCount' "$check_num"
  
  # Read the full prompt file
  local prompt_content=$(cat "$PROMPT_FILE")
  
  # Build the prompt for this check - HARDENING focus
  local prompt="$prompt_content

---

## Current Check: #$check_num at $now

### Coordination State
\`\`\`json
$(cat "$COORD_FILE" | jq .)
\`\`\`

### Your Task This Check

1. **Read progress.txt** - See what Claude had to do manually
2. **Identify automation opportunities** - What could be codified?
3. **IMPLEMENT CODE FIXES** - Actually edit files (.pre-commit-config.yaml, CI workflows, scripts)
4. **Update lessons-learned.md** - Document what you fixed
5. **Log your actions** to progress.txt

CRITICAL: You are the HARDENING agent. Your job is to IMPLEMENT CODE FIXES, not just log issues.
If you see Claude fixing the same issue repeatedly, fixing conflicts, or running manual steps - FIX THE CODE so it doesn't happen next time.

Example: If progress.txt shows 'Fixed clippy warnings in 5 PRs' - edit .pre-commit-config.yaml to add clippy check."

  log "Running hardening check #$check_num..."
  
  # Run droid in exec mode with full permissions
  droid exec \
    --skip-permissions-unsafe \
    --cwd "$REPO_ROOT" \
    "$prompt" 2>&1 || {
      warn "Hardening check #$check_num completed with warnings"
    }
  
  success "Hardening check #$check_num complete"
}

# Main monitoring loop
monitor_loop() {
  local check_num=1
  
  log "Starting monitor loop (check interval: ${CHECK_INTERVAL}s)"
  
  while true; do
    # Check if merger is still running
    local merger_status=$(jq -r '.merger.status' "$COORD_FILE" 2>/dev/null || echo "unknown")
    
    if [ "$merger_status" = "stopped" ] || [ "$merger_status" = "not_started" ]; then
      warn "Merger is not running (status: $merger_status). Waiting..."
      sleep 30
      continue
    fi
    
    # Run the check
    run_monitor_check $check_num
    check_num=$((check_num + 1))
    
    # Wait before next check
    log "Next check in ${CHECK_INTERVAL}s..."
    sleep $CHECK_INTERVAL
  done
}

# Main
main() {
  parse_args "$@"
  
  log "=== PR Merge Monitor Agent (Droid) ==="
  log "Repo root: $REPO_ROOT"
  log "Prompt file: $PROMPT_FILE"
  log "Check interval: ${CHECK_INTERVAL}s"
  
  check_prereqs
  wait_for_merger
  init_monitor
  
  # Set up cleanup trap
  trap cleanup EXIT
  
  warn "Using --skip-permissions-unsafe - full access mode"
  log "Press Ctrl+C to stop"
  
  cd "$REPO_ROOT"
  
  # Run the monitoring loop
  monitor_loop
}

main "$@"