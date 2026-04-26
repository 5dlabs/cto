#!/usr/bin/env bash
set -euo pipefail

# =============================================================================
# ralph-dual.sh - Launcher for Dual Ralph Self-Healing System
# =============================================================================
# Manages both monitor (GPT-5.2) and remediation (Claude) agents in screen sessions.
# =============================================================================

ROOT_DIR="${WORKSPACE:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
CONFIG_PATH="${RALPH_CONFIG:-${ROOT_DIR}/lifecycle-test/ralph-cto.json}"
COORDINATION_FILE="${ROOT_DIR}/lifecycle-test/ralph-coordination.json"

MONITOR_SCREEN="ralph-monitor"
REMEDIATION_SCREEN="ralph-remediation"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() { echo -e "[$(date -u '+%Y-%m-%dT%H:%M:%SZ')] $*"; }
info() { log "${BLUE}[INFO]${NC} $*"; }
success() { log "${GREEN}[SUCCESS]${NC} $*"; }
warn() { log "${YELLOW}[WARN]${NC} $*"; }
error() { log "${RED}[ERROR]${NC} $*"; }

usage() {
  cat <<'EOF'
Usage: scripts/2026-01/ralph-dual.sh <command>

Commands:
  start           Start both monitor and remediation agents
  stop            Stop both agents
  restart         Restart both agents
  status          Show status of both agents (including circuit breaker and session)
  attach          Attach to a screen session (specify: monitor or remediation)
  logs            Show recent logs from both agents
  queue           Show current issue queue
  reset           Reset coordination file to initial state
  reset-session   Reset session (clears context, starts fresh)
  circuit-status  Show circuit breaker status
  circuit-reset   Reset circuit breaker to closed state

Examples:
  ./scripts/2026-01/ralph-dual.sh start
  ./scripts/2026-01/ralph-dual.sh status
  ./scripts/2026-01/ralph-dual.sh attach monitor
  ./scripts/2026-01/ralph-dual.sh attach remediation
  ./scripts/2026-01/ralph-dual.sh circuit-status
  ./scripts/2026-01/ralph-dual.sh reset-session
  ./scripts/2026-01/ralph-dual.sh stop
EOF
}

# Check if screen is installed
check_prerequisites() {
  if ! command -v screen &> /dev/null; then
    error "screen is not installed. Install with: brew install screen"
    exit 1
  fi
  
  if ! command -v jq &> /dev/null; then
    error "jq is not installed. Install with: brew install jq"
    exit 1
  fi
  
  if [[ ! -f "$CONFIG_PATH" ]]; then
    error "Config file not found: $CONFIG_PATH"
    exit 1
  fi
}

# Check if dual agent is enabled in config
check_dual_agent_enabled() {
  local enabled
  enabled=$(jq -r '.dualAgent.enabled // false' "$CONFIG_PATH")
  
  if [[ "$enabled" != "true" ]]; then
    error "Dual agent is not enabled in config"
    info "Set dualAgent.enabled to true in $CONFIG_PATH"
    exit 1
  fi
}

# Initialize coordination file
init_coordination_file() {
  if [[ ! -f "$COORDINATION_FILE" ]]; then
    info "Initializing coordination file..."
    cat > "$COORDINATION_FILE" <<'EOF'
{
  "monitor": {
    "status": "stopped",
    "currentPhase": null,
    "lastCheck": null,
    "pid": null
  },
  "remediation": {
    "status": "idle",
    "currentIssue": null,
    "pid": null
  },
  "issueQueue": [],
  "stats": {
    "totalIssues": 0,
    "resolved": 0,
    "failed": 0
  }
}
EOF
  fi
}

# Check if a screen session exists
screen_exists() {
  local name="$1"
  # Try sending a command to the screen - returns 0 if session exists
  screen -S "$name" -X version >/dev/null 2>&1
}

# Get PID of screen session  
get_screen_pid() {
  local name="$1"
  # Parse screen -list output to get PID
  screen -list 2>&1 | grep "\.$name" | awk -F'[.\t]' '{for(i=1;i<=NF;i++) if($i ~ /^[0-9]+$/) {print $i; exit}}'
}

