#!/bin/bash
# ralph-dashboard-sync.sh - Sync Ralph loop state to mobile dashboard
#
# Usage: source this file in your Ralph loop scripts
#   source scripts/2026-01/ralph-dashboard-sync.sh
#
# Then call the functions to update state:
#   ralph_init_session "my-session-name"
#   ralph_update_step "Deploying ArgoCD" 15 25
#   ralph_log "Starting deployment..."
#   ralph_check_commands  # Returns pending command if any

# Dashboard API endpoint (Cloudflare Pages)
RALPH_API_URL="${RALPH_API_URL:-https://5dlabs.ai/api/ralph}"

# Session state (in-memory)
_RALPH_SESSION_ID=""
_RALPH_START_TIME=""

# Initialize a new Ralph session
ralph_init_session() {
  local name="${1:-ralph-session}"
  _RALPH_SESSION_ID="${name}-$(date +%s)"
  _RALPH_START_TIME=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  
  local state=$(cat <<EOF
{
  "sessionId": "$_RALPH_SESSION_ID",
  "executor": {
    "status": "running",
    "currentStep": "Initializing",
    "stepNumber": 0,
    "totalSteps": 0,
    "lastUpdate": "$_RALPH_START_TIME",
    "lastError": null
  },
  "watcher": {
    "status": "stopped",
    "lastCheck": null,
    "checkCount": 0
  },
  "stats": {
    "totalRetries": 0,
    "issuesDetected": 0,
    "issuesFixed": 0,
    "successfulSteps": 0,
    "totalDuration": "0m"
  },
  "hardeningActions": [],
  "progressLog": ["[$_RALPH_START_TIME] Session started: $_RALPH_SESSION_ID"]
}
EOF
)

  curl -s -X POST "${RALPH_API_URL}/state" \
    -H "Content-Type: application/json" \
    -d "$state" > /dev/null 2>&1 &
    
  echo "📱 Ralph dashboard: ${RALPH_API_URL%/api/ralph}/ralph"
}

# Update current step
ralph_update_step() {
  local step_name="$1"
  local step_num="${2:-0}"
  local total_steps="${3:-0}"
  local status="${4:-running}"
  
  local now=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  local duration=$(ralph_duration)
  
  # Get current state, update it
  local current=$(curl -s "${RALPH_API_URL}/state" 2>/dev/null || echo "{}")
  
  local state=$(echo "$current" | jq --arg step "$step_name" \
    --arg num "$step_num" \
    --arg total "$total_steps" \
    --arg status "$status" \
    --arg now "$now" \
    --arg dur "$duration" \
    '.executor.currentStep = $step |
     .executor.stepNumber = ($num | tonumber) |
     .executor.totalSteps = ($total | tonumber) |
     .executor.status = $status |
     .executor.lastUpdate = $now |
     .stats.totalDuration = $dur')
  
  curl -s -X POST "${RALPH_API_URL}/state" \
    -H "Content-Type: application/json" \
    -d "$state" > /dev/null 2>&1 &
}

# Log a message
ralph_log() {
  local message="$1"
  local level="${2:-info}"
  
  curl -s -X POST "${RALPH_API_URL}/log" \
    -H "Content-Type: application/json" \
    -d "{\"message\": \"$message\", \"level\": \"$level\"}" > /dev/null 2>&1 &
}

# Record an issue and fix
ralph_hardening_action() {
  local issue="$1"
  local workaround="$2"
  local success="${3:-true}"
  
  local now=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  
  # Get current state
  local current=$(curl -s "${RALPH_API_URL}/state" 2>/dev/null || echo "{}")
  
  local state=$(echo "$current" | jq --arg issue "$issue" \
    --arg fix "$workaround" \
    --arg ts "$now" \
    --argjson success "$success" \
    '.hardeningActions += [{"issue": $issue, "workaround": $fix, "timestamp": $ts, "success": $success}] |
     .stats.issuesDetected += 1 |
     .stats.issuesFixed += (if $success then 1 else 0 end)')
  
  curl -s -X POST "${RALPH_API_URL}/state" \
    -H "Content-Type: application/json" \
    -d "$state" > /dev/null 2>&1 &
}

# Report an error
ralph_error() {
  local error_msg="$1"
  local now=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  
  local current=$(curl -s "${RALPH_API_URL}/state" 2>/dev/null || echo "{}")
  
  local state=$(echo "$current" | jq --arg err "$error_msg" --arg now "$now" \
    '.executor.lastError = $err | .executor.lastUpdate = $now')
  
  curl -s -X POST "${RALPH_API_URL}/state" \
    -H "Content-Type: application/json" \
    -d "$state" > /dev/null 2>&1 &
    
  ralph_log "$error_msg" "error"
}

# Mark session complete
ralph_complete() {
  local now=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  local duration=$(ralph_duration)
  
  local current=$(curl -s "${RALPH_API_URL}/state" 2>/dev/null || echo "{}")
  
  local state=$(echo "$current" | jq --arg now "$now" --arg dur "$duration" \
    '.executor.status = "complete" |
     .executor.currentStep = "Complete" |
     .executor.lastUpdate = $now |
     .executor.lastError = null |
     .stats.totalDuration = $dur')
  
  curl -s -X POST "${RALPH_API_URL}/state" \
    -H "Content-Type: application/json" \
    -d "$state" > /dev/null 2>&1 &
    
  ralph_log "Session completed successfully" "success"
}

# Check for pending commands from mobile
ralph_check_commands() {
  local response=$(curl -s "${RALPH_API_URL}/command" 2>/dev/null)
  local cmd=$(echo "$response" | jq -r '.command.command // empty')
  local cmd_id=$(echo "$response" | jq -r '.command.id // empty')
  
  if [ -n "$cmd" ] && [ "$cmd" != "null" ]; then
    # Acknowledge the command
    curl -s "${RALPH_API_URL}/command?ack=$cmd_id" > /dev/null 2>&1
    echo "$cmd"
  fi
}

# Handle commands (call this in your loop)
ralph_handle_commands() {
  local cmd=$(ralph_check_commands)
  
  case "$cmd" in
    pause)
      ralph_log "Paused by mobile command" "info"
      ralph_update_step "$(jq -r '.executor.currentStep' /tmp/ralph-state.json 2>/dev/null || echo "Paused")" 0 0 "paused"
      return 1  # Signal to pause
      ;;
    resume)
      ralph_log "Resumed by mobile command" "info"
      return 0  # Signal to continue
      ;;
    stop)
      ralph_log "Stopped by mobile command" "info"
      ralph_update_step "Stopped" 0 0 "stopped"
      return 2  # Signal to exit
      ;;
    *)
      return 0  # No command, continue
      ;;
  esac
}

