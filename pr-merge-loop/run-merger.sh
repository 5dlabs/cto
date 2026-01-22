#!/bin/bash
# Run the Merger Agent (Claude) for PR merging
# Usage: ./run-merger.sh [--interactive]
#
# By default runs in unattended mode with --dangerously-skip-permissions
# Use --interactive for manual approval mode

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PROMPT_FILE="$SCRIPT_DIR/merger-prompt.md"
COORD_FILE="$SCRIPT_DIR/ralph-coordination.json"
INTERACTIVE=false

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
  
  # Check GitHub CLI
  if ! command -v gh &> /dev/null; then
    error "GitHub CLI not found. Install with: brew install gh"
    exit 1
  fi
  
  # Check GitHub authentication
  if ! gh auth status &> /dev/null; then
    error "GitHub not authenticated. Run: gh auth login"
    exit 1
  fi
  
  # Check git
  if ! command -v git &> /dev/null; then
    error "Git not found"
    exit 1
  fi
  
  # Check we're in the right repo
  if [[ ! -f "$REPO_ROOT/.git/config" ]]; then
    error "Not in a git repository. Run from CTO repo root."
    exit 1
  fi
  
  success "All prerequisites met"
}

# Initialize coordination file if it doesn't exist
init_coord() {
  if [[ ! -f "$COORD_FILE" ]]; then
    log "Initializing coordination file..."
    cat > "$COORD_FILE" <<EOF
{
  "merger": {
    "status": "not_started",
    "currentPr": null,
    "lastUpdate": null,
    "prsProcessed": 0,
    "prsMerged": 0,
    "prsFailed": 0,
    "pid": null
  },
  "monitor": {
    "status": "not_started",
    "lastCheck": null,
    "fixesImplemented": 0,
    "pid": null
  },
  "hardeningActions": [],
  "circuitBreaker": {
    "state": "closed",
    "failureCount": 0,
    "threshold": 3
  },
  "session": {
    "id": null,
    "startedAt": null,
    "lastActivity": null
  }
}
EOF
    success "Coordination file initialized"
  fi
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
  update_coord '.merger.status' '"running"'
  update_coord '.merger.lastUpdate' "\"$now\""
  update_coord '.merger.pid' "$$"
  
  log "Session initialized: $session_id"
}

# Cleanup on exit
cleanup() {
  log "Cleaning up..."
  update_coord '.merger.status' '"stopped"'
  update_coord '.merger.pid' 'null'
  log "Merger agent stopped"
}

# Main
main() {
  parse_args "$@"
  
  log "=== PR Merger Agent (Claude) ==="
  log "Repo root: $REPO_ROOT"
  log "Prompt file: $PROMPT_FILE"
  log "Mode: $([ "$INTERACTIVE" = true ] && echo "interactive" || echo "unattended")"
  
  check_prereqs
  init_coord
  init_session
  
  # Set up cleanup trap
  trap cleanup EXIT
  
  # Build the initial prompt
  local initial_prompt="You are the PR Merger Agent. Read and follow the instructions in $PROMPT_FILE.

IMPORTANT: This is an UNATTENDED continuous loop. Execute each step without asking for confirmation.
The loop should run FOREVER - process PRs, wait 5 minutes, repeat.

Current coordination state:
$(cat "$COORD_FILE" | jq -c .)

Key files:
- Coordination file: $COORD_FILE
- Progress log: $SCRIPT_DIR/progress.txt
- Lessons learned: $SCRIPT_DIR/lessons-learned.md (may not exist yet)

Your tasks:
1. First, update progress.txt to log that you're starting
2. Enter an infinite loop:
   a. List all open PRs that need work
   b. For each PR: fix conflicts, bug-bot comments, CI failures
   c. Merge when ready
   d. Wait 5 minutes
   e. Repeat
3. Update the coordination file after each PR processed
4. Log everything to progress.txt for the Monitor Agent to observe

START NOW - do not ask for permission, just execute the infinite loop."

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