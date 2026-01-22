#!/usr/bin/env bash
set -euo pipefail

# Unified E2E - tmux Session Launcher
# Admin CTO Provisioning + Platform + BoltRun + Client CTO
# Spawns Claude (Installer), Droid (Monitor), and state viewer in split panes

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SESSION_NAME="unified-e2e"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Launch Unified E2E dual-agent loop in tmux with 3 panes:
  - Left:         Installer Agent (Claude) - Implements all phases
  - Right top:    Monitor Agent (Droid) - Verifies gates and progress
  - Right bottom: State viewer (watch ralph-coordination.json)

Phases covered (3-4 hours total):
  1. Pre-Flight checks
  2. Admin CTO Infrastructure (DAL region)
  3. Admin CTO Talos installation
  4. Admin CTO Kubernetes bootstrap
  5. Admin CTO GitOps (ArgoCD)
  6. Platform Stack deployment
  7. BoltRun E2E verification
  8. UI Testing (Agent Browser)
  9. Client CTO provisioning (via BoltRun)
  10. Connectivity (WARP + ClusterMesh)
  11. Final verification

Options:
    --attach                Attach to existing session if it exists
    -h, --help              Show this help message

Examples:
    # Start the unified E2E loop
    ./run-tmux.sh

    # Attach to existing session
    ./run-tmux.sh --attach
EOF
    exit 0
}

ATTACH_ONLY=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --attach)
            ATTACH_ONLY=true
            shift
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            usage
            ;;
    esac
done

# Check if tmux is installed
if ! command -v tmux &>/dev/null; then
    echo -e "${RED}Error: tmux is not installed. Install with: brew install tmux${NC}"
    exit 1
fi

# Attach to existing session if requested
if [ "$ATTACH_ONLY" = true ]; then
    if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
        echo -e "${GREEN}Attaching to existing session: $SESSION_NAME${NC}"
        tmux attach-session -t "$SESSION_NAME"
        exit 0
    else
        echo -e "${RED}No existing session found: $SESSION_NAME${NC}"
        exit 1
    fi
fi

# Kill existing session if it exists
if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
    echo -e "${YELLOW}Killing existing session: $SESSION_NAME${NC}"
    tmux kill-session -t "$SESSION_NAME"
fi

echo -e "${CYAN}╔═══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║         UNIFIED E2E: Admin CTO + Tier 2 Managed               ║${NC}"
echo -e "${CYAN}╠═══════════════════════════════════════════════════════════════╣${NC}"
echo -e "${CYAN}║  Session:    ${NC}$SESSION_NAME"
echo -e "${CYAN}║  Installer:  ${NC}Claude (implements all phases)"
echo -e "${CYAN}║  Monitor:    ${NC}Droid (verifies gates and progress)"
echo -e "${CYAN}║  Duration:   ${NC}~3-4 hours (unattended)"
echo -e "${CYAN}╠═══════════════════════════════════════════════════════════════╣${NC}"
echo -e "${CYAN}║  Phases:                                                      ║${NC}"
echo -e "${CYAN}║    1. Pre-Flight → 2. Admin Infrastructure → 3. Talos       ║${NC}"
echo -e "${CYAN}║    4. Kubernetes → 5. GitOps → 6. Platform → 7. BoltRun     ║${NC}"
echo -e "${CYAN}║    8. UI Testing → 9. Client CTO → 10. Connectivity         ║${NC}"
echo -e "${CYAN}║    11. Final Verification                                    ║${NC}"
echo -e "${CYAN}╚═══════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Create new tmux session with installer pane
tmux new-session -d -s "$SESSION_NAME" -c "$SCRIPT_DIR/.."

# Rename the first window
tmux rename-window -t "$SESSION_NAME:0" "agents"

# Split vertically (left/right)
tmux split-window -h -t "$SESSION_NAME:0" -c "$SCRIPT_DIR/.."

# Split right pane horizontally (top/bottom)
tmux split-window -v -t "$SESSION_NAME:0.1" -c "$SCRIPT_DIR"

# Pane layout:
# ┌─────────────────┬─────────────────┐
# │                 │   Monitor       │
# │   Installer     │   (pane 1)      │
# │   (pane 0)      ├─────────────────┤
# │                 │   State Viewer  │
# │                 │   (pane 2)      │
# └─────────────────┴─────────────────┘

# Pane 0: Installer Agent (Claude)
tmux send-keys -t "$SESSION_NAME:0.0" "cd '$SCRIPT_DIR/..' && echo '=== Unified E2E Installer Agent (Claude) ===' && sleep 2 && ./tier2-managed/run-installer.sh" Enter

# Pane 1: Monitor Agent (Droid) - with delay to let installer start
tmux send-keys -t "$SESSION_NAME:0.1" "cd '$SCRIPT_DIR/..' && echo '=== Unified E2E Monitor Agent (Droid) ===' && sleep 5 && ./tier2-managed/run-monitor.sh" Enter

# Pane 2: State viewer
tmux send-keys -t "$SESSION_NAME:0.2" "cd '$SCRIPT_DIR' && echo '=== State Viewer ===' && watch -n 2 'cat ralph-coordination.json | jq -C . 2>/dev/null || cat ralph-coordination.json'" Enter

# Set pane titles (requires tmux 3.2+)
tmux select-pane -t "$SESSION_NAME:0.0" -T "Installer (Claude)"
tmux select-pane -t "$SESSION_NAME:0.1" -T "Monitor (Droid)"
tmux select-pane -t "$SESSION_NAME:0.2" -T "State"

# Select the installer pane
tmux select-pane -t "$SESSION_NAME:0.0"

# Attach to the session
echo -e "${GREEN}Attaching to tmux session...${NC}"
echo ""
echo "tmux shortcuts:"
echo "  Ctrl+b then arrow keys - switch panes"
echo "  Ctrl+b then d          - detach (agents keep running)"
echo "  Ctrl+b then z          - zoom current pane"
echo "  Ctrl+b then x          - kill current pane"
echo ""
echo -e "${YELLOW}The agents will run unattended for ~3-4 hours.${NC}"
echo -e "${YELLOW}Detach with Ctrl+b then d to let them continue in background.${NC}"
echo ""

tmux attach-session -t "$SESSION_NAME"
