#!/bin/bash
# Run the Installer Agent (Claude) for Latitude bare metal installation
# Usage: ./run-installer.sh [--interactive]
#
# By default runs in unattended mode with --dangerously-skip-permissions
# Use --interactive for manual approval mode

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PROMPT_FILE="$SCRIPT_DIR/installer-prompt.md"
COORD_FILE="$SCRIPT_DIR/ralph-coordination.json"
INTERACTIVE=false

# Latitude.sh credentials (from Cursor MCP config)
export LATITUDE_API_KEY="${LATITUDE_API_KEY:-e70a9cf08b22b96a0f18050902959f97df38}"
export LATITUDE_PROJECT_ID="${LATITUDE_PROJECT_ID:-proj_bBmw0KKxQ09VR}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() {
  echo -e "${BLUE}[$(date '+%Y-%m-%d %H:%M:%S')]${NC} $1"
}

error() {
  echo -e "${RED}[ERROR]${NC} $1" >&2
}

success() {
  echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
  echo -e "${YELLOW}[WARN]${NC} $1"
}

# Parse arguments
parse_args() {
  while [[ $# -gt 0 ]]; do
    case $1 in
      --interactive)
        INTERACTIVE=true
        shift
        ;;
      *)
        error "Unknown option: $1"
        exit 1
        ;;
    esac
  done
}

# Check prerequisites
check_prereqs() {
  log "Checking prerequisites..."
  
  # Check Claude CLI
  if ! command -v claude &> /dev/null; then
    error "Claude CLI not found. Install with: npm install -g @anthropic-ai/claude-code"
    exit 1
  fi
  
  # Check MCP servers
  log "Checking MCP servers..."
  if ! claude mcp list 2>&1 | grep -q "latitude"; then
    error "Latitude MCP not configured. Run: claude mcp add latitude -- npx -y latitudesh start --bearer <API_KEY>"
    exit 1
  fi
  
  if ! claude mcp list 2>&1 | grep -q "talos-mcp"; then
    warn "Talos MCP not configured. Some operations may fail."
  fi
  
  # Check installer binary
  if [[ ! -f "$REPO_ROOT/target/release/installer" ]]; then
    log "Building installer binary..."
    cd "$REPO_ROOT"
    cargo build --release -p installer
  fi
  
  success "All prerequisites met"
}

# Update coordination state
update_coord() {
  local key="$1"
  local value="$2"
  local tmp=$(mktemp)
  jq "$key = $value" "$COORD_FILE" > "$tmp" && mv "$tmp" "$COORD_FILE"
}

# Initialize session
init_session() {
  local session_id=$(uuidgen | tr '[:upper:]' '[:lower:]')
  local now=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  
  update_coord '.session.id' "\"$session_id\""
  update_coord '.session.startedAt' "\"$now\""
  update_coord '.session.lastActivity' "\"$now\""
  update_coord '.installer.status' '"running"'
  update_coord '.installer.lastUpdate' "\"$now\""
  update_coord '.installer.pid' "$$"
  
  log "Session initialized: $session_id"
}

# Cleanup on exit
cleanup() {
  log "Cleaning up..."
  update_coord '.installer.status' '"stopped"'
  update_coord '.installer.pid' 'null'
  log "Installer agent stopped"
}

# Main
main() {
  parse_args "$@"
  
  log "=== Latitude Installer Agent (Claude) ==="
  log "Repo root: $REPO_ROOT"
  log "Prompt file: $PROMPT_FILE"
  log "Mode: $([ "$INTERACTIVE" = true ] && echo "interactive" || echo "unattended")"
  
  check_prereqs
  init_session
  
  # Set up cleanup trap
  trap cleanup EXIT
  
  # Build the initial prompt
  local initial_prompt="You are the Latitude Installer Agent. Read and follow the instructions in $PROMPT_FILE.

IMPORTANT: This is an UNATTENDED installation. Execute each step without asking for confirmation.

Current coordination state:
$(cat "$COORD_FILE" | jq -c .)

Key files:
- Installer binary: $REPO_ROOT/target/release/installer
- Coordination file: $COORD_FILE
- Progress log: $SCRIPT_DIR/progress.txt

Your tasks:
1. First, update progress.txt to log that you're starting
2. Check if there's existing state in /tmp/latitude-test/ to resume from
3. Run the installer binary with appropriate flags
4. Monitor progress and update the coordination file
5. On failure, collect diagnostics and update the issue queue

START NOW - do not ask for permission, just execute."

  log "Starting Claude agent in unattended mode..."
  warn "Using --dangerously-skip-permissions - all operations will be auto-approved"
  log "Press Ctrl+C to stop"
  
  cd "$REPO_ROOT"
  
  # Run Claude with appropriate flags
  if [ "$INTERACTIVE" = true ]; then
    claude "$initial_prompt"
  else
    # Unattended mode with full permissions
    claude --dangerously-skip-permissions "$initial_prompt"
  fi
}

main "$@"
