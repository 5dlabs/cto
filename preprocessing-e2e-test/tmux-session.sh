#!/bin/bash
# Preprocessing Pipeline E2E Test - TMux Session Setup
#
# Creates a tmux session with 10 panes for monitoring each subagent.
# Run ./loop.sh in a separate terminal to start the Ralph loop.
#
# Usage: ./tmux-session.sh [--attach]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SESSION_NAME="preprocessing-e2e"

# Kill existing session if it exists
tmux kill-session -t "$SESSION_NAME" 2>/dev/null || true

# Create new session; first pane = oauth-agent
tmux new-session -d -s "$SESSION_NAME" -n "swarm" -c "$SCRIPT_DIR"

# Layout (3x4 grid for 10 panes + 2 empty):
# ┌─────────────────────┬─────────────────────┬─────────────────────┐
# │  1. oauth-agent     │  2. environment     │  3. intake-mcp       │
# ├─────────────────────┼─────────────────────┼─────────────────────┤
# │  4. tools-validation│  5. linear-sync     │  6. linear-update    │
# ├─────────────────────┼─────────────────────┼─────────────────────┤
# │  7. parity          │  8. critic-observer │  9. failback         │
# ├─────────────────────┼─────────────────────┼─────────────────────┤
# │ 10. features        │       (empty)       │       (empty)        │
# └─────────────────────┴─────────────────────┴─────────────────────┘

# Create 3 columns
tmux split-window -h -t "$SESSION_NAME:swarm" -c "$SCRIPT_DIR"
tmux split-window -h -t "$SESSION_NAME:swarm" -c "$SCRIPT_DIR"

# Split left column (0) vertically 3 times -> 0, 4, 7, 9
tmux select-pane -t "$SESSION_NAME:swarm.0"
tmux split-window -v -t "$SESSION_NAME:swarm" -c "$SCRIPT_DIR"
tmux split-window -v -t "$SESSION_NAME:swarm" -c "$SCRIPT_DIR"
tmux split-window -v -t "$SESSION_NAME:swarm" -c "$SCRIPT_DIR"

# Split middle column (1) vertically 3 times -> 1, 5, 8
tmux select-pane -t "$SESSION_NAME:swarm.1"
tmux split-window -v -t "$SESSION_NAME:swarm" -c "$SCRIPT_DIR"
tmux split-window -v -t "$SESSION_NAME:swarm" -c "$SCRIPT_DIR"

# Split right column (2) vertically 3 times -> 2, 6, 10
tmux select-pane -t "$SESSION_NAME:swarm.2"
tmux split-window -v -t "$SESSION_NAME:swarm" -c "$SCRIPT_DIR"
tmux split-window -v -t "$SESSION_NAME:swarm" -c "$SCRIPT_DIR"

# Pane 0: oauth-agent
tmux send-keys -t "$SESSION_NAME:swarm.0" "cd '$SCRIPT_DIR' && echo '=== OAuth Agent ===' && tail -f issues/issues-oauth.md 2>/dev/null || echo 'Waiting for issues...'" Enter

# Pane 1: environment-agent
tmux send-keys -t "$SESSION_NAME:swarm.1" "cd '$SCRIPT_DIR' && echo '=== Environment Agent ===' && tail -f issues/issues-environment.md 2>/dev/null || echo 'Waiting for issues...'" Enter

# Pane 2: intake-mcp-agent
tmux send-keys -t "$SESSION_NAME:swarm.2" "cd '$SCRIPT_DIR' && echo '=== Intake MCP Agent ===' && tail -f issues/issues-intake-mcp.md 2>/dev/null || echo 'Waiting for issues...'" Enter

# Pane 3: tools-validation-agent
tmux send-keys -t "$SESSION_NAME:swarm.3" "cd '$SCRIPT_DIR' && echo '=== Tools Validation Agent ===' && tail -f issues/issues-tools-validation.md 2>/dev/null || echo 'Waiting for issues...'" Enter

# Pane 4: linear-sync-agent
tmux send-keys -t "$SESSION_NAME:swarm.4" "cd '$SCRIPT_DIR' && echo '=== Linear Sync Agent ===' && tail -f issues/issues-linear-sync.md 2>/dev/null || echo 'Waiting for issues...'" Enter