# Calculate duration since session start
ralph_duration() {
  if [ -z "$_RALPH_START_TIME" ]; then
    echo "0m"
    return
  fi
  
  local start_epoch=$(date -j -f "%Y-%m-%dT%H:%M:%SZ" "$_RALPH_START_TIME" "+%s" 2>/dev/null || date -d "$_RALPH_START_TIME" "+%s" 2>/dev/null || echo 0)
  local now_epoch=$(date "+%s")
  local diff=$((now_epoch - start_epoch))
  
  if [ $diff -lt 60 ]; then
    echo "${diff}s"
  elif [ $diff -lt 3600 ]; then
    echo "$((diff / 60))m$((diff % 60))s"
  else
    echo "$((diff / 3600))h$((diff % 3600 / 60))m"
  fi
}

# Sync from local coordination file (for existing Ralph loops)
ralph_sync_from_file() {
  local coord_file="${1:-ralph-coordination.json}"
  local progress_file="${2:-progress.txt}"
  
  if [ ! -f "$coord_file" ]; then
    return 1
  fi
  
  # Read local state
  local local_state=$(cat "$coord_file")
  
  # Build dashboard state from local coordination file
  local session_id=$(echo "$local_state" | jq -r '.session.id // "unknown"')
  local installer_status=$(echo "$local_state" | jq -r '.installer.status // "stopped"')
  local current_step=$(echo "$local_state" | jq -r '.installer.currentStep // "Unknown"')
  local step_num=$(echo "$local_state" | jq -r '.installer.stepNumber // 0')
  local total_steps=$(echo "$local_state" | jq -r '.installer.totalSteps // 0')
  local last_update=$(echo "$local_state" | jq -r '.installer.lastUpdate // ""')
  local last_error=$(echo "$local_state" | jq -r '.installer.lastError // null')
  local watcher_status=$(echo "$local_state" | jq -r '.monitor.status // "stopped"')
  local check_count=$(echo "$local_state" | jq -r '.monitor.checkCount // 0')
  local duration=$(echo "$local_state" | jq -r '.stats.totalDuration // "0m"')
  local issues_detected=$(echo "$local_state" | jq -r '.stats.issuesDetected // 0')
  local issues_fixed=$(echo "$local_state" | jq -r '.stats.issuesFixed // 0')
  local successful_steps=$(echo "$local_state" | jq -r '.stats.successfulSteps // 0')
  
  # Get hardening actions
  local hardening=$(echo "$local_state" | jq '.hardeningActions // []')
  
  # Get last 20 lines from progress.txt
  local progress_log="[]"
  if [ -f "$progress_file" ]; then
    progress_log=$(tail -20 "$progress_file" | jq -R -s 'split("\n") | map(select(length > 0))')
  fi
  
  # Build dashboard state
  local dashboard_state=$(cat <<EOF
{
  "sessionId": "$session_id",
  "executor": {
    "status": "$installer_status",
    "currentStep": "$current_step",
    "stepNumber": $step_num,
    "totalSteps": $total_steps,
    "lastUpdate": "$last_update",
    "lastError": $last_error
  },
  "watcher": {
    "status": "$watcher_status",
    "lastCheck": null,
    "checkCount": $check_count
  },
  "stats": {
    "totalRetries": 0,
    "issuesDetected": $issues_detected,
    "issuesFixed": $issues_fixed,
    "successfulSteps": $successful_steps,
    "totalDuration": "$duration"
  },
  "hardeningActions": $hardening,
  "progressLog": $progress_log
}
EOF
)

  curl -s -X POST "${RALPH_API_URL}/state" \
    -H "Content-Type: application/json" \
    -d "$dashboard_state" > /dev/null 2>&1
}

echo "📱 Ralph dashboard sync loaded. Functions available:"
echo "   ralph_init_session <name>     - Start a new session"
echo "   ralph_update_step <name> <n> <total> - Update progress"
echo "   ralph_log <message>           - Log a message"
echo "   ralph_error <message>         - Report an error"
echo "   ralph_complete                - Mark session complete"
echo "   ralph_sync_from_file <file>   - Sync from coordination.json"
echo "   ralph_check_commands          - Check for mobile commands"
