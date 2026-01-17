#!/bin/bash
# Ralph Guardian - Monitors Ralph and intervenes if he goes off-rails
# Usage: ralph-guardian.sh <project_dir>

set -euo pipefail

PROJECT_DIR="${1:-}"
if [ -z "$PROJECT_DIR" ]; then
  echo "Usage: $0 <project_dir>"
  exit 1
fi

cd "$PROJECT_DIR" || exit 1
PROJECT_DIR="$(pwd)"

LOG_FILE="$PROJECT_DIR/ralph-guardian.log"
STATE_FILE="$PROJECT_DIR/.ralph-guardian-state.json"
MONITOR_SCRIPT="/Users/jonathonfritz/.config/opencode/scripts/ralph-ultra/ralph-monitor.sh"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

log() {
  local level="$1"
  local message="$2"
  local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
  local color=""
  
  case "$level" in
    INFO)  color="$BLUE" ;;
    WARN)  color="$YELLOW" ;;
    ERROR) color="$RED" ;;
    OK)    color="$GREEN" ;;
  esac
  
  echo -e "${color}[${timestamp}] [${level}]${NC} ${message}" | tee -a "$LOG_FILE"
}

# Initialize state
init_state() {
  if [ ! -f "$STATE_FILE" ]; then
    cat > "$STATE_FILE" <<EOF
{
  "last_story": "",
  "story_start_time": 0,
  "stuck_count": 0,
  "restart_count": 0,
  "last_progress_lines": 0,
  "alerts_sent": []
}
EOF
  fi
}

# Get current state
get_state() {
  local key="$1"
  jq -r ".$key" "$STATE_FILE" 2>/dev/null || echo ""
}

# Update state
update_state() {
  local key="$1"
  local value="$2"
  local tmp=$(mktemp)
  jq ".$key = $value" "$STATE_FILE" > "$tmp" && mv "$tmp" "$STATE_FILE"
}

# Check if Ralph is making progress
check_progress() {
  local current_story=$(jq -r '[.userStories[] | select(.passes != true)] | .[0].id // "DONE"' prd.json 2>/dev/null)
  local last_story=$(get_state "last_story")
  local story_start=$(get_state "story_start_time")
  local now=$(date +%s)
  
  # Check progress.txt line count
  local current_lines=$(wc -l < progress.txt 2>/dev/null || echo "0")
  local last_lines=$(get_state "last_progress_lines")
  
  log "INFO" "Current story: $current_story (was: $last_story)"
  log "INFO" "Progress lines: $current_lines (was: $last_lines)"
  
  # Story changed - good progress!
  if [ "$current_story" != "$last_story" ]; then
    log "OK" "Story changed from $last_story to $current_story - Ralph is making progress!"
    update_state "last_story" "\"$current_story\""
    update_state "story_start_time" "$now"
    update_state "stuck_count" "0"
    return 0
  fi
  
  # Progress.txt shrunk - RED FLAG!
  if [ "$current_lines" -lt "$last_lines" ]; then
    local diff=$((last_lines - current_lines))
    log "ERROR" "Progress.txt SHRUNK by $diff lines! Ralph may be deleting content!"
    update_state "stuck_count" "$(($(get_state stuck_count) + 1))"
    return 1
  fi
  
  # Progress.txt grew - good sign
  if [ "$current_lines" -gt "$last_lines" ]; then
    log "OK" "Progress.txt grew by $((current_lines - last_lines)) lines"
    update_state "last_progress_lines" "$current_lines"
    update_state "stuck_count" "0"
    return 0
  fi
  
  # Same story, same progress - check time
  if [ "$story_start" -eq 0 ]; then
    update_state "story_start_time" "$now"
    story_start=$now
  fi
  
  local elapsed=$(( (now - story_start) / 60 ))
  log "WARN" "Ralph stuck on $current_story for ${elapsed}m"
  
  # Stuck for >30 minutes - intervene
  if [ "$elapsed" -gt 30 ]; then
    log "ERROR" "Ralph stuck for >30m on $current_story - needs intervention!"
    update_state "stuck_count" "$(($(get_state stuck_count) + 1))"
    return 1
  fi
  
  return 0
}