# Pane 5: critic-observer-agent
tmux send-keys -t "$SESSION_NAME:swarm.5" "cd '$SCRIPT_DIR' && echo '=== Critic Observer Agent ===' && tail -f issues/issues-critic-observer.md 2>/dev/null || echo 'Waiting for issues...'" Enter

# Pane 6: linear-update-agent
tmux send-keys -t "$SESSION_NAME:swarm.6" "cd '$SCRIPT_DIR' && echo '=== Linear Update Agent ===' && tail -f issues/issues-linear-update.md 2>/dev/null || echo 'Waiting for issues...'" Enter

# Pane 7: parity-agent
tmux send-keys -t "$SESSION_NAME:swarm.7" "cd '$SCRIPT_DIR' && echo '=== Parity Agent ===' && tail -f issues/issues-parity.md 2>/dev/null || echo 'Waiting for issues...'" Enter

# Pane 8: failback-agent
tmux send-keys -t "$SESSION_NAME:swarm.8" "cd '$SCRIPT_DIR' && echo '=== Failback Agent ===' && tail -f issues/issues-failback.md 2>/dev/null || echo 'Waiting for issues...'" Enter

# Pane 9: features-agent
tmux send-keys -t "$SESSION_NAME:swarm.9" "cd '$SCRIPT_DIR' && echo '=== Features Agent ===' && tail -f issues/issues-features.md 2>/dev/null || echo 'Waiting for issues...' && echo 'Reading features backlog...' && head -30 features.md" Enter

tmux select-layout -t "$SESSION_NAME:swarm" tiled
tmux select-pane -t "$SESSION_NAME:swarm.0"

# Second window: logs
tmux new-window -t "$SESSION_NAME" -n "logs" -c "$SCRIPT_DIR"
tmux split-window -h -t "$SESSION_NAME:logs" -c "$SCRIPT_DIR"
tmux split-window -v -t "$SESSION_NAME:logs.0" -c "$SCRIPT_DIR"
tmux split-window -v -t "$SESSION_NAME:logs.2" -c "$SCRIPT_DIR"
tmux send-keys -t "$SESSION_NAME:logs.0" "tail -f /tmp/cto-launchd/pm-server.log 2>/dev/null || echo 'PM Server log not found'" Enter
tmux send-keys -t "$SESSION_NAME:logs.1" "tail -f /tmp/cto-launchd/controller.log 2>/dev/null || echo 'Controller log not found'" Enter
tmux send-keys -t "$SESSION_NAME:logs.2" "cd '$SCRIPT_DIR' && watch -n 2 'jq \".milestones\" ralph-coordination.json'" Enter
tmux send-keys -t "$SESSION_NAME:logs.3" "cd '$SCRIPT_DIR' && watch -n 5 'find issues -name \"*.md\" -exec grep -l OPEN {} \\; 2>/dev/null | wc -l | xargs echo \"Open issues:\"'" Enter
tmux select-layout -t "$SESSION_NAME:logs" tiled

# Third window: coordination state (including failback)
tmux new-window -t "$SESSION_NAME" -n "state" -c "$SCRIPT_DIR"
tmux send-keys -t "$SESSION_NAME:state" "cd '$SCRIPT_DIR' && watch -n 3 'jq \".\" ralph-coordination.json'" Enter

tmux select-window -t "$SESSION_NAME:swarm"

echo ""
echo "=============================================="
echo "TMux session '$SESSION_NAME' created!"
echo "=============================================="
echo ""
echo "Attach with: tmux attach -t $SESSION_NAME"
echo ""
echo "Windows:"
echo "  0: swarm   - 10 agent panes (oauth, environment, intake-mcp, tools-validation,"
echo "              linear-sync, linear-update, parity, critic-observer, failback, features)"
echo "  1: logs    - Service logs and status"
echo "  2: state   - Coordination state (incl. failback)"
echo ""
echo "Navigation:"
echo "  Ctrl+b, 0/1/2  - Switch windows"
echo "  Ctrl+b, arrow  - Switch panes"
echo "  Ctrl+b, z      - Zoom pane (toggle)"
echo "  Ctrl+b, d      - Detach"
echo ""
echo "Run ./loop.sh in another terminal to start the Ralph loop."
echo ""

if [[ "${1:-}" == "--attach" ]]; then
    tmux attach -t "$SESSION_NAME"
fi
