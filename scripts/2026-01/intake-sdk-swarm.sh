#!/usr/bin/env bash
# intake-sdk-swarm.sh - TMUX session for monitoring intake SDK migration swarm
#
# Usage: ./scripts/2026-01/intake-sdk-swarm.sh [start|attach|kill]
#
# This script creates a TMUX session with multiple panes for monitoring
# the parallel swarm agents working on the intake SDK migration.

set -euo pipefail

SESSION_NAME="intake-sdk-swarm"
PLAN_FILE="$HOME/.cursor/plans/intake_sdk_bun_compile_migration.plan.md"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() { echo -e "${BLUE}[INFO]${NC} $*"; }
log_success() { echo -e "${GREEN}[OK]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }

check_dependencies() {
    if ! command -v tmux &>/dev/null; then
        log_error "tmux is not installed. Install with: brew install tmux"
        exit 1
    fi
    if ! command -v bun &>/dev/null; then
        log_error "bun is not installed. Install with: curl -fsSL https://bun.sh/install | bash"
        exit 1
    fi
}

create_session() {
    if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
        log_warn "Session '$SESSION_NAME' already exists. Use 'attach' or 'kill' first."
        exit 1
    fi

    log_info "Creating TMUX session: $SESSION_NAME"
    
    # Create the session with the first window for the main orchestrator
    tmux new-session -d -s "$SESSION_NAME" -n "orchestrator" -c "$PROJECT_ROOT"
    
    # Window 0: Orchestrator - Main swarm coordinator
    tmux send-keys -t "$SESSION_NAME:orchestrator" "# Swarm Orchestrator - Main Coordinator" Enter
    tmux send-keys -t "$SESSION_NAME:orchestrator" "# Plan: $PLAN_FILE" Enter
    tmux send-keys -t "$SESSION_NAME:orchestrator" "clear && echo '🎯 Intake SDK Migration Swarm - Orchestrator'" Enter
    tmux send-keys -t "$SESSION_NAME:orchestrator" "echo ''" Enter
    tmux send-keys -t "$SESSION_NAME:orchestrator" "echo 'Plan: intake_sdk_bun_compile_migration.plan.md'" Enter
    tmux send-keys -t "$SESSION_NAME:orchestrator" "echo 'Tasks: 10 total, Groups A-H'" Enter
    tmux send-keys -t "$SESSION_NAME:orchestrator" "echo ''" Enter
    
    # Window 1: Nova Agent - TypeScript (Tasks 1-7)
    tmux new-window -t "$SESSION_NAME" -n "nova" -c "$PROJECT_ROOT/tools"
    tmux send-keys -t "$SESSION_NAME:nova" "# Nova Agent - TypeScript/Bun Specialist" Enter
    tmux send-keys -t "$SESSION_NAME:nova" "# Tasks: 1-7 (Foundation, Protocol, Operations, MCP, Build)" Enter
    tmux send-keys -t "$SESSION_NAME:nova" "clear && echo '📦 Nova Agent - TypeScript SDK Implementation'" Enter
    tmux send-keys -t "$SESSION_NAME:nova" "echo ''" Enter
    tmux send-keys -t "$SESSION_NAME:nova" "echo 'Assigned Tasks:'" Enter
    tmux send-keys -t "$SESSION_NAME:nova" "echo '  1. Setup TypeScript Project'" Enter
    tmux send-keys -t "$SESSION_NAME:nova" "echo '  2. Implement JSON Protocol Handler'" Enter
    tmux send-keys -t "$SESSION_NAME:nova" "echo '  3. Implement PRD Parsing Operation'" Enter
    tmux send-keys -t "$SESSION_NAME:nova" "echo '  4. Implement Task Expansion Operation'" Enter
    tmux send-keys -t "$SESSION_NAME:nova" "echo '  5. Implement Complexity Analysis'" Enter
    tmux send-keys -t "$SESSION_NAME:nova" "echo '  6. MCP Integration (Research Mode)'" Enter
    tmux send-keys -t "$SESSION_NAME:nova" "echo '  7. Build System & Binary Distribution'" Enter
    tmux send-keys -t "$SESSION_NAME:nova" "echo ''" Enter
    
    # Window 2: Rex Agent - Rust (Task 8)
    tmux new-window -t "$SESSION_NAME" -n "rex" -c "$PROJECT_ROOT/crates/intake"
    tmux send-keys -t "$SESSION_NAME:rex" "# Rex Agent - Rust Specialist" Enter
    tmux send-keys -t "$SESSION_NAME:rex" "# Task: 8 (Update Rust SDK Provider)" Enter
    tmux send-keys -t "$SESSION_NAME:rex" "clear && echo '🦀 Rex Agent - Rust Integration'" Enter
    tmux send-keys -t "$SESSION_NAME:rex" "echo ''" Enter
    tmux send-keys -t "$SESSION_NAME:rex" "echo 'Assigned Task:'" Enter
    tmux send-keys -t "$SESSION_NAME:rex" "echo '  8. Update sdk_provider.rs to call intake-agent binary'" Enter
    tmux send-keys -t "$SESSION_NAME:rex" "echo ''" Enter
    tmux send-keys -t "$SESSION_NAME:rex" "echo 'Key Files:'" Enter
    tmux send-keys -t "$SESSION_NAME:rex" "echo '  - src/ai/sdk_provider.rs'" Enter
    tmux send-keys -t "$SESSION_NAME:rex" "echo '  - src/ai/registry.rs'" Enter
    tmux send-keys -t "$SESSION_NAME:rex" "echo ''" Enter
    
    # Window 3: Tess Agent - Testing (Task 9)
    tmux new-window -t "$SESSION_NAME" -n "tess" -c "$PROJECT_ROOT"
    tmux send-keys -t "$SESSION_NAME:tess" "# Tess Agent - Testing Specialist" Enter
    tmux send-keys -t "$SESSION_NAME:tess" "# Task: 9 (Integration Testing)" Enter
    tmux send-keys -t "$SESSION_NAME:tess" "clear && echo '🧪 Tess Agent - Integration Testing'" Enter
    tmux send-keys -t "$SESSION_NAME:tess" "echo ''" Enter
    tmux send-keys -t "$SESSION_NAME:tess" "echo 'Assigned Task:'" Enter
    tmux send-keys -t "$SESSION_NAME:tess" "echo '  9. Integration Testing (after Nova + Rex complete)'" Enter
    tmux send-keys -t "$SESSION_NAME:tess" "echo ''" Enter
    tmux send-keys -t "$SESSION_NAME:tess" "echo 'Test Cases:'" Enter
    tmux send-keys -t "$SESSION_NAME:tess" "echo '  - PRD parsing end-to-end'" Enter
    tmux send-keys -t "$SESSION_NAME:tess" "echo '  - Task expansion'" Enter
    tmux send-keys -t "$SESSION_NAME:tess" "echo '  - Complexity analysis'" Enter
    tmux send-keys -t "$SESSION_NAME:tess" "echo '  - MCP research mode'" Enter
    tmux send-keys -t "$SESSION_NAME:tess" "echo ''" Enter
    
    # Window 4: Logs - Build output and errors
    tmux new-window -t "$SESSION_NAME" -n "logs" -c "$PROJECT_ROOT"
    tmux send-keys -t "$SESSION_NAME:logs" "# Build Logs & Monitoring" Enter
    tmux send-keys -t "$SESSION_NAME:logs" "clear && echo '📋 Build Logs'" Enter
    tmux send-keys -t "$SESSION_NAME:logs" "echo ''" Enter
    tmux send-keys -t "$SESSION_NAME:logs" "echo 'Commands:'" Enter
    tmux send-keys -t "$SESSION_NAME:logs" "echo '  cargo build --release -p intake  # Build Rust'" Enter
    tmux send-keys -t "$SESSION_NAME:logs" "echo '  bun run build                     # Build TS agent'" Enter
    tmux send-keys -t "$SESSION_NAME:logs" "echo '  cargo test -p intake              # Run tests'" Enter
    tmux send-keys -t "$SESSION_NAME:logs" "echo ''" Enter
    
    # Window 5: Plan - Read-only plan viewer
    tmux new-window -t "$SESSION_NAME" -n "plan" -c "$PROJECT_ROOT"
    tmux send-keys -t "$SESSION_NAME:plan" "# Migration Plan" Enter
    if [ -f "$PLAN_FILE" ]; then
        tmux send-keys -t "$SESSION_NAME:plan" "cat '$PLAN_FILE' | less" Enter
    else
        tmux send-keys -t "$SESSION_NAME:plan" "echo 'Plan file not found: $PLAN_FILE'" Enter
    fi
    
    # Go back to orchestrator window
    tmux select-window -t "$SESSION_NAME:orchestrator"
    
    log_success "TMUX session '$SESSION_NAME' created with 6 windows:"
    log_info "  0: orchestrator - Main swarm coordinator"
    log_info "  1: nova        - TypeScript agent (Tasks 1-7)"
    log_info "  2: rex         - Rust agent (Task 8)"
    log_info "  3: tess        - Testing agent (Task 9)"
    log_info "  4: logs        - Build output & monitoring"
    log_info "  5: plan        - Migration plan viewer"
    echo ""
    log_info "Attach with: tmux attach -t $SESSION_NAME"
    log_info "Switch windows: Ctrl-b + number (0-5)"
}

attach_session() {
    if ! tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
        log_error "Session '$SESSION_NAME' does not exist. Use 'start' first."
        exit 1
    fi
    tmux attach -t "$SESSION_NAME"
}

kill_session() {
    if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
        tmux kill-session -t "$SESSION_NAME"
        log_success "Session '$SESSION_NAME' killed."
    else
        log_warn "Session '$SESSION_NAME' does not exist."
    fi
}

show_usage() {
    echo "Usage: $0 [start|attach|kill]"
    echo ""
    echo "Commands:"
    echo "  start   - Create new TMUX session with agent windows"
    echo "  attach  - Attach to existing session"
    echo "  kill    - Kill existing session"
    echo ""
    echo "TMUX Keybindings:"
    echo "  Ctrl-b + number  - Switch to window N (0-5)"
    echo "  Ctrl-b + n       - Next window"
    echo "  Ctrl-b + p       - Previous window"
    echo "  Ctrl-b + d       - Detach from session"
    echo "  Ctrl-b + c       - Create new window"
    echo "  Ctrl-b + w       - List windows"
}

# Main
check_dependencies

case "${1:-}" in
    start)
        create_session
        ;;
    attach)
        attach_session
        ;;
    kill)
        kill_session
        ;;
    *)
        show_usage
        ;;
esac