# Check for off-rails behavior
check_off_rails() {
  local issues=()
  
  # Check 1: Progress.txt has header
  if ! grep -q "^# CTO Platform Lifecycle Test" progress.txt 2>/dev/null; then
    issues+=("Missing progress.txt header")
  fi
  
  # Check 2: PRD exists
  if [ ! -f "prd.json" ]; then
    issues+=("prd.json missing!")
  fi
  
  # Check 3: Progress.txt not empty
  local lines=$(wc -l < progress.txt 2>/dev/null || echo "0")
  if [ "$lines" -lt 5 ]; then
    issues+=("progress.txt too short ($lines lines)")
  fi
  
  # Check 4: Claude process exists
  if ! pgrep -x "claude" > /dev/null 2>&1; then
    issues+=("Claude process not running!")
  fi
  
  if [ ${#issues[@]} -gt 0 ]; then
    log "ERROR" "Off-rails issues detected:"
    for issue in "${issues[@]}"; do
      log "ERROR" "  - $issue"
    done
    return 1
  fi
  
  return 0
}

# Check for healer issues that need remediation
check_healer_issues() {
  # Check for open healer issues without associated PRs
  local open_issues=$(gh issue list --repo 5dlabs/cto --label healer --label ci-failure --state open --json number,title --jq 'length' 2>/dev/null || echo "0")
  
  if [ "$open_issues" -gt 0 ]; then
    log "WARN" "Found $open_issues open healer CI failure issues"
    
    # Check for failed remediation CodeRuns
    local failed_coderuns=$(kubectl get coderuns -n cto -l 'healer.agents.platform/type=remediation' --field-selector status.phase=Failed --no-headers 2>/dev/null | wc -l || echo "0")
    
    if [ "$failed_coderuns" -gt 0 ]; then
      log "ERROR" "Found $failed_coderuns failed remediation CodeRuns - healer not fixing issues!"
      return 1
    fi
  fi
  
  return 0
}

# Restart Ralph
restart_ralph() {
  local reason="$1"
  log "WARN" "Restarting Ralph: $reason"
  
  # Kill tmux session
  tmux kill-session -t ralph 2>/dev/null || true
  sleep 2
  
  # Restart via monitor script
  "$MONITOR_SCRIPT" "$PROJECT_DIR" 5 &
  
  update_state "restart_count" "$(($(get_state restart_count) + 1))"
  update_state "stuck_count" "0"
  
  log "OK" "Ralph restarted (total restarts: $(get_state restart_count))"
}

# Main monitoring loop
monitor_loop() {
  log "INFO" "Ralph Guardian starting"
  log "INFO" "Project: $PROJECT_DIR"
  log "INFO" "Check interval: 5 minutes"
  
  init_state
  
  while true; do
    log "INFO" "=== Guardian Check ==="
    
    # Check if Ralph is making progress
    if ! check_progress; then
      local stuck_count=$(get_state "stuck_count")
      log "WARN" "No progress detected (stuck count: $stuck_count)"
      
      if [ "$stuck_count" -ge 3 ]; then
        restart_ralph "Stuck for 3+ checks"
      fi
    fi
    
    # Check for off-rails behavior
    if ! check_off_rails; then
      local stuck_count=$(get_state "stuck_count")
      if [ "$stuck_count" -ge 2 ]; then
        restart_ralph "Off-rails behavior detected"
      fi
    fi
    
    # Check for healer issues
    if ! check_healer_issues; then
      log "WARN" "Healer remediation issues detected - Ralph should investigate"
    fi
    
    # Update progress line count
    local current_lines=$(wc -l < progress.txt 2>/dev/null || echo "0")
    update_state "last_progress_lines" "$current_lines"
    
    log "INFO" "Next check in 5 minutes..."
    sleep 300
  done
}

# Run
monitor_loop
