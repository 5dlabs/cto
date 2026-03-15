#!/usr/bin/env bash
# =========================================================================
# Docker-based CLI Integration Test
# 
# Runs actual agent images from GHCR for accurate production testing.
# This is the most accurate way to test CLI invocation locally.
#
# Prerequisites:
#   - Docker running (colima start)
#   - Registry authentication (echo $GITHUB_TOKEN | docker login registry.5dlabs.ai -u <user> --password-stdin)
#   - Twingate connected (for tools server access)
#
# Usage:
#   ./tests/cli-invocation/docker-test.sh [cli] [prompt]
#
# Examples:
#   ./tests/cli-invocation/docker-test.sh claude "Say hello"
#   ./tests/cli-invocation/docker-test.sh          # Interactive mode
# =========================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"

# Configuration
GHCR_REGISTRY="registry.5dlabs.ai/5dlabs"
TOOLS_SERVER_URL="${TOOLS_SERVER_URL:-http://cto-tools.cto.svc.cluster.local:3000}"
WORKSPACE_DIR="${WORKSPACE_DIR:-/tmp/cto-docker-test}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() { echo -e "${BLUE}[TEST]${NC} $1"; }
success() { echo -e "${GREEN}[PASS]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[FAIL]${NC} $1"; }

# CLI to image mapping
get_image_for_cli() {
    local cli="$1"
    case "$cli" in
        claude)  echo "$GHCR_REGISTRY/claude:latest" ;;
        codex)   echo "$GHCR_REGISTRY/runtime:latest" ;;  # Uses base runtime
        cursor)  echo "$GHCR_REGISTRY/runtime:latest" ;;
        opencode) echo "$GHCR_REGISTRY/runtime:latest" ;;
        gemini)  echo "$GHCR_REGISTRY/runtime:latest" ;;
        factory) echo "$GHCR_REGISTRY/runtime:latest" ;;
        code)    echo "$GHCR_REGISTRY/runtime:latest" ;;
        dexter)  echo "$GHCR_REGISTRY/runtime:latest" ;;
        *)       echo "" ;;
    esac
}

# Check prerequisites
check_prereqs() {
    log "Checking prerequisites..."
    
    # Check Docker
    if ! docker info >/dev/null 2>&1; then
        error "Docker is not running"
        log "Start Docker with: colima start"
        exit 1
    fi
    success "Docker is running"
    
    # Check GHCR auth
    if ! docker pull "$GHCR_REGISTRY/runtime:latest" --quiet >/dev/null 2>&1; then
        warn "May need registry authentication"
        log "Run: echo \$GITHUB_TOKEN | docker login registry.5dlabs.ai -u YOUR_USERNAME --password-stdin"
    fi
    
    # Check tools server
    if curl -sf "$TOOLS_SERVER_URL/health" >/dev/null 2>&1; then
        success "Tools server reachable at $TOOLS_SERVER_URL"
    else
        warn "Tools server not reachable - ensure Twingate is connected"
    fi
}

# Create MCP config for container
create_mcp_config() {
    local config_dir="$1"
    mkdir -p "$config_dir"
    
    cat > "$config_dir/mcp.json" << EOF
{
  "mcpServers": {
    "cto-tools": {
      "url": "${TOOLS_SERVER_URL}/mcp",
      "transport": {
        "type": "sse"
      }
    }
  }
}
EOF
    echo "$config_dir/mcp.json"
}

# Create prompt file
create_prompt() {
    local workspace="$1"
    local prompt="$2"
    
    mkdir -p "$workspace/task"
    echo "$prompt" > "$workspace/task/prompt.md"
}

# Run CLI in Docker container
run_cli_docker() {
    local cli="${1:-claude}"
    local prompt="${2:-Say hello and report your model name}"
    
    local image
    image=$(get_image_for_cli "$cli")
    
    if [[ -z "$image" ]]; then
        error "Unknown CLI: $cli"
        exit 1
    fi
    
    log "=== Docker CLI Test: $cli ==="
    log "Image: $image"
    log "Prompt: $prompt"
    
    # Setup workspace
    local workspace="$WORKSPACE_DIR/$cli"
    rm -rf "$workspace"
    mkdir -p "$workspace"
    
    # Create MCP config
    local mcp_config
    mcp_config=$(create_mcp_config "$workspace/mcp-config")
    log "MCP Config: $mcp_config"
    
    # Create prompt
    create_prompt "$workspace" "$prompt"
    log "Prompt file: $workspace/task/prompt.md"
    
    # Create output directory
    mkdir -p "$workspace/output"
    
    # Pull image if needed
    log "Pulling image..."
    docker pull "$image" --quiet || {
        error "Failed to pull image: $image"
        exit 1
    }
    success "Image ready"
    
    # Build CLI command based on type
    local cli_cmd=""
    case "$cli" in
        claude)
            cli_cmd="claude --print --output-format stream-json --verbose \
                --mcp-config /mcp-config/mcp.json --strict-mcp-config \
                --dangerously-skip-permissions \
                \"\$(cat /workspace/task/prompt.md)\""
            ;;
        *)
            error "CLI $cli not yet implemented"
            exit 1
            ;;
    esac
    
    log ""
    log "═══════════════════════════════════════════════════════════════"
    log "║               DOCKER EXECUTION START                         ║"
    log "═══════════════════════════════════════════════════════════════"
    log ""
    
    # Run container
    # Note: Using host network for Twingate DNS resolution
    docker run --rm \
        --name "cto-cli-test-$cli" \
        --network host \
        -v "$workspace:/workspace" \
        -v "$workspace/mcp-config:/mcp-config:ro" \
        -e "ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY:-}" \
        -e "CLAUDE_API_KEY=${CLAUDE_API_KEY:-}" \
        -w /workspace \
        "$image" \
        bash -c "$cli_cmd" 2>&1 | tee "$workspace/output/stream.jsonl"
    
    local exit_code=${PIPESTATUS[0]}
    
    log ""
    log "═══════════════════════════════════════════════════════════════"
    log "║               DOCKER EXECUTION COMPLETE                      ║"
    log "═══════════════════════════════════════════════════════════════"
    log ""
    
    if [[ $exit_code -eq 0 ]]; then
        success "CLI execution completed successfully"
    else
        error "CLI execution failed with exit code: $exit_code"
    fi
    
    log "Output: $workspace/output/stream.jsonl"
    log "Lines: $(wc -l < "$workspace/output/stream.jsonl" | tr -d ' ')"
    
    # Show sample output
    log ""
    log "Sample output (first 10 lines):"
    head -10 "$workspace/output/stream.jsonl" 2>/dev/null || echo "(empty)"
    
    return $exit_code
}

# Main
main() {
    echo ""
    echo "=============================================="
    echo "  Docker-based CLI Integration Test"
    echo "=============================================="
    echo ""
    
    check_prereqs
    echo ""
    
    local cli="${1:-claude}"
    local prompt="${2:-Say hello and report your model name}"
    
    run_cli_docker "$cli" "$prompt"
}

main "$@"
