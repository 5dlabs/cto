#!/usr/bin/env bash
# =========================================================================
# Build and Test CLI Integration Locally with Docker
#
# Builds images locally for Apple Silicon (arm64) and runs the full
# CLI + sidecar integration test.
#
# Usage:
#   ./tests/cli-invocation/build-and-test.sh [build|test|all]
#
# Examples:
#   ./tests/cli-invocation/build-and-test.sh build  # Build images only
#   ./tests/cli-invocation/build-and-test.sh test   # Run test (assumes images built)
#   ./tests/cli-invocation/build-and-test.sh        # Build and test
# =========================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"

# Configuration
TOOLS_SERVER_URL="${TOOLS_SERVER_URL:-http://cto-tools.cto.svc.cluster.local:3000}"
WORKSPACE_DIR="${WORKSPACE_DIR:-/tmp/cto-docker-test}"

# Image names for local builds
RUNTIME_IMAGE="cto-runtime:local"
CLAUDE_IMAGE="cto-claude:local"
SIDECAR_IMAGE="cto-linear-sidecar:local"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() { echo -e "${BLUE}[BUILD]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Check Docker is running
check_docker() {
    if ! docker info >/dev/null 2>&1; then
        error "Docker is not running"
        log "Start with: colima start"
        exit 1
    fi
    success "Docker is running"
}

# Build runtime base image
build_runtime() {
    log "Building runtime base image..."
    docker build \
        -t "$RUNTIME_IMAGE" \
        -f "$PROJECT_ROOT/infra/images/runtime/Dockerfile" \
        "$PROJECT_ROOT/infra/images/runtime"
    success "Runtime image built: $RUNTIME_IMAGE"
}

# Build Claude image (on top of runtime)
build_claude() {
    log "Building Claude image..."
    docker build \
        -t "$CLAUDE_IMAGE" \
        --build-arg "BASE_IMAGE=$RUNTIME_IMAGE" \
        -f "$PROJECT_ROOT/infra/images/claude/Dockerfile" \
        "$PROJECT_ROOT/infra/images/claude"
    success "Claude image built: $CLAUDE_IMAGE"
}

# Build linear-sidecar from source
build_sidecar() {
    log "Building linear-sidecar image..."
    
    # Build the Rust binary first (for the platform we're on)
    log "Compiling linear-sidecar binary..."
    cd "$PROJECT_ROOT"
    cargo build --release -p linear-sink --bin linear-sidecar
    
    # Create a temp Dockerfile for the sidecar
    local temp_dockerfile="/tmp/linear-sidecar.Dockerfile"
    cat > "$temp_dockerfile" << 'EOF'
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    --no-install-recommends \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -r -u 1000 -m -d /app -s /bin/bash app

WORKDIR /app

COPY linear-sidecar /app/linear-sidecar
RUN chmod +x /app/linear-sidecar

USER app

ENV RUST_LOG=info

CMD ["./linear-sidecar"]
EOF

    # Copy binary to temp location and build
    cp "$PROJECT_ROOT/target/release/linear-sidecar" /tmp/linear-sidecar
    
    docker build \
        -t "$SIDECAR_IMAGE" \
        -f "$temp_dockerfile" \
        /tmp
    
    rm -f "$temp_dockerfile" /tmp/linear-sidecar
    success "Sidecar image built: $SIDECAR_IMAGE"
}

# Build all images
build_all() {
    log "=== Building All Images ==="
    check_docker
    build_runtime
    build_claude
    build_sidecar
    success "All images built successfully"
    
    echo ""
    log "Images ready:"
    docker images | grep -E "cto-runtime|cto-claude|cto-linear-sidecar" | head -5
}

# Create test workspace and config
setup_workspace() {
    local cli="$1"
    local prompt="$2"
    local workspace="$WORKSPACE_DIR/$cli"
    
    rm -rf "$workspace"
    mkdir -p "$workspace/task" "$workspace/mcp-config" "$workspace/output"
    
    # MCP config
    cat > "$workspace/mcp-config/mcp.json" << EOF
{
  "mcpServers": {
    "cto-tools": {
      "url": "${TOOLS_SERVER_URL}/mcp",
      "transport": { "type": "sse" }
    }
  }
}
EOF
    
    # Prompt
    echo "$prompt" > "$workspace/task/prompt.md"
    
    echo "$workspace"
}

# Run Claude + Sidecar test
run_test() {
    local prompt="${1:-Say hello and report your model name}"
    
    log "=== Running Integration Test ==="
    
    # Check tools server
    if curl -sf "$TOOLS_SERVER_URL/health" >/dev/null 2>&1; then
        success "Tools server reachable"
    else
        warn "Tools server not reachable at $TOOLS_SERVER_URL"
        warn "Ensure Twingate is connected"
    fi
    
    # Setup workspace
    local workspace
    workspace=$(setup_workspace "claude" "$prompt")
    log "Workspace: $workspace"
    
    # Create shared network
    docker network create cto-test-net 2>/dev/null || true
    
    echo ""
    log "═══════════════════════════════════════════════════════════════"
    log "║               STARTING CONTAINERS                            ║"
    log "═══════════════════════════════════════════════════════════════"
    echo ""
    
    # Start sidecar in background (DRY_RUN mode for now - no Linear session)
    log "Starting sidecar (dry-run mode)..."
    docker run -d --rm \
        --name cto-sidecar \
        --network host \
        -v "$workspace:/workspace" \
        -e "DRY_RUN=1" \
        -e "CLI_TYPE=claude" \
        -e "STREAM_FILE=/workspace/output/claude-stream.jsonl" \
        -e "RUST_LOG=info" \
        "$SIDECAR_IMAGE" || {
            warn "Sidecar may already be running or failed to start"
        }
    
    # Give sidecar a moment to start
    sleep 2
    
    # Run Claude CLI
    log "Running Claude CLI..."
    docker run --rm \
        --name cto-claude-test \
        --network host \
        -v "$workspace:/workspace" \
        -v "$workspace/mcp-config:/mcp-config:ro" \
        -w /workspace \
        "$CLAUDE_IMAGE" \
        bash -c 'claude --print --output-format stream-json --verbose \
            --mcp-config /mcp-config/mcp.json --strict-mcp-config \
            --dangerously-skip-permissions \
            "$(cat /workspace/task/prompt.md)"' 2>&1 | tee "$workspace/output/claude-stream.jsonl"
    
    local exit_code=${PIPESTATUS[0]}
    
    echo ""
    log "═══════════════════════════════════════════════════════════════"
    log "║               EXECUTION COMPLETE                             ║"
    log "═══════════════════════════════════════════════════════════════"
    echo ""
    
    # Stop sidecar
    docker stop cto-sidecar 2>/dev/null || true
    
    # Show results
    if [[ $exit_code -eq 0 ]]; then
        success "Claude execution completed"
    else
        error "Claude execution failed (exit code: $exit_code)"
    fi
    
    log "Stream output: $workspace/output/claude-stream.jsonl"
    log "Lines: $(wc -l < "$workspace/output/claude-stream.jsonl" 2>/dev/null | tr -d ' ' || echo 0)"
    
    echo ""
    log "Sample output:"
    head -20 "$workspace/output/claude-stream.jsonl" 2>/dev/null || echo "(empty)"
    
    # Show sidecar logs
    echo ""
    log "Sidecar logs:"
    docker logs cto-sidecar 2>&1 | tail -20 || echo "(no logs)"
    
    # Cleanup
    docker network rm cto-test-net 2>/dev/null || true
    
    return $exit_code
}

# Main
main() {
    echo ""
    echo "=============================================="
    echo "  Local Docker Build & Test"
    echo "=============================================="
    echo ""
    
    cd "$PROJECT_ROOT"
    
    case "${1:-all}" in
        build)
            build_all
            ;;
        test)
            check_docker
            run_test "${2:-Say hello and report your model}"
            ;;
        all|"")
            build_all
            echo ""
            run_test "${2:-Say hello and report your model}"
            ;;
        *)
            echo "Usage: $0 [build|test|all] [prompt]"
            exit 1
            ;;
    esac
}

main "$@"
