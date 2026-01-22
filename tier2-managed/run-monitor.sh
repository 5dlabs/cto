#!/bin/bash
# Unified E2E Monitor Agent
# Monitors Installer progress, verifies gates, detects issues
#
# Usage: ./run-monitor.sh [--no-wait]
#
# By default waits for installer to be running before starting

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PROMPT_FILE="$SCRIPT_DIR/monitor-prompt.md"
COORD_FILE="$SCRIPT_DIR/ralph-coordination.json"
PRD_FILE="$SCRIPT_DIR/prd.json"
NO_WAIT=false
CHECK_INTERVAL=120  # 2 minutes between monitor checks

# Source environment variables (including FACTORY_API_KEY for Droid)
if [[ -f "$REPO_ROOT/.env.local" ]]; then
  source "$REPO_ROOT/.env.local"
fi

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

usage() {
  cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Launch the Unified E2E Monitor Agent.

This agent monitors the Installer's progress for 3-4 hours:
  - Checks phase gates and timeouts
  - Verifies VERIFICATION criteria with kubectl
  - Detects stuck states and errors
  - Logs findings to progress.txt
  - Flags cleanup requirements

Options:
    --no-wait           Don't wait for installer to start
    -h, --help          Show this help message

Examples:
    ./run-monitor.sh
    ./run-monitor.sh --no-wait
EOF
  exit 0
}

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --no-wait)
      NO_WAIT=true
      shift
      ;;
    -h|--help)
      usage
      ;;
    *)
      error "Unknown option: $1"
      usage
      ;;
  esac
done

# Check prerequisites
check_prereqs() {
  log "Checking prerequisites..."
  
  # Check Droid CLI
  if ! command -v droid &> /dev/null; then
    error "Droid CLI not found. Install from Factory.ai"
    exit 1
  fi
  
  # Check FACTORY_API_KEY
  if [[ -z "${FACTORY_API_KEY:-}" ]]; then
    error "FACTORY_API_KEY not set. Check .env.local"
    exit 1
  fi
  
  # Check coordination file
  if [[ ! -f "$COORD_FILE" ]]; then
    error "Coordination file not found: $COORD_FILE"
    error "Run the installer first to initialize state"
    exit 1
  fi
  
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
  update_coord '.session.monitorCli' '"droid"'
  
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
  
  # Get current installer status
  local installer_step=$(jq -r '.installer.currentStep // "unknown"' "$COORD_FILE")
  local installer_status=$(jq -r '.installer.status // "unknown"' "$COORD_FILE")
  
  # Build the prompt for this check
  local prompt="You are the Unified E2E Monitor Agent. Read and follow the instructions in $PROMPT_FILE.

This is monitor check #$check_num at $now.

INSTALLER STATUS:
- Status: $installer_status
- Current Step: $installer_step

Current coordination state:
$(cat "$COORD_FILE" | jq -c .)

YOUR TASKS FOR THIS CHECK:

1. CHECK INSTALLER PROGRESS
   - Is it stuck on the same step for > 30 min? → Flag CRITICAL
   - What story is it working on?
   - Is lastUpdate recent?

2. VERIFY PHASE GATES
   Use kubectl with KUBECONFIG=/tmp/admin-cto/kubeconfig (when cluster exists):
   - kubectl get nodes (should show Ready)
   - kubectl get pods -n cto (controller, web)
   - kubectl get crd boltruns.cto.5dlabs.ai

3. CHECK FOR ISSUES
   - Multi-region servers? → Flag cleanupRequired
   - Pods in CrashLoopBackOff?
   - API errors?

4. LOG FINDINGS
   - Update progress.txt with summary
   - Add issues to issueQueue if found

5. VERIFY STORY COMPLETION
   If installer marks a story as passes: true, verify with actual commands.

GATE TIMEOUTS:
- Pre-Flight: 5 min
- Admin Infrastructure: 20 min
- Admin Talos: 20 min
- Admin Kubernetes: 15 min
- Admin GitOps: 30 min
- Platform: 15 min
- BoltRun: 10 min
- UI Testing: 15 min
- Client Infra: 45 min
- Connectivity: 15 min

Be thorough but concise. This is an automated check."

  log "Running monitor check #$check_num..."
  
  # Run droid in exec mode with full permissions
  droid exec \
    --skip-permissions-unsafe \
    --cwd "$REPO_ROOT" \
    "$prompt" 2>&1 || {
      warn "Monitor check #$check_num completed with warnings"
    }
  
  success "Monitor check #$check_num complete"
}

# Main monitoring loop
monitor_loop() {
  local check_num=1
  
  log "Starting monitor loop (check interval: ${CHECK_INTERVAL}s)"
  
  while true; do
    # Check if installer is still running
    local installer_status=$(jq -r '.installer.status' "$COORD_FILE" 2>/dev/null || echo "unknown")
    
    if [ "$installer_status" = "complete" ]; then
      success "Installer completed successfully! Running final verification..."
      run_monitor_check $check_num
      success "Monitoring complete."
      break
    fi
    
    if [ "$installer_status" = "stopped" ] || [ "$installer_status" = "pending" ]; then
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
  echo -e "${CYAN}╔═══════════════════════════════════════════════════════════════╗${NC}"
  echo -e "${CYAN}║         UNIFIED E2E MONITOR AGENT (Droid)                     ║${NC}"
  echo -e "${CYAN}╚═══════════════════════════════════════════════════════════════╝${NC}"
  
  log "Repo root: $REPO_ROOT"
  log "Check interval: ${CHECK_INTERVAL}s"
  
  check_prereqs
  wait_for_installer
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
