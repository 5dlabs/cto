#!/bin/bash
# Create a tmux session for the GitOps Ralph Loop

set -euo pipefail

SESSION="gitops-ralph"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Kill existing session if it exists
tmux kill-session -t "$SESSION" 2>/dev/null || true

# Create new session
tmux new-session -d -s "$SESSION" -n "ralph" -c "$SCRIPT_DIR"

# Layout:
# +-----------------------------------+
# |         Deployer (Claude)         |
# +-----------------------------------+
# |         Hardener (Droid)          |
# +-----------------+-----------------+
# |  Coordination   |   Progress Log  |
# +-----------------+-----------------+

# Pane 0: Deployer
tmux send-keys -t "$SESSION:0.0" "clear && echo '=== GITOPS DEPLOYER (Claude) ===' && echo 'Run: ./run-deployer.sh' && echo ''" Enter

# Split for hardener
tmux split-window -t "$SESSION:0" -v -c "$SCRIPT_DIR"
tmux send-keys -t "$SESSION:0.1" "clear && echo '=== GITOPS HARDENER (Droid) ===' && echo 'Run: ./run-hardener.sh' && echo ''" Enter

# Split for logs
tmux split-window -t "$SESSION:0" -v -c "$SCRIPT_DIR"
tmux split-window -t "$SESSION:0.2" -h -c "$SCRIPT_DIR"

# Coordination state
tmux send-keys -t "$SESSION:0.2" "watch -n 5 'echo === Coordination ===; jq . ralph-coordination.json 2>/dev/null || echo waiting'" Enter

# Progress log
tmux send-keys -t "$SESSION:0.3" "watch -n 2 'echo === Progress ===; tail -30 progress.txt 2>/dev/null || echo waiting'" Enter

# Layout
tmux select-layout -t "$SESSION:0" main-horizontal
tmux resize-pane -t "$SESSION:0.0" -y 18
tmux resize-pane -t "$SESSION:0.1" -y 12

# Select deployer pane
tmux select-pane -t "$SESSION:0.0"

echo ""
echo "=== GitOps Ralph Loop - tmux Session ==="
echo ""
echo "Session '$SESSION' created with 4 panes:"
echo "  - Top:          Deployer (Claude)"
echo "  - Middle:       Hardener (Droid)"  
echo "  - Bottom Left:  Coordination State"
echo "  - Bottom Right: Progress Log"
echo ""
echo "To attach: tmux attach -t $SESSION"
echo ""
echo "In the top pane, run: ./run-deployer.sh"
echo "In the middle pane (after deployer starts), run: ./run-hardener.sh"
echo ""

# Attach to the session
exec tmux attach -t "$SESSION"
