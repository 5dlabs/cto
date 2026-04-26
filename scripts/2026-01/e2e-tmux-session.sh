#!/bin/bash
# E2E Intake Test TMUX Session
# Creates a multi-pane monitoring session for observing the intake swarm workflow
#
# Usage:
#   ./scripts/2026-01/e2e-tmux-session.sh        # Create session
#   tmux attach -t e2e-intake-test       # Connect to session
#
# Layout:
#   ┌──────────────────────────────────────────────────────────────────────┐
#   │ Pane 0: Swarm Coordinator (claudesp team lead output)                │
#   ├────────────────────────────────────┬─────────────────────────────────┤
#   │ Pane 1: Intake Binary Output       │ Pane 2: CLI Stream Output       │
#   │ (progress.jsonl tail)              │ (claude-stream.jsonl tail)      │
#   ├────────────────────────────────────┼─────────────────────────────────┤
#   │ Pane 3: Service Logs               │ Pane 4: Linear Sidecar Logs     │
#   │ (pm-server + controller)           │ (status-sync output)            │
#   └────────────────────────────────────┴─────────────────────────────────┘

set -euo pipefail

SESSION="e2e-intake-test"
WORKSPACE="/Users/jonathonfritz/cto-e2e-testing"
WORKDIR="${WORKSPACE}/alerthub-e2e-test"
LAUNCHD_LOGS="/tmp/cto-launchd"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if session already exists
if tmux has-session -t "$SESSION" 2>/dev/null; then
    echo -e "${YELLOW}Session '$SESSION' already exists.${NC}"
    echo "Attach with: tmux attach -t $SESSION"
    echo "Or kill with: tmux kill-session -t $SESSION"
    exit 0
fi

# Create work directory if it doesn't exist
mkdir -p "$WORKDIR"

echo -e "${GREEN}Creating TMUX session '$SESSION'...${NC}"

# Create new detached session
tmux new-session -d -s "$SESSION" -c "$WORKSPACE" -x 200 -y 50

# Rename first window
tmux rename-window -t "$SESSION:0" 'E2E-Intake-Test'

# Pane 0 (top): Swarm Coordinator - full width
# This pane will be used to run the claudesp swarm

# Split horizontally for middle row (pane 1)
tmux split-window -v -t "$SESSION:0" -c "$WORKSPACE" -p 66

# Split horizontally for bottom row (pane 2)
tmux split-window -v -t "$SESSION:0.1" -c "$WORKSPACE" -p 50

# Split pane 1 (middle left) vertically to create pane 3 (middle right)
tmux split-window -h -t "$SESSION:0.1" -c "$WORKSPACE" -p 50

# Split pane 2 (bottom left) vertically to create pane 4 (bottom right)
tmux split-window -h -t "$SESSION:0.3" -c "$WORKSPACE" -p 50

# Set pane titles (requires pane-border-status to be visible)
tmux select-pane -t "$SESSION:0.0" -T "Swarm Coordinator"
tmux select-pane -t "$SESSION:0.1" -T "Intake Progress"
tmux select-pane -t "$SESSION:0.2" -T "CLI Stream"
tmux select-pane -t "$SESSION:0.3" -T "Service Logs"
tmux select-pane -t "$SESSION:0.4" -T "Sidecar Logs"

# Enable pane border status to show titles
tmux set-option -t "$SESSION" pane-border-status top
tmux set-option -t "$SESSION" pane-border-format " #{pane_index}: #{pane_title} "

# Pane 0: Swarm Coordinator - wait for user to start swarm
tmux send-keys -t "$SESSION:0.0" "# Swarm Coordinator - Run claudesp swarm here" C-m
tmux send-keys -t "$SESSION:0.0" "# Example: claudesp --swarm 'Run E2E intake test'" C-m
tmux send-keys -t "$SESSION:0.0" "cd ${WORKSPACE}" C-m
tmux send-keys -t "$SESSION:0.0" "echo '=== Ready for Swarm ===' && echo ''" C-m

# Pane 1: Intake Progress - tail progress.jsonl
tmux send-keys -t "$SESSION:0.1" "# Intake Progress - tailing progress.jsonl" C-m
tmux send-keys -t "$SESSION:0.1" "cd ${WORKDIR}" C-m
tmux send-keys -t "$SESSION:0.1" "touch progress.jsonl 2>/dev/null || true" C-m
tmux send-keys -t "$SESSION:0.1" "tail -f progress.jsonl 2>/dev/null || (echo 'Waiting for progress.jsonl...' && sleep infinity)" C-m

# Pane 2: CLI Stream - tail claude-stream.jsonl with jq for better formatting
tmux send-keys -t "$SESSION:0.2" "# CLI Stream - tailing claude-stream.jsonl" C-m
tmux send-keys -t "$SESSION:0.2" "cd ${WORKDIR}" C-m
tmux send-keys -t "$SESSION:0.2" "touch claude-stream.jsonl 2>/dev/null || true" C-m
tmux send-keys -t "$SESSION:0.2" "tail -f claude-stream.jsonl 2>/dev/null | jq -r '.type // .event // \"data\"' 2>/dev/null || tail -f claude-stream.jsonl" C-m

# Pane 3: Service Logs - tail pm-server and controller logs
tmux send-keys -t "$SESSION:0.3" "# Service Logs - pm-server + controller" C-m
if [ -d "$LAUNCHD_LOGS" ]; then
    tmux send-keys -t "$SESSION:0.3" "tail -f ${LAUNCHD_LOGS}/pm-server.log ${LAUNCHD_LOGS}/controller.log 2>/dev/null || echo 'Service logs not available'" C-m
else
    tmux send-keys -t "$SESSION:0.3" "echo 'Launchd logs not found. Start services with: just launchd-install'" C-m
fi

# Pane 4: Sidecar Logs - placeholder for status-sync logs
tmux send-keys -t "$SESSION:0.4" "# Sidecar Logs - status-sync output" C-m
tmux send-keys -t "$SESSION:0.4" "cd ${WORKDIR}" C-m
tmux send-keys -t "$SESSION:0.4" "echo 'Sidecar logs will appear during test run'" C-m
tmux send-keys -t "$SESSION:0.4" "echo 'Touch sidecar.log to start tailing:'" C-m
tmux send-keys -t "$SESSION:0.4" "echo '  touch sidecar.log && tail -f sidecar.log'" C-m

# Select top pane (coordinator) for user interaction
tmux select-pane -t "$SESSION:0.0"

echo -e "${GREEN}✓ TMUX session '$SESSION' created successfully!${NC}"
echo ""
echo "To connect: tmux attach -t $SESSION"
echo "To kill:    tmux kill-session -t $SESSION"
echo ""
echo "Panes:"
echo "  0 (top):          Swarm Coordinator - run claudesp swarm here"
echo "  1 (middle-left):  Intake Progress - progress.jsonl"
echo "  2 (middle-right): CLI Stream - claude-stream.jsonl"
echo "  3 (bottom-left):  Service Logs - pm-server + controller"
echo "  4 (bottom-right): Sidecar Logs - status-sync output"
