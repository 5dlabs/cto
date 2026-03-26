#!/usr/bin/env bash
set -uo pipefail
trap 'exit 0' INT TERM

WS="${WORKSPACE:-/Users/jonathon/5dlabs/cto}"
INTAKE="$WS/.intake"
TASKS="$WS/.tasks"
LAST_STATE=""
CYCLE=0

announce() {
  local msg="$1"
  echo "[$(date '+%H:%M:%S')] $msg"
  say "$msg" &
}

get_state() {
  local state=""

  if [ -f "$INTAKE/intake-summary.json" ]; then
    state="intake-complete"
  elif [ -f "$INTAKE/intake-sub-workflow.log" ] && [ "$(stat -f%z "$INTAKE/intake-sub-workflow.log" 2>/dev/null || echo 0)" -gt 0 ]; then
    local last_line
    last_line=$(tail -1 "$INTAKE/intake-sub-workflow.log" 2>/dev/null || echo "")
    state="intake-sub: $last_line"
  elif [ -f "$INTAKE/design-deliberation.log" ] && [ "$(stat -f%z "$INTAKE/design-deliberation.log" 2>/dev/null || echo 0)" -gt 0 ]; then
    state="design-deliberation"
  elif [ -f "$INTAKE/tech-decision-points.json" ] && [ "$(stat -f%z "$INTAKE/tech-decision-points.json" 2>/dev/null || echo 0)" -gt 0 ]; then
    local ts
    ts=$(stat -f%m "$INTAKE/tech-decision-points.json" 2>/dev/null || echo 0)
    local now
    now=$(date +%s)
    if [ $((now - ts)) -lt 120 ]; then
      state="tech-decisions-fresh"
    else
      state="post-decisions"
    fi
  elif [ -f "$INTAKE/design-decision-points.json" ]; then
    state="design-decisions"
  elif [ -f "$INTAKE/decision-points.json" ]; then
    state="decision-points"
  elif [ -f "$INTAKE/initial-tasks.json" ]; then
    state="initial-tasks"
  elif [ -f "$INTAKE/linear-session-id.txt" ]; then
    state="linear-session"
  else
    state="preflight-or-design"
  fi

  # Check for new .tasks content
  local task_docs=0 task_files=0
  if [ -d "$TASKS/docs" ]; then
    task_docs=$(find "$TASKS/docs" -type f 2>/dev/null | wc -l | tr -d ' ')
  fi
  if [ -d "$TASKS/tasks" ]; then
    task_files=$(find "$TASKS/tasks" -type f 2>/dev/null | wc -l | tr -d ' ')
  fi

  # Check for design-intake temp file
  local design_temp_size=0
  for f in /tmp/design-intake-*.json; do
    if [ -f "$f" ]; then
      local sz
      sz=$(stat -f%z "$f" 2>/dev/null || echo 0)
      if [ "$sz" -gt "$design_temp_size" ]; then
        design_temp_size=$sz
      fi
    fi
  done

  echo "${state}|docs=${task_docs}|tasks=${task_files}|design_tmp=${design_temp_size}"
}

check_lobster_alive() {
  if ! pgrep -f "lobster.*pipeline" >/dev/null 2>&1; then
    echo "dead"
  else
    echo "alive"
  fi
}

announce "Monitor started. Watching pipeline progress."

while true; do
  CYCLE=$((CYCLE + 1))
  CURRENT_STATE=$(get_state)
  ALIVE=$(check_lobster_alive)

  if [ "$ALIVE" = "dead" ]; then
    announce "Warning: pipeline process appears to have stopped."
    echo "[$(date '+%H:%M:%S')] Pipeline process not found. Exiting monitor."
    exit 1
  fi

  if [ "$CURRENT_STATE" != "$LAST_STATE" ]; then
    PHASE=$(echo "$CURRENT_STATE" | cut -d'|' -f1)
    DOCS=$(echo "$CURRENT_STATE" | cut -d'|' -f2 | cut -d= -f2)
    TASKS_N=$(echo "$CURRENT_STATE" | cut -d'|' -f3 | cut -d= -f2)
    DTMP=$(echo "$CURRENT_STATE" | cut -d'|' -f4 | cut -d= -f2)

    case "$PHASE" in
      preflight-or-design)
        announce "Pipeline phase: design intake in progress." ;;
      initial-tasks)
        announce "Pipeline phase: initial tasks generated." ;;
      decision-points)
        announce "Pipeline phase: decision points extracted." ;;
      design-decisions)
        announce "Pipeline phase: design decisions ready." ;;
      tech-decisions-fresh)
        announce "Pipeline phase: tech decisions just completed." ;;
      post-decisions)
        announce "Pipeline phase: moving past decision extraction." ;;
      design-deliberation)
        announce "Pipeline phase: design deliberation running." ;;
      intake-sub*)
        announce "Pipeline phase: intake sub-workflow active." ;;
      intake-complete)
        announce "Pipeline phase: intake complete! Summary generated." ;;
    esac

    if [ "$DOCS" -gt 0 ] || [ "$TASKS_N" -gt 0 ]; then
      announce "Artifact count: ${DOCS} docs, ${TASKS_N} task files."
    fi

    LAST_STATE="$CURRENT_STATE"
  elif [ $((CYCLE % 4)) -eq 0 ]; then
    # Every ~2 minutes, give a heartbeat even if state hasn't changed
    PHASE=$(echo "$CURRENT_STATE" | cut -d'|' -f1)
    echo "[$(date '+%H:%M:%S')] Heartbeat — phase: $PHASE (no change)"
    say "Still working. Phase: ${PHASE}." &
  fi

  sleep 30
done