# Wait for screen session to start with proper health check
# Args: $1 = screen name, $2 = log file, $3 = timeout seconds (default 10)
wait_for_screen_healthy() {
  local screen_name="$1"
  local log_file="$2"
  local timeout="${3:-10}"
  local waited=0
  
  while [[ $waited -lt $timeout ]]; do
    # Check if screen exists
    if screen_exists "$screen_name"; then
      # Verify the script is actually running by checking log file activity
      if [[ -f "$log_file" ]]; then
        local log_age
        log_age=$(( $(date +%s) - $(stat -f %m "$log_file" 2>/dev/null || stat -c %Y "$log_file" 2>/dev/null || echo 0) ))
        # If log was modified in last 5 seconds, script is running
        if [[ $log_age -lt 5 ]]; then
          return 0
        fi
      fi
      # Screen exists but no log activity yet, wait a bit more
      sleep 1
      waited=$((waited + 1))
    else
      # Screen doesn't exist yet, wait
      sleep 1
      waited=$((waited + 1))
    fi
  done
  
  # Final check - just verify screen exists if log check isn't working
  screen_exists "$screen_name"
}

# Start monitor agent
start_monitor() {
  if screen_exists "$MONITOR_SCREEN"; then
    warn "Monitor agent already running"
    return 0
  fi
  
  # Clear old log for fresh start detection
  : > /tmp/ralph-monitor.log
  
  info "Starting monitor agent in screen session: $MONITOR_SCREEN"
  screen -dmS "$MONITOR_SCREEN" bash -c "cd '$ROOT_DIR' && ./scripts/2026-01/ralph-monitor.sh 2>&1 | tee -a /tmp/ralph-monitor.log"
  
  # Wait for screen session to be healthy (with timeout)
  if wait_for_screen_healthy "$MONITOR_SCREEN" "/tmp/ralph-monitor.log" 10; then
    success "Monitor agent started"
  else
    error "Failed to start monitor agent (timeout waiting for initialization)"
    error "Check /tmp/ralph-monitor.log for details"
    return 1
  fi
}

# Start remediation agent
start_remediation() {
  if screen_exists "$REMEDIATION_SCREEN"; then
    warn "Remediation agent already running"
    return 0
  fi
  
  # Clear old log for fresh start detection
  : > /tmp/ralph-remediation.log
  
  info "Starting remediation agent in screen session: $REMEDIATION_SCREEN"
  screen -dmS "$REMEDIATION_SCREEN" bash -c "cd '$ROOT_DIR' && ./scripts/2026-01/ralph-remediation.sh 2>&1 | tee -a /tmp/ralph-remediation.log"
  
  # Wait for screen session to be healthy (with timeout)
  if wait_for_screen_healthy "$REMEDIATION_SCREEN" "/tmp/ralph-remediation.log" 10; then
    success "Remediation agent started"
  else
    error "Failed to start remediation agent (timeout waiting for initialization)"
    error "Check /tmp/ralph-remediation.log for details"
    return 1
  fi
}

# Stop monitor agent
stop_monitor() {
  if ! screen_exists "$MONITOR_SCREEN"; then
    info "Monitor agent not running"
    return 0
  fi
  
  info "Stopping monitor agent..."
  screen -S "$MONITOR_SCREEN" -X quit
  sleep 1
  
  if screen_exists "$MONITOR_SCREEN"; then
    warn "Monitor agent still running, forcing kill..."
    local pid
    pid=$(get_screen_pid "$MONITOR_SCREEN")
    kill -9 "$pid" 2>/dev/null || true
  fi
  
  success "Monitor agent stopped"
}

# Stop remediation agent
stop_remediation() {
  if ! screen_exists "$REMEDIATION_SCREEN"; then
    info "Remediation agent not running"
    return 0
  fi
  
  info "Stopping remediation agent..."
  screen -S "$REMEDIATION_SCREEN" -X quit
  sleep 1
  
  if screen_exists "$REMEDIATION_SCREEN"; then
    warn "Remediation agent still running, forcing kill..."
    local pid
    pid=$(get_screen_pid "$REMEDIATION_SCREEN")
    kill -9 "$pid" 2>/dev/null || true
  fi
  
  success "Remediation agent stopped"
}

