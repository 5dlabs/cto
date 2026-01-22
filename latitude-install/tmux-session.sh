#!/bin/bash
# Create a tmux session for the Latitude Ralph Loop
# Shows installer, hardening agent, and logs in split panes

set -euo pipefail

SESSION="latitude-ralph"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Kill existing session if it exists
tmux kill-session -t "$SESSION" 2>/dev/null || true

# Create new session with the installer pane
tmux new-session -d -s "$SESSION" -n "ralph" -c "$SCRIPT_DIR"

# Layout:
# +-----------------------------------+
# |         Installer (Claude)        |
# +-----------------------------------+
# |        Hardening (Droid)          |
# +-----------------+-----------------+
# |  Coordination   |   Progress Log  |
# +-----------------+-----------------+

# Pane 0: Installer (top) - just show prompt, user starts manually
tmux send-keys -t "$SESSION:0.0" "clear" Enter
tmux send-keys -t "$SESSION:0.0" "echo '=== INSTALLER AGENT (Claude) ==='" Enter
tmux send-keys -t "$SESSION:0.0" "echo 'Run: ./run-installer.sh'" Enter
tmux send-keys -t "$SESSION:0.0" "echo ''" Enter

# Split horizontally for hardening agent
tmux split-window -t "$SESSION:0" -v -c "$SCRIPT_DIR"
tmux send-keys -t "$SESSION:0.1" "clear" Enter
tmux send-keys -t "$SESSION:0.1" "echo '=== HARDENING AGENT (Droid) ==='" Enter
tmux send-keys -t "$SESSION:0.1" "echo 'Run: ./run-monitor.sh (after installer starts)'" Enter
tmux send-keys -t "$SESSION:0.1" "echo ''" Enter

# Split horizontally again for logs
tmux split-window -t "$SESSION:0" -v -c "$SCRIPT_DIR"

# Split the bottom pane vertically for coordination and progress
tmux split-window -t "$SESSION:0.2" -h -c "$SCRIPT_DIR"

# Pane 2: Coordination state (bottom left)
tmux send-keys -t "$SESSION:0.2" "watch -n 5 'echo === Coordination State ===; cat ralph-coordination.json | jq . 2>/dev/null || echo waiting'" Enter

# Pane 3: Progress log (bottom right)
tmux send-keys -t "$SESSION:0.3" "watch -n 2 'echo === Progress Log ===; tail -40 progress.txt 2>/dev/null || echo waiting'" Enter

# Set pane sizes (give more space to top panes)
tmux select-layout -t "$SESSION:0" main-horizontal
tmux resize-pane -t "$SESSION:0.0" -y 20
tmux resize-pane -t "$SESSION:0.1" -y 15
tmux resize-pane -t "$SESSION:0.2" -y 15

# Select the installer pane
tmux select-pane -t "$SESSION:0.0"

echo ""
echo "=== Latitude Ralph Loop - tmux Session ==="
echo ""
echo "Session '$SESSION' created with 4 panes:"
echo "  - Top:          Installer Agent (Claude)"
echo "  - Middle:       Hardening Agent (Droid)"  
echo "  - Bottom Left:  Coordination State"
echo "  - Bottom Right: Progress Log"
echo ""
echo "To attach: tmux attach -t $SESSION"
echo ""
echo "In the top pane, run: ./run-installer.sh"
echo "In the middle pane (after installer starts), run: ./run-monitor.sh"
echo ""

# Attach to the session
exec tmux attach -t "$SESSION"
