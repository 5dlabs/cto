#!/bin/bash
# Ralph Guardian for Code Quality - Monitors Ralph and tracks progress
# Usage: ralph-guardian.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$SCRIPT_DIR"

LOG_FILE="$SCRIPT_DIR/ralph-guardian.log"
STATE_FILE="$SCRIPT_DIR/ralph-cto.state.json"

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

# Get current story from prd.json
get_current_story() {
  jq -r '[.userStories[] | select(.passes != true)] | .[0].id // "DONE"' prd.json 2>/dev/null
}

# Count completed stories
count_completed() {
  jq '[.userStories[] | select(.passes == true)] | length' prd.json 2>/dev/null
}

# Count total stories
count_total() {
  jq '.userStories | length' prd.json 2>/dev/null
}

# Run verification checks
verify_codebase() {
  log "INFO" "Running verification checks..."
  
  # Format check
  if cargo fmt --all --check >/dev/null 2>&1; then
    log "OK" "✓ cargo fmt check passed"
  else
    log "WARN" "✗ cargo fmt check failed - run 'cargo fmt --all'"
    return 1
  fi
  
  # Clippy check
  if cargo clippy --all-targets -- -D warnings >/dev/null 2>&1; then
    log "OK" "✓ cargo clippy passed"
  else
    log "WARN" "✗ cargo clippy failed"
    return 1
  fi
  
  return 0
}

# Update metrics
update_metrics() {
  local clippy_allows=$(rg '#\[allow\(clippy::' "$ROOT_DIR/crates" --count-matches 2>/dev/null | awk -F: '{sum+=$2} END {print sum}' || echo "0")
  local completed=$(count_completed)
  local total=$(count_total)
  
  log "INFO" "Metrics:"
  log "INFO" "  Clippy allows: $clippy_allows"
  log "INFO" "  Stories: $completed / $total completed"
  
  # Update state file
  local tmp=$(mktemp)
  jq ".metrics.clippy_allows_current = $clippy_allows | .metrics.stories_completed = $completed | .metrics.stories_remaining = ($total - $completed)" "$STATE_FILE" > "$tmp" && mv "$tmp" "$STATE_FILE"
}

# Display status
show_status() {
  echo ""
  echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
  echo -e "${CYAN}   🔧 CTO Code Quality - Ralph Status${NC}"
  echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
  echo ""
  
  local current=$(get_current_story)
  local completed=$(count_completed)
  local total=$(count_total)
  local clippy_allows=$(rg '#\[allow\(clippy::' "$ROOT_DIR/crates" --count-matches 2>/dev/null | awk -F: '{sum+=$2} END {print sum}' || echo "?")
  
  echo -e "  Current Story: ${YELLOW}$current${NC}"
  echo -e "  Progress:      ${GREEN}$completed${NC} / $total stories"
  echo -e "  Clippy Allows: $clippy_allows"
  echo ""
  
  echo -e "${BLUE}Completed Stories:${NC}"
  jq -r '.userStories[] | select(.passes == true) | "  ✓ \(.id): \(.title)"' prd.json 2>/dev/null
  echo ""
  
  echo -e "${YELLOW}Remaining Stories:${NC}"
  jq -r '.userStories[] | select(.passes != true) | "  ○ \(.id): \(.title)"' prd.json 2>/dev/null
  echo ""
}

# Main
main() {
  log "INFO" "Ralph Guardian starting for Code Quality"
  log "INFO" "Working directory: $SCRIPT_DIR"
  
  show_status
  update_metrics
  
  if verify_codebase; then
    log "OK" "All verification checks passed"
  else
    log "WARN" "Some verification checks failed"
  fi
}

main "$@"
