#!/bin/bash
# Cleanup script for Latitude installation
# Usage: ./cleanup.sh [--full]
#
# Options:
#   --full    Delete Latitude servers (requires MCP access)
#   --local   Only clean local state (default)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
COORD_FILE="$SCRIPT_DIR/ralph-coordination.json"
STATE_DIR="/tmp/latitude-test"

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

# Clean local state
clean_local() {
  log "Cleaning local state..."
  
  # Remove installer state directory
  if [[ -d "$STATE_DIR" ]]; then
    log "Removing $STATE_DIR"
    rm -rf "$STATE_DIR"
    success "Removed installer state directory"
  else
    log "No installer state directory found"
  fi
  
  # Reset coordination file
  log "Resetting coordination file..."
  cat > "$COORD_FILE" << 'EOF'
{
  "installer": {
    "status": "not_started",
    "currentStep": null,
    "stepNumber": 0,
    "totalSteps": 23,
    "lastUpdate": null,
    "lastError": null,
    "attemptCount": 0,
    "pid": null
  },
  "monitor": {
    "status": "idle",
    "lastCheck": null,
    "checkCount": 0,
    "pid": null
  },
  "issueQueue": [],
  "circuitBreaker": {
    "state": "closed",
    "failureCount": 0,
    "sameStepFailures": 0,
    "lastFailedStep": null,
    "lastError": null,
    "openedAt": null,
    "threshold": 3
  },
  "servers": {
    "controlPlane": null,
    "workers": []
  },
  "cluster": {
    "name": "latitude-test",
    "region": "DAL",
    "cpPlan": "c2-small-x86",
    "workerPlan": "c2-small-x86",
    "nodeCount": 2,
    "talosVersion": "v1.9.0",
    "kubeconfig": null,
    "talosconfig": null
  },
  "session": {
    "id": null,
    "startedAt": null,
    "lastActivity": null
  },
  "stats": {
    "totalIssues": 0,
    "resolvedIssues": 0,
    "failedAttempts": 0,
    "successfulSteps": 0,
    "totalDuration": null
  }
}
EOF
  success "Reset coordination file"
  
  # Clear progress log
  if [[ -f "$SCRIPT_DIR/progress.txt" ]]; then
    log "Clearing progress log..."
    echo "# Latitude Installation Progress" > "$SCRIPT_DIR/progress.txt"
    echo "" >> "$SCRIPT_DIR/progress.txt"
    echo "Cleaned at $(date -u +"%Y-%m-%dT%H:%M:%SZ")" >> "$SCRIPT_DIR/progress.txt"
    echo "" >> "$SCRIPT_DIR/progress.txt"
    success "Cleared progress log"
  fi
  
  # Remove merged kubeconfig context
  if command -v kubectl &> /dev/null; then
    if kubectl config get-contexts latitude-test &> /dev/null; then
      log "Removing kubeconfig context 'latitude-test'..."
      kubectl config delete-context latitude-test 2>/dev/null || true
      kubectl config delete-cluster latitude-test 2>/dev/null || true
      kubectl config delete-user admin@latitude-test 2>/dev/null || true
      success "Removed kubeconfig context"
    fi
  fi
}

# Clean Latitude servers (requires Claude MCP)
clean_servers() {
  log "Cleaning Latitude servers..."
  warn "This will DELETE all servers created by this installation!"
  
  read -p "Are you sure? (y/N) " -n 1 -r
  echo
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    log "Cancelled"
    return
  fi
  
  # Check if Claude CLI is available
  if ! command -v claude &> /dev/null; then
    error "Claude CLI not found. Cannot delete servers via MCP."
    error "Please delete servers manually via Latitude dashboard."
    return 1
  fi
  
  log "Use Claude with Latitude MCP to delete servers:"
  echo ""
  echo "  1. Run: claude"
  echo "  2. Ask: 'List all servers via Latitude MCP'"
  echo "  3. For each server with 'latitude-test' in hostname, ask: 'Delete server <id>'"
  echo ""
  echo "Or delete via Latitude dashboard: https://app.latitude.sh"
  echo ""
  
  # Also check coordination file for known servers
  if [[ -f "$COORD_FILE" ]]; then
    local cp_id=$(jq -r '.servers.controlPlane.id // empty' "$COORD_FILE")
    local worker_ids=$(jq -r '.servers.workers[].id // empty' "$COORD_FILE")
    
    if [[ -n "$cp_id" ]] || [[ -n "$worker_ids" ]]; then
      echo "Known servers from coordination file:"
      [[ -n "$cp_id" ]] && echo "  - Control plane: $cp_id"
      for wid in $worker_ids; do
        echo "  - Worker: $wid"
      done
      echo ""
    fi
  fi
}

# Print usage
usage() {
  echo "Usage: $0 [--full|--local]"
  echo ""
  echo "Options:"
  echo "  --local    Clean only local state (default)"
  echo "  --full     Clean local state AND guide through server deletion"
  echo ""
  echo "Local state includes:"
  echo "  - /tmp/latitude-test/ (installer state)"
  echo "  - ralph-coordination.json (reset to defaults)"
  echo "  - progress.txt (cleared)"
  echo "  - kubeconfig context 'latitude-test' (removed)"
}

# Main
main() {
  local mode="local"
  
  while [[ $# -gt 0 ]]; do
    case $1 in
      --full)
        mode="full"
        shift
        ;;
      --local)
        mode="local"
        shift
        ;;
      --help|-h)
        usage
        exit 0
        ;;
      *)
        error "Unknown option: $1"
        usage
        exit 1
        ;;
    esac
  done
  
  log "=== Latitude Installation Cleanup ==="
  log "Mode: $mode"
  
  if [[ "$mode" == "full" ]]; then
    clean_servers
  fi
  
  clean_local
  
  success "Cleanup complete!"
  echo ""
  log "To start fresh, run: ./run-installer.sh"
}

main "$@"
