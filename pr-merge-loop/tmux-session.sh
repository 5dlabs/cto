#!/bin/bash
# Create a tmux session for the PR Merge Ralph Loop
# Shows merger, monitor agent, and logs in split panes

set -euo pipefail

SESSION="pr-merge-ralph"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Kill existing session if it exists
tmux kill-session -t "$SESSION" 2>/dev/null || true

# Create new session with the merger pane
tmux new-session -d -s "$SESSION" -n "ralph" -c "$SCRIPT_DIR"

# Layout:
# +-----------------------------------+
# |         Merger (Claude)           |
# +-----------------------------------+
# |        Monitor (Droid)             |
# +-----------------------------------+
# |      Remediation (Claude)          |
# +-----------------+-----------------+
# |  Coordination   |   Progress Log  |
# +-----------------+-----------------+

# Pane 0: Merger (top) - just show prompt, user starts manually
tmux send-keys -t "$SESSION:0.0" "clear" Enter
tmux send-keys -t "$SESSION:0.0" "echo '=== MERGER AGENT (Claude) ==='" Enter
tmux send-keys -t "$SESSION:0.0" "echo 'Run: ./run-merger.sh'" Enter
tmux send-keys -t "$SESSION:0.0" "echo ''" Enter

# Split horizontally for monitor agent
tmux split-window -t "$SESSION:0" -v -c "$SCRIPT_DIR"
tmux send-keys -t "$SESSION:0.1" "clear" Enter
tmux send-keys -t "$SESSION:0.1" "echo '=== MONITOR AGENT (Droid) ==='" Enter
tmux send-keys -t "$SESSION:0.1" "echo 'Run: ./run-monitor.sh (after merger starts)'" Enter
tmux send-keys -t "$SESSION:0.1" "echo ''" Enter

# Split horizontally for remediation agent
tmux split-window -t "$SESSION:0" -v -c "$SCRIPT_DIR"
tmux send-keys -t "$SESSION:0.2" "clear" Enter
tmux send-keys -t "$SESSION:0.2" "echo '=== REMEDIATION AGENT (Claude) ==='" Enter
tmux send-keys -t "$SESSION:0.2" "echo 'Run: ./run-remediation.sh (after merger starts)'" Enter
tmux send-keys -t "$SESSION:0.2" "echo ''" Enter

# Split horizontally again for logs
tmux split-window -t "$SESSION:0" -v -c "$SCRIPT_DIR"

# Split the bottom pane vertically for coordination and progress
tmux split-window -t "$SESSION:0.3" -h -c "$SCRIPT_DIR"

# Pane 3: Coordination state (bottom left)
tmux send-keys -t "$SESSION:0.3" "watch -n 5 'echo === Coordination State ===; cat ralph-coordination.json | jq . 2>/dev/null || echo waiting'" Enter

# Pane 4: Progress log (bottom right)
tmux send-keys -t "$SESSION:0.4" "watch -n 2 'echo === Progress Log ===; tail -40 progress.txt 2>/dev/null || echo waiting'" Enter

# Set pane sizes (give more space to top panes)
tmux select-layout -t "$SESSION:0" main-horizontal
tmux resize-pane -t "$SESSION:0.0" -y 15
tmux resize-pane -t "$SESSION:0.1" -y 12
tmux resize-pane -t "$SESSION:0.2" -y 12
tmux resize-pane -t "$SESSION:0.3" -y 12

# Select the merger pane
tmux select-pane -t "$SESSION:0.0"

echo ""
echo "=== PR Merge Ralph Loop - tmux Session ==="
echo ""
echo "Session '$SESSION' created with 5 panes:"
echo "  - Top:          Merger Agent (Claude)"
echo "  - Upper Middle: Monitor Agent (Droid)"  
echo "  - Lower Middle: Remediation Agent (Claude)"
echo "  - Bottom Left:  Coordination State"
echo "  - Bottom Right: Progress Log"
echo ""
echo "To attach: tmux attach -t $SESSION"
echo ""
echo "In the top pane, run: ./run-merger.sh"
echo "In the upper middle pane (after merger starts), run: ./run-monitor.sh"
echo "In the lower middle pane (after merger starts), run: ./run-remediation.sh"
echo ""

# Attach to the session
exec tmux attach -t "$SESSION"