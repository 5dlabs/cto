#!/usr/bin/env bash
# =============================================================================
# Run CLI Integration Test with FluentD Logging
# =============================================================================
#
# Prerequisites:
#   1. Docker with FluentD logging driver plugin
#   2. .env file with LINEAR_OAUTH_TOKEN, LINEAR_SESSION_ID, etc.
#
# Usage:
#   ./run-test.sh              # Run Claude with FluentD → Loki → Linear
#   ./run-test.sh --no-loki    # Run Claude with file-based logging (original)
#
# =============================================================================

set -euo pipefail

cd "$(dirname "$0")"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo_step() {
    echo -e "${GREEN}==>${NC} $1"
}

echo_warn() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

echo_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check prerequisites
check_prereqs() {
    echo_step "Checking prerequisites..."
    
    # Check .env file
    if [[ ! -f .env ]]; then
        echo_error ".env file not found. Copy .env.example and fill in values."
        exit 1
    fi
    
    # Source .env
    set -a
    source .env
    set +a
    
    # Check required vars
    if [[ -z "${LINEAR_OAUTH_TOKEN:-}" ]]; then
        echo_error "LINEAR_OAUTH_TOKEN not set in .env"
        exit 1
    fi
    
    if [[ -z "${ANTHROPIC_API_KEY:-}" ]]; then
        echo_warn "ANTHROPIC_API_KEY not set - Claude will fail"
    fi
    
    echo "  ✓ Environment configured"
}

# Build images
build_images() {
    echo_step "Building Docker images..."
    ./build-images.sh sidecar
    
    # Check if Claude image exists, build if not
    if ! docker images | grep -q "cto-claude.*local"; then
        echo_warn "cto-claude:local not found, building..."
        ./build-images.sh claude
    else
        echo "  ✓ cto-claude:local exists"
    fi
}

# Setup skills and tools
setup_config() {
    echo_step "Setting up skills and tools..."
    ./setup.sh rex coder
}

# Run with FluentD + Loki
run_with_fluentd() {
    echo_step "Starting FluentD + Loki stack..."
    
    # Start infrastructure first
    docker compose -f docker-compose.yml -f docker-compose.loki.yml up -d fluentd loki grafana sidecar
    
    # Wait for FluentD to be ready
    echo "  Waiting for FluentD..."
    sleep 5
    
    # Start Claude
    echo_step "Starting Claude CLI..."
    docker compose -f docker-compose.yml -f docker-compose.loki.yml up claude
}

# Run with original file-based logging
run_with_files() {
    echo_step "Starting with file-based logging..."
    docker compose up claude claude-sidecar
}

# Main
main() {
    check_prereqs
    build_images
    setup_config
    
    if [[ "${1:-}" == "--no-loki" ]]; then
        run_with_files
    else
        run_with_fluentd
    fi
}

# Cleanup on exit
cleanup() {
    echo ""
    echo_step "Cleaning up..."
    docker compose -f docker-compose.yml -f docker-compose.loki.yml down 2>/dev/null || true
}
trap cleanup EXIT

main "$@"
