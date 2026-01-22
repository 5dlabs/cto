#!/bin/bash
# ralph-sync-watcher.sh - Watch a local Ralph coordination file and sync to dashboard
#
# Usage: ./scripts/ralph-sync-watcher.sh [coordination-file] [progress-file]
#
# Example: ./scripts/ralph-sync-watcher.sh latitude-install/ralph-coordination.json latitude-install/progress.txt
#
# This runs in a loop, syncing state every 10 seconds.
# Run this alongside your Ralph loop to enable mobile monitoring.

set -euo pipefail

COORD_FILE="${1:-latitude-install/ralph-coordination.json}"
PROGRESS_FILE="${2:-latitude-install/progress.txt}"
SYNC_INTERVAL="${SYNC_INTERVAL:-10}"

# Load the sync functions
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/ralph-dashboard-sync.sh"

echo "🔄 Ralph Dashboard Sync Watcher"
echo "   Coordination file: $COORD_FILE"
echo "   Progress file: $PROGRESS_FILE"
echo "   Sync interval: ${SYNC_INTERVAL}s"
echo ""
echo "📱 Dashboard URL: ${RALPH_API_URL%/api/ralph}/ralph"
echo ""
echo "Press Ctrl+C to stop"
echo ""

# Track last modification time
last_mod=0

while true; do
  # Check if file exists
  if [ -f "$COORD_FILE" ]; then
    # Get current modification time
    if [[ "$OSTYPE" == "darwin"* ]]; then
      current_mod=$(stat -f %m "$COORD_FILE")
    else
      current_mod=$(stat -c %Y "$COORD_FILE")
    fi
    
    # Only sync if file changed or first run
    if [ "$current_mod" != "$last_mod" ]; then
      echo "[$(date '+%H:%M:%S')] Syncing state to dashboard..."
      ralph_sync_from_file "$COORD_FILE" "$PROGRESS_FILE"
      last_mod=$current_mod
      
      # Show current status
      status=$(jq -r '.installer.status // "unknown"' "$COORD_FILE" 2>/dev/null || echo "unknown")
      step=$(jq -r '.installer.currentStep // "unknown"' "$COORD_FILE" 2>/dev/null || echo "unknown")
      echo "[$(date '+%H:%M:%S')] Status: $status | Step: $step"
    fi
    
    # Check for mobile commands
    cmd=$(ralph_check_commands)
    if [ -n "$cmd" ]; then
      echo "[$(date '+%H:%M:%S')] 📱 Mobile command received: $cmd"
      
      case "$cmd" in
        pause)
          # Update local coordination file to paused
          jq '.installer.status = "paused"' "$COORD_FILE" > /tmp/coord-update.json && \
            mv /tmp/coord-update.json "$COORD_FILE"
          echo "[$(date '+%H:%M:%S')] ⏸️  Paused (updated coordination file)"
          ;;
        resume)
          jq '.installer.status = "running"' "$COORD_FILE" > /tmp/coord-update.json && \
            mv /tmp/coord-update.json "$COORD_FILE"
          echo "[$(date '+%H:%M:%S')] ▶️  Resumed (updated coordination file)"
          ;;
        stop)
          jq '.installer.status = "stopped"' "$COORD_FILE" > /tmp/coord-update.json && \
            mv /tmp/coord-update.json "$COORD_FILE"
          echo "[$(date '+%H:%M:%S')] ⏹️  Stopped (updated coordination file)"
          ;;
      esac
    fi
  else
    echo "[$(date '+%H:%M:%S')] Waiting for $COORD_FILE..."
  fi
  
  sleep "$SYNC_INTERVAL"
done
