#!/usr/bin/env bash
# Ralph Guardian for Agent CI Optimization
# Monitors Ralph progress and restarts if stalled

set -euo pipefail

PROJECT_DIR="/Users/jonathonfritz/code/work-projects/5dlabs/cto/agent-ci-test"
MAX_ITERATIONS=50
STALL_THRESHOLD_MINUTES=10

echo "🤖 Ralph Guardian for Agent CI Optimization"
echo "============================================"
echo "Project: $PROJECT_DIR"
echo "Max iterations: $MAX_ITERATIONS"
echo "Stall threshold: $STALL_THRESHOLD_MINUTES minutes"
echo ""

# Function to check if Ralph is running
is_ralph_running() {
    tmux has-session -t ralph 2>/dev/null && return 0 || return 1
}

# Function to get Ralph status
get_ralph_status() {
    if command -v ralph-monitor.sh >/dev/null 2>&1; then
        ~/.config/opencode/scripts/ralph-ultra/ralph-monitor.sh --status "$PROJECT_DIR" 2>/dev/null || true
    fi
}

# Function to start Ralph
start_ralph() {
    echo "🚀 Starting Ralph on agent-ci-test..."
    
    # Kill existing session if any
    tmux kill-session -t ralph 2>/dev/null || true
    sleep 2
    
    # Start new session
    tmux new-session -d -s ralph -c "$PROJECT_DIR" \
        "PATH=\"/Users/jonathonfritz/.claude/local:\$PATH\" ~/.config/opencode/scripts/ralph-ultra/ralph-monitor.sh \"$PROJECT_DIR\" $MAX_ITERATIONS"
    
    echo "✅ Ralph started in tmux session 'ralph'"
    echo "   Attach with: tmux attach -t ralph"
}

# Main guardian loop
guardian_loop() {
    while true; do
        echo ""
        echo "--- $(date '+%Y-%m-%d %H:%M:%S') ---"
        
        if ! is_ralph_running; then
            echo "⚠️  Ralph not running. Starting..."
            start_ralph
        else
            echo "✓ Ralph is running"
            
            # Check progress age
            if [ -f "$PROJECT_DIR/progress.txt" ]; then
                last_modified=$(stat -f %m "$PROJECT_DIR/progress.txt" 2>/dev/null || stat -c %Y "$PROJECT_DIR/progress.txt" 2>/dev/null)
                now=$(date +%s)
                age_minutes=$(( (now - last_modified) / 60 ))
                
                echo "  Progress age: ${age_minutes}m"
                
                if [ $age_minutes -gt $STALL_THRESHOLD_MINUTES ]; then
                    echo "⚠️  Progress stalled for ${age_minutes} minutes. Restarting..."
                    start_ralph
                fi
            fi
        fi
        
        # Brief status
        get_ralph_status | head -10 || true
        
        # Sleep before next check
        sleep 120  # Check every 2 minutes
    done
}

# Handle script arguments
case "${1:-}" in
    --start)
        start_ralph
        ;;
    --status)
        get_ralph_status
        ;;
    --attach)
        tmux attach -t ralph
        ;;
    --loop)
        guardian_loop
        ;;
    *)
        echo "Usage: $0 [--start|--status|--attach|--loop]"
        echo ""
        echo "  --start   Start Ralph once"
        echo "  --status  Show current status"
        echo "  --attach  Attach to Ralph tmux session"
        echo "  --loop    Run guardian loop (restarts if stalled)"
        exit 1
        ;;
esac