# Start both agents
cmd_start() {
  check_dual_agent_enabled
  init_coordination_file
  
  info "Starting Dual Ralph Self-Healing System..."
  echo ""
  
  start_monitor
  start_remediation
  
  echo ""
  success "Dual Ralph system started"
  echo ""
  info "Use 'ralph-dual.sh attach monitor' to attach to monitor"
  info "Use 'ralph-dual.sh attach remediation' to attach to remediation"
  info "Use 'ralph-dual.sh status' to check status"
}

# Stop both agents
cmd_stop() {
  info "Stopping Dual Ralph Self-Healing System..."
  echo ""
  
  stop_monitor
  stop_remediation
  
  # Update coordination file
  if [[ -f "$COORDINATION_FILE" ]]; then
    local tmp
    tmp="$(mktemp)"
    jq '.monitor.status = "stopped" | .remediation.status = "stopped"' "$COORDINATION_FILE" > "$tmp" && mv "$tmp" "$COORDINATION_FILE"
  fi
  
  echo ""
  success "Dual Ralph system stopped"
}

# Restart both agents
cmd_restart() {
  cmd_stop
  sleep 2
  cmd_start
}

# Show status
cmd_status() {
  echo ""
  echo "╔══════════════════════════════════════════════════════════════╗"
  echo "║           Dual Ralph Self-Healing System Status              ║"
  echo "╚══════════════════════════════════════════════════════════════╝"
  echo ""
  
  # Screen session status
  echo "┌──────────────────────────────────────────────────────────────┐"
  echo "│ Screen Sessions                                              │"
  echo "├──────────────────────────────────────────────────────────────┤"
  
  if screen_exists "$MONITOR_SCREEN"; then
    local pid
    pid=$(get_screen_pid "$MONITOR_SCREEN")
    echo -e "│ Monitor:     ${GREEN}● RUNNING${NC} (screen: $MONITOR_SCREEN, pid: $pid)"
  else
    echo -e "│ Monitor:     ${RED}○ STOPPED${NC}"
  fi
  
  if screen_exists "$REMEDIATION_SCREEN"; then
    local pid
    pid=$(get_screen_pid "$REMEDIATION_SCREEN")
    echo -e "│ Remediation: ${GREEN}● RUNNING${NC} (screen: $REMEDIATION_SCREEN, pid: $pid)"
  else
    echo -e "│ Remediation: ${RED}○ STOPPED${NC}"
  fi
  
  echo "└──────────────────────────────────────────────────────────────┘"
  echo ""
  
  # Coordination file status
  if [[ -f "$COORDINATION_FILE" ]]; then
    echo "┌──────────────────────────────────────────────────────────────┐"
    echo "│ Coordination Status                                         │"
    echo "├──────────────────────────────────────────────────────────────┤"
    
    local monitor_status remediation_status current_phase current_issue
    monitor_status=$(jq -r '.monitor.status // "unknown"' "$COORDINATION_FILE")
    remediation_status=$(jq -r '.remediation.status // "unknown"' "$COORDINATION_FILE")
    current_phase=$(jq -r '.monitor.currentPhase // "none"' "$COORDINATION_FILE")
    current_issue=$(jq -r '.remediation.currentIssue // "none"' "$COORDINATION_FILE")
    
    echo "│ Monitor Status:     $monitor_status"
    echo "│ Current Phase:      $current_phase"
    echo "│ Remediation Status: $remediation_status"
    echo "│ Current Issue:      $current_issue"
    echo "└──────────────────────────────────────────────────────────────┘"
    echo ""
    
    echo "┌──────────────────────────────────────────────────────────────┐"
    echo "│ Statistics                                                  │"
    echo "├──────────────────────────────────────────────────────────────┤"
    
    local total resolved failed pending cb_trips session_resets
    total=$(jq -r '.stats.totalIssues // 0' "$COORDINATION_FILE")
    resolved=$(jq -r '.stats.resolved // 0' "$COORDINATION_FILE")
    failed=$(jq -r '.stats.failed // 0' "$COORDINATION_FILE")
    pending=$(jq -r '[.issueQueue[] | select(.status == "pending" or .status == "claimed")] | length' "$COORDINATION_FILE")
    cb_trips=$(jq -r '.stats.circuitBreakerTrips // 0' "$COORDINATION_FILE")
    session_resets=$(jq -r '.stats.sessionResets // 0' "$COORDINATION_FILE")
    
    echo "│ Total Issues:       $total"
    echo -e "│ Resolved:           ${GREEN}$resolved${NC}"
    echo -e "│ Failed:             ${RED}$failed${NC}"
    echo -e "│ Pending:            ${YELLOW}$pending${NC}"
    echo "│ Circuit Breaker Trips: $cb_trips"
    echo "│ Session Resets:     $session_resets"
    echo "└──────────────────────────────────────────────────────────────┘"
    echo ""
    
    # Circuit Breaker Status
    echo "┌──────────────────────────────────────────────────────────────┐"
    echo "│ Circuit Breaker                                             │"
    echo "├──────────────────────────────────────────────────────────────┤"
    
    local cb_state cb_no_progress cb_same_error cb_opened_at
    cb_state=$(jq -r '.circuitBreaker.state // "unknown"' "$COORDINATION_FILE")
    cb_no_progress=$(jq -r '.circuitBreaker.noProgressCount // 0' "$COORDINATION_FILE")
    cb_same_error=$(jq -r '.circuitBreaker.sameErrorCount // 0' "$COORDINATION_FILE")
    cb_opened_at=$(jq -r '.circuitBreaker.openedAt // "N/A"' "$COORDINATION_FILE")
    
    case "$cb_state" in
      closed)
        echo -e "│ State:            ${GREEN}● CLOSED${NC}"
        ;;
      open)
        echo -e "│ State:            ${RED}● OPEN${NC}"
        ;;
      half-open)
        echo -e "│ State:            ${YELLOW}● HALF-OPEN${NC}"
        ;;
      *)
        echo "│ State:            $cb_state"
        ;;
    esac
    echo "│ No Progress Count: $cb_no_progress"
    echo "│ Same Error Count:  $cb_same_error"
    if [[ "$cb_opened_at" != "N/A" && "$cb_opened_at" != "null" ]]; then
      echo "│ Opened At:         $cb_opened_at"
    fi
    echo "└──────────────────────────────────────────────────────────────┘"
    echo ""
    
    # Session Status
    echo "┌──────────────────────────────────────────────────────────────┐"
    echo "│ Session                                                     │"
    echo "├──────────────────────────────────────────────────────────────┤"
    
    local session_id session_started session_last_activity session_expiration
    session_id=$(jq -r '.session.id // "none"' "$COORDINATION_FILE")
    session_started=$(jq -r '.session.startedAt // "N/A"' "$COORDINATION_FILE")
    session_last_activity=$(jq -r '.session.lastActivity // "N/A"' "$COORDINATION_FILE")
    session_expiration=$(jq -r '.session.expirationHours // 24' "$COORDINATION_FILE")
    
    if [[ "$session_id" != "none" && "$session_id" != "null" ]]; then
      echo -e "│ Session ID:       ${GREEN}$session_id${NC}"
    else
      echo -e "│ Session ID:       ${YELLOW}(no active session)${NC}"
    fi
    echo "│ Started At:       $session_started"
    echo "│ Last Activity:    $session_last_activity"
    echo "│ Expiration:       ${session_expiration}h"
    echo "└──────────────────────────────────────────────────────────────┘"
  else
    warn "Coordination file not found: $COORDINATION_FILE"
  fi
  
  echo ""
}

