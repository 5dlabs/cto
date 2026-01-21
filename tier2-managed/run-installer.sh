#!/bin/bash
# Unified E2E Installer Agent
# Provisions Admin CTO, deploys platform, runs BoltRun verification, and Client CTO
#
# Usage: ./run-installer.sh [--interactive]
#
# By default runs in unattended mode for 3-4 hours

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PROMPT_FILE="$SCRIPT_DIR/installer-prompt.md"
COORD_FILE="$SCRIPT_DIR/ralph-coordination.json"
PRD_FILE="$SCRIPT_DIR/prd.json"
INTERACTIVE=false

# Source environment variables (including API keys)
if [[ -f "$REPO_ROOT/.env.local" ]]; then
  source "$REPO_ROOT/.env.local"
fi

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
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

usage() {
  cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Launch the Unified E2E Installer Agent.

This agent runs UNATTENDED for 3-4 hours, implementing:
  1. Pre-Flight checks
  2. Admin CTO Infrastructure (servers, VLAN)
  3. Admin CTO Talos installation
  4. Admin CTO Kubernetes bootstrap
  5. Admin CTO GitOps (ArgoCD)
  6. Platform Stack (secrets, controller, web app)
  7. BoltRun E2E verification
  8. UI Testing (Agent Browser)
  9. Client CTO provisioning (via BoltRun)
  10. Connectivity (WARP + ClusterMesh)
  11. Final verification

Options:
    --interactive       Run in interactive mode (prompts for approval)
    -h, --help          Show this help message

Examples:
    ./run-installer.sh
    ./run-installer.sh --interactive
EOF
  exit 0
}

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --interactive)
      INTERACTIVE=true
      shift
      ;;
    -h|--help)
      usage
      ;;
    *)
      error "Unknown option: $1"
      usage
      ;;
  esac
done

# Check prerequisites
check_prereqs() {
  log "Checking prerequisites..."
  
  # Check Claude CLI
  if ! command -v claude &> /dev/null; then
    error "Claude CLI not found. Install with: npm install -g @anthropic-ai/claude-code"
    exit 1
  fi
  
  # Check required tools
  local missing_tools=()
  command -v talosctl &> /dev/null || missing_tools+=("talosctl")
  command -v kubectl &> /dev/null || missing_tools+=("kubectl")
  command -v helm &> /dev/null || missing_tools+=("helm")
  
  if [[ ${#missing_tools[@]} -gt 0 ]]; then
    warn "Missing tools: ${missing_tools[*]}"
    warn "These will be checked by the agent during pre-flight"
  fi
  
  # Check API key
  if [[ -z "${LATITUDE_API_KEY:-}" ]]; then
    warn "LATITUDE_API_KEY not set - will need to be sourced from .env.local"
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
  update_coord '.session.installerCli' '"claude"'
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
  echo -e "${CYAN}╔═══════════════════════════════════════════════════════════════╗${NC}"
  echo -e "${CYAN}║         UNIFIED E2E INSTALLER AGENT (Claude)                  ║${NC}"
  echo -e "${CYAN}╚═══════════════════════════════════════════════════════════════╝${NC}"
  
  log "Repo root: $REPO_ROOT"
  log "Mode: $([ "$INTERACTIVE" = true ] && echo "interactive" || echo "unattended")"
  log "Estimated duration: 3-4 hours"
  
  check_prereqs
  init_session
  
  # Set up cleanup trap
  trap cleanup EXIT
  
  # Build the initial prompt
  local initial_prompt="You are the Unified E2E Installer Agent. Read and follow the instructions in $PROMPT_FILE.

IMPORTANT: This is an UNATTENDED installation running for 3-4 hours. Execute each step without asking for confirmation.

Current coordination state:
$(cat "$COORD_FILE" | jq -c .)

Key files:
- PRD: $PRD_FILE (contains ALL user stories for all phases)
- Prompt: $PROMPT_FILE (detailed implementation instructions)
- Coordination: $COORD_FILE (shared state with monitor)
- Progress: $SCRIPT_DIR/progress.txt

PHASES TO IMPLEMENT (in order):
1. Pre-Flight (PRE-001 to PRE-003) - Verify tools and API access
2. Admin CTO Infrastructure (ADMIN-INF-*) - Create servers in DAL region
3. Admin CTO Talos (ADMIN-TALOS-*) - Install Talos Linux
4. Admin CTO Kubernetes (ADMIN-K8S-*) - Bootstrap cluster
5. Admin CTO GitOps (ADMIN-GITOPS-*) - Deploy ArgoCD
6. Platform (PLATFORM-*) - Deploy secrets, controller, web app
7. BoltRun (BOLT-*) - Verify CRD and controller
8. UI Testing (UI-*) - Use browser automation for onboarding
9. Client CTO (CLIENT-INF-*) - BoltRun provisions customer cluster
10. Connectivity (CONN-*) - WARP Connector and ClusterMesh
11. Verification (VERIFY-*) - Final E2E testing

IMPORTANT RULES:
- Single-region only: ALL servers in DAL (Admin CTO) or NYC (Client CTO)
- Update coordination state after each story
- Log progress to progress.txt
- Mark stories passes: true only after VERIFICATION criteria met
- If blocked > 30 min, skip and document

START NOW - iterate through prd.json stories until ALL have passes: true"

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
