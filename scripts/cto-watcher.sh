#!/usr/bin/env bash
# cto-watcher.sh - Watches for binary rebuilds and restarts launchd services
# =============================================================================
# This script monitors the target/release directory for changes to CTO binaries
# and uses launchctl kickstart to restart the corresponding launchd services.
#
# Usage: This script is meant to be run by launchd, not manually.
# =============================================================================

# Don't use set -e because we want the loop to continue even if kickstart fails
set -uo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WATCH_DIR="${CTO_WATCH_DIR:-$SCRIPT_DIR/../target/release}"
LOG_PREFIX="[cto-watcher]"
STATE_DIR="/tmp/cto-watcher-state"

# Binary to service mappings
# Note: healer binary is used by both healer and healer-sensor services
BINARIES="agent-controller pm-server healer"

# Returns space-separated list of service labels for a binary
get_service_labels() {
    local binary="$1"
    case "$binary" in
        agent-controller) echo "ai.5dlabs.cto.controller" ;;
        pm-server) echo "ai.5dlabs.cto.pm-server" ;;
        healer) echo "ai.5dlabs.cto.healer ai.5dlabs.cto.healer-sensor" ;;
        *) echo "" ;;
    esac
}

log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') $LOG_PREFIX $1"
}

# Get modification time as epoch seconds
get_mtime() {
    stat -f "%m" "$1" 2>/dev/null || echo "0"
}

# Get/set last known mtime for a binary (using files for bash 3.x compatibility)
get_last_mtime() {
    local binary="$1"
    cat "$STATE_DIR/$binary.mtime" 2>/dev/null || echo "0"
}

set_last_mtime() {
    local binary="$1"
    local mtime="$2"
    echo "$mtime" > "$STATE_DIR/$binary.mtime"
}

log "Starting watcher on $WATCH_DIR"
log "Watching binaries: $BINARIES"

# Create state directory
mkdir -p "$STATE_DIR"

# Store initial modification times
for binary in $BINARIES; do
    mtime=$(get_mtime "$WATCH_DIR/$binary")
    set_last_mtime "$binary" "$mtime"
    log "Initial mtime for $binary: $mtime"
done

# Get current user's GUI domain for launchctl
GUI_DOMAIN="gui/$(id -u)"

log "Entering watch loop..."

# Use fswatch to monitor the release directory
# -1 exits after first event, then we loop
while true; do
    # Wait for any change in the watch directory
    fswatch -1 -l 2 "$WATCH_DIR" >/dev/null 2>&1
    
    log "Detected filesystem change in $WATCH_DIR"
    
    # Small delay to let builds finish writing
    sleep 1
    
    # Check each binary for changes
    for binary in $BINARIES; do
        binary_path="$WATCH_DIR/$binary"
        services=$(get_service_labels "$binary")
        
        if [[ -z "$services" ]]; then
            continue
        fi
        
        if [[ -f "$binary_path" ]]; then
            current_mtime=$(get_mtime "$binary_path")
            last_mtime=$(get_last_mtime "$binary")
            
            if [[ "$current_mtime" != "$last_mtime" ]]; then
                log "Binary $binary changed (mtime: $last_mtime -> $current_mtime)"
                set_last_mtime "$binary" "$current_mtime"
                
                # Restart all services that use this binary
                for service in $services; do
                    log "Restarting $service..."
                    # kickstart -k kills the existing process and starts a new one
                    if launchctl kickstart -k "$GUI_DOMAIN/$service" 2>&1; then
                        log "✅ Successfully restarted $service"
                    else
                        log "⚠️  Could not restart $service (may not be loaded)"
                    fi
                done
            fi
        fi
    done
done