# Attach to a screen session
cmd_attach() {
  local target="${1:-}"
  
  case "$target" in
    monitor)
      if ! screen_exists "$MONITOR_SCREEN"; then
        error "Monitor agent not running"
        exit 1
      fi
      info "Attaching to monitor agent (Ctrl+A, D to detach)..."
      screen -r "$MONITOR_SCREEN"
      ;;
    remediation)
      if ! screen_exists "$REMEDIATION_SCREEN"; then
        error "Remediation agent not running"
        exit 1
      fi
      info "Attaching to remediation agent (Ctrl+A, D to detach)..."
      screen -r "$REMEDIATION_SCREEN"
      ;;
    *)
      error "Specify which agent to attach to: monitor or remediation"
      exit 1
      ;;
  esac
}

# Show recent logs
cmd_logs() {
  echo ""
  echo "═══════════════════════════════════════════════════════════════"
  echo "                    Recent Monitor Logs"
  echo "═══════════════════════════════════════════════════════════════"
  if [[ -f /tmp/ralph-monitor.log ]]; then
    tail -30 /tmp/ralph-monitor.log
  else
    echo "(no logs yet)"
  fi
  
  echo ""
  echo "═══════════════════════════════════════════════════════════════"
  echo "                  Recent Remediation Logs"
  echo "═══════════════════════════════════════════════════════════════"
  if [[ -f /tmp/ralph-remediation.log ]]; then
    tail -30 /tmp/ralph-remediation.log
  else
    echo "(no logs yet)"
  fi
  echo ""
}

