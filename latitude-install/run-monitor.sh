#!/bin/bash
# Run the Monitor Agent (Droid) for Latitude installation monitoring
# Usage: ./run-monitor.sh [--no-wait]
#
# By default waits for installer to be running before starting
# Use --no-wait to skip the wait

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PROMPT_FILE="$SCRIPT_DIR/monitor-prompt.md"
COORD_FILE="$SCRIPT_DIR/ralph-coordination.json"
NO_WAIT=false
CHECK_INTERVAL=120  # 2 minutes between monitor checks

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

# Set up environment for MCP tools
setup_mcp_env() {
  local state_dir="/tmp/latitude-test"
  
  # Export TALOSCONFIG if it exists (needed for talos-mcp)
  if [[ -f "$state_dir/talosconfig" ]]; then
    export TALOSCONFIG="$state_dir/talosconfig"
    log "TALOSCONFIG=$TALOSCONFIG"
  else
    warn "TALOSCONFIG not found yet - talos-mcp checks will skip"
  fi

  # Provide a fallback for tools that don't inherit env vars (talosctl default path)
  if [[ -n "${TALOSCONFIG:-}" ]]; then
    local talos_dir="$HOME/.talos"
    local talos_default="$talos_dir/config"

    if [[ ! -d "$talos_dir" ]]; then
      mkdir -p "$talos_dir" || warn "Unable to create $talos_dir for talosconfig fallback"
    fi

    if [[ -d "$talos_dir" && ! -e "$talos_default" ]]; then
      ln -s "$TALOSCONFIG" "$talos_default" 2>/dev/null || warn "Unable to link $talos_default to $TALOSCONFIG"
      if [[ -L "$talos_default" ]]; then
        log "Linked talosconfig to default path: $talos_default"
      fi
    fi
  fi
  
  # Export KUBECONFIG if it exists (needed for kubectl checks)
  if [[ -f "$state_dir/kubeconfig" ]]; then
    export KUBECONFIG="$state_dir/kubeconfig"
    log "KUBECONFIG=$KUBECONFIG"
  else
    warn "KUBECONFIG not found yet - kubectl checks will skip"
  fi
}

# Check prerequisites
check_prereqs() {
  log "Checking prerequisites..."
  
  # Check Droid CLI
  if ! command -v droid &> /dev/null; then
    error "Droid CLI not found. Install from Factory.ai"
    exit 1
  fi
  
  # Check coordination file
  if [[ ! -f "$COORD_FILE" ]]; then
    error "Coordination file not found: $COORD_FILE"
    error "Run the installer first to initialize state"
    exit 1
  fi
  
  # Set up MCP environment
  setup_mcp_env
  
  success "All prerequisites met"
}

# Wait for installer to be running
wait_for_installer() {
  if [ "$NO_WAIT" = true ]; then
    log "Skipping installer wait (--no-wait)"
    return 0
  fi
  
  log "Waiting for installer agent to start..."
  local max_wait=300  # 5 minutes
  local waited=0
  
  while [ $waited -lt $max_wait ]; do
    local status=$(jq -r '.installer.status' "$COORD_FILE" 2>/dev/null || echo "unknown")
    
    if [ "$status" = "running" ]; then
      success "Installer is running, starting monitor"
      return 0
    fi
    
    log "Installer status: $status (waiting...)"
    sleep 5
    waited=$((waited + 5))
  done
  
  error "Timeout waiting for installer to start"
  exit 1
}

# Update coordination state
update_coord() {
  local key="$1"
  local value="$2"
  local tmp=$(mktemp)
  jq "$key = $value" "$COORD_FILE" > "$tmp" && mv "$tmp" "$COORD_FILE"
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
  
  # Re-check for talosconfig/kubeconfig (may have been created since startup)
  setup_mcp_env
  
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
3. **IMPLEMENT CODE FIXES** - Actually edit files in crates/installer/ or crates/metal/
4. **Update lessons-learned.md** - Document what you fixed
5. **Log your actions** to progress.txt

CRITICAL: You are the HARDENING agent. Your job is to IMPLEMENT CODE FIXES, not just log issues.
If you see Claude retrying, working around issues, or fixing configs manually - FIX THE CODE so it doesn't happen next time.

Example: If progress.txt shows 'Talos config missing primary interface' - edit crates/metal/src/talos/config.rs to generate correct configs."

  log "Running hardening check #$check_num..."
  
  # Run droid in exec mode with full permissions
  # Pass environment variables explicitly
  TALOSCONFIG="${TALOSCONFIG:-}" KUBECONFIG="${KUBECONFIG:-}" \
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
    # Check if installer is still running
    local installer_status=$(jq -r '.installer.status' "$COORD_FILE" 2>/dev/null || echo "unknown")
    
    if [ "$installer_status" = "complete" ]; then
      success "Installer completed successfully! Stopping monitor."
      break
    fi
    
    if [ "$installer_status" = "stopped" ] || [ "$installer_status" = "not_started" ]; then
      warn "Installer is not running (status: $installer_status). Waiting..."
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
  
  log "=== Latitude Monitor Agent (Droid) ==="
  log "Repo root: $REPO_ROOT"
  log "Prompt file: $PROMPT_FILE"
  log "Check interval: ${CHECK_INTERVAL}s"
  
  check_prereqs
  wait_for_installer
  init_monitor
  
  # Set up cleanup trap
  trap cleanup EXIT
  
  warn "Using --skip-permissions-unsafe --auto high - full access mode"
  log "Press Ctrl+C to stop"
  
  cd "$REPO_ROOT"
  
  # Run the monitoring loop
  monitor_loop
}

main "$@"