# Show issue queue
cmd_queue() {
  if [[ ! -f "$COORDINATION_FILE" ]]; then
    error "Coordination file not found"
    exit 1
  fi
  
  echo ""
  echo "═══════════════════════════════════════════════════════════════"
  echo "                       Issue Queue"
  echo "═══════════════════════════════════════════════════════════════"
  
  local queue_length
  queue_length=$(jq '.issueQueue | length' "$COORDINATION_FILE")
  
  if [[ "$queue_length" -eq 0 ]]; then
    echo "(queue is empty)"
  else
    jq -r '.issueQueue[] | "[\(.status | ascii_upcase)] \(.id) - \(.phase)/\(.gate) (exit: \(.exitCode))"' "$COORDINATION_FILE"
  fi
  
  echo ""
}

# Reset coordination file
cmd_reset() {
  info "Resetting coordination file..."
  
  # Stop agents first if running
  if screen_exists "$MONITOR_SCREEN" || screen_exists "$REMEDIATION_SCREEN"; then
    warn "Stopping running agents before reset..."
    cmd_stop
    sleep 1
  fi
  
  cat > "$COORDINATION_FILE" <<'EOF'
{
  "monitor": {
    "status": "stopped",
    "currentPhase": null,
    "lastCheck": null,
    "pid": null
  },
  "remediation": {
    "status": "idle",
    "currentIssue": null,
    "pid": null
  },
  "issueQueue": [],
  "circuitBreaker": {
    "state": "closed",
    "noProgressCount": 0,
    "sameErrorCount": 0,
    "lastError": null,
    "lastFileChange": null,
    "openedAt": null
  },
  "session": {
    "id": null,
    "startedAt": null,
    "lastActivity": null,
    "expirationHours": 24
  },
  "stats": {
    "totalIssues": 0,
    "resolved": 0,
    "failed": 0,
    "circuitBreakerTrips": 0,
    "sessionResets": 0,
    "averageResolutionTime": null,
    "lastSuccessfulPhase": null
  }
}
EOF
  
  success "Coordination file reset"
}

# Reset session only
cmd_reset_session() {
  if [[ ! -f "$COORDINATION_FILE" ]]; then
    error "Coordination file not found"
    exit 1
  fi
  
  info "Resetting session..."
  
  # Generate new session ID
  local session_id
  session_id="ralph-$(date +%s)-$$-$(head -c 4 /dev/urandom | xxd -p)"
  
  local tmp
  tmp="$(mktemp)"
  
  jq --arg id "$session_id" \
     '.session.id = $id |
      .session.startedAt = (now | todate) |
      .session.lastActivity = (now | todate) |
      .stats.sessionResets = ((.stats.sessionResets // 0) + 1)' \
     "$COORDINATION_FILE" > "$tmp" && mv "$tmp" "$COORDINATION_FILE"
  
  success "Session reset. New session ID: $session_id"
}

# Show circuit breaker status
cmd_circuit_status() {
  if [[ ! -f "$COORDINATION_FILE" ]]; then
    error "Coordination file not found"
    exit 1
  fi
  
  echo ""
  echo "╔══════════════════════════════════════════════════════════════╗"
  echo "║               Circuit Breaker Status                         ║"
  echo "╚══════════════════════════════════════════════════════════════╝"
  echo ""
  
  local cb_state cb_no_progress cb_same_error cb_last_error cb_opened_at cb_trips
  cb_state=$(jq -r '.circuitBreaker.state // "unknown"' "$COORDINATION_FILE")
  cb_no_progress=$(jq -r '.circuitBreaker.noProgressCount // 0' "$COORDINATION_FILE")
  cb_same_error=$(jq -r '.circuitBreaker.sameErrorCount // 0' "$COORDINATION_FILE")
  cb_last_error=$(jq -r '.circuitBreaker.lastError // "none"' "$COORDINATION_FILE")
  cb_opened_at=$(jq -r '.circuitBreaker.openedAt // "N/A"' "$COORDINATION_FILE")
  cb_trips=$(jq -r '.stats.circuitBreakerTrips // 0' "$COORDINATION_FILE")
  
  # Get thresholds from config
  local no_progress_threshold same_error_threshold recovery_minutes
  no_progress_threshold=$(jq -r '.dualAgent.circuitBreaker.noProgressThreshold // 3' "$CONFIG_PATH")
  same_error_threshold=$(jq -r '.dualAgent.circuitBreaker.sameErrorThreshold // 5' "$CONFIG_PATH")
  recovery_minutes=$(jq -r '.dualAgent.circuitBreaker.recoveryMinutes // 5' "$CONFIG_PATH")
  
  case "$cb_state" in
    closed)
      echo -e "State:             ${GREEN}● CLOSED${NC} (normal operation)"
      ;;
    open)
      echo -e "State:             ${RED}● OPEN${NC} (paused - no requests allowed)"
      ;;
    half-open)
      echo -e "State:             ${YELLOW}● HALF-OPEN${NC} (testing - single request allowed)"
      ;;
    *)
      echo "State:             $cb_state"
      ;;
  esac
  
  echo ""
  echo "Counters:"
  echo "  No Progress:     $cb_no_progress / $no_progress_threshold (threshold)"
  echo "  Same Error:      $cb_same_error / $same_error_threshold (threshold)"
  echo ""
  echo "Configuration:"
  echo "  Recovery Time:   ${recovery_minutes} minutes"
  echo ""
  echo "History:"
  echo "  Total Trips:     $cb_trips"
  if [[ "$cb_last_error" != "none" && "$cb_last_error" != "null" ]]; then
    echo "  Last Error:      $cb_last_error"
  fi
  if [[ "$cb_opened_at" != "N/A" && "$cb_opened_at" != "null" ]]; then
    echo "  Opened At:       $cb_opened_at"
  fi
  echo ""
}

# Reset circuit breaker
cmd_circuit_reset() {
  if [[ ! -f "$COORDINATION_FILE" ]]; then
    error "Coordination file not found"
    exit 1
  fi
  
  info "Resetting circuit breaker..."
  
  local tmp
  tmp="$(mktemp)"
  
  jq '.circuitBreaker = {
        "state": "closed",
        "noProgressCount": 0,
        "sameErrorCount": 0,
        "lastError": null,
        "lastFileChange": (now | todate),
        "openedAt": null
      }' \
     "$COORDINATION_FILE" > "$tmp" && mv "$tmp" "$COORDINATION_FILE"
  
  success "Circuit breaker reset to CLOSED state"
}

# Main entry point
main() {
  check_prerequisites
  
  local cmd="${1:-}"
  shift || true
  
  case "$cmd" in
    start)
      cmd_start
      ;;
    stop)
      cmd_stop
      ;;
    restart)
      cmd_restart
      ;;
    status)
      cmd_status
      ;;
    attach)
      cmd_attach "$@"
      ;;
    logs)
      cmd_logs
      ;;
    queue)
      cmd_queue
      ;;
    reset)
      cmd_reset
      ;;
    reset-session)
      cmd_reset_session
      ;;
    circuit-status)
      cmd_circuit_status
      ;;
    circuit-reset)
      cmd_circuit_reset
      ;;
    -h|--help|help|"")
      usage
      ;;
    *)
      error "Unknown command: $cmd"
      usage
      exit 1
      ;;
  esac
}

main "$@"
