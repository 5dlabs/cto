#!/usr/bin/env bash
set -euo pipefail
# =============================================================================
# dev-local.sh - Local Development Orchestration Script
# =============================================================================
# Starts all CTO platform services locally with proper environment setup.
# Services run as native binaries, connecting to the real cluster via kubeconfig.
#
# Prerequisites:
#   - Rust toolchain installed
#   - kubectl configured with cluster access
#   - Environment variables set (source .env.local)
#
# Usage:
#   ./scripts/dev-local.sh              # Start all services
#   ./scripts/dev-local.sh --pm         # Start only PM server
#   ./scripts/dev-local.sh --controller # Start only controller
#   ./scripts/dev-local.sh --tools      # Start only tools server
#   ./scripts/dev-local.sh --healer     # Start only healer server
#   ./scripts/dev-local.sh --check      # Just validate environment
#
# =============================================================================

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# PIDs for cleanup
declare -a PIDS=()

# =============================================================================
# Cleanup Handler
# =============================================================================
cleanup() {
    echo ""
    echo -e "${YELLOW}Shutting down services...${NC}"
    for pid in "${PIDS[@]}"; do
        if kill -0 "$pid" 2>/dev/null; then
            echo "  Stopping PID $pid"
            kill "$pid" 2>/dev/null || true
        fi
    done
    wait 2>/dev/null || true
    echo -e "${GREEN}All services stopped.${NC}"
}

trap cleanup EXIT INT TERM

# =============================================================================
# Helper Functions
# =============================================================================

log_header() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

log_success() {
    echo -e "  ${GREEN}✓${NC} $1"
}

log_warning() {
    echo -e "  ${YELLOW}⚠${NC} $1"
}

log_error() {
    echo -e "  ${RED}✗${NC} $1"
}

log_info() {
    echo -e "  ${CYAN}ℹ${NC} $1"
}

# =============================================================================
# Environment Validation
# =============================================================================

validate_environment() {
    log_header "Validating Environment"
    
    local errors=0
    
    # Check Rust toolchain
    if command -v cargo &> /dev/null; then
        log_success "Cargo: $(cargo --version | head -1)"
    else
        log_error "Cargo not found. Install Rust: https://rustup.rs"
        ((errors++))
    fi
    
    # Check kubectl
    if command -v kubectl &> /dev/null; then
        local context
        context=$(kubectl config current-context 2>/dev/null || echo "none")
        log_success "kubectl: context=$context"
    else
        log_warning "kubectl not found. Services may fail to connect to cluster."
    fi
    
    # Check kubeconfig
    if [ -n "${KUBECONFIG:-}" ] && [ -f "$KUBECONFIG" ]; then
        log_success "KUBECONFIG: $KUBECONFIG"
    elif [ -f "$HOME/.kube/config" ]; then
        log_success "KUBECONFIG: ~/.kube/config (default)"
    else
        log_warning "No kubeconfig found. Kubernetes operations will fail."
    fi
    
    # Check required environment variables
    echo ""
    echo "  Checking required environment variables..."
    
    # Critical vars
    local critical_vars=("ANTHROPIC_API_KEY")
    for var in "${critical_vars[@]}"; do
        if [ -n "${!var:-}" ]; then
            log_success "$var is set"
        else
            log_error "$var is NOT set (required)"
            ((errors++))
        fi
    done
    
    # PM server vars
    local pm_vars=("LINEAR_OAUTH_TOKEN" "LINEAR_API_KEY")
    for var in "${pm_vars[@]}"; do
        if [ -n "${!var:-}" ]; then
            log_success "$var is set"
        else
            log_warning "$var is NOT set (needed for PM server)"
        fi
    done
    
    # GitHub vars
    if [ -n "${GITHUB_TOKEN:-}" ] || [ -n "${GITHUB_PERSONAL_ACCESS_TOKEN:-}" ]; then
        log_success "GitHub token is set"
    else
        log_warning "GITHUB_TOKEN is NOT set (needed for GitHub operations)"
    fi
    
    # Namespace
    if [ -n "${NAMESPACE:-}" ]; then
        log_success "NAMESPACE=$NAMESPACE"
    else
        export NAMESPACE="cto"
        log_info "NAMESPACE defaulted to 'cto'"
    fi
    
    if [ "$errors" -gt 0 ]; then
        echo ""
        log_error "Environment validation failed with $errors error(s)"
        echo ""
        echo "To set up your environment:"
        echo "  1. Run: ./scripts/sync-secrets-for-dev.sh"
        echo "  2. Or: cp env.template .env.local && edit .env.local"
        echo "  3. Then: source .env.local"
        return 1
    fi
    
    log_success "Environment validation passed"
    return 0
}

# =============================================================================
# Service Runners
# =============================================================================

start_pm() {
    log_header "Starting PM Server"
    log_info "Port: ${LINEAR_PORT:-8081}"
    log_info "Team: ${LINEAR_TEAM_ID:-CTOPA}"
    
    cd "$PROJECT_ROOT"
    cargo run --bin pm-server &
    PIDS+=($!)
    log_success "PM server started (PID: ${PIDS[-1]})"
}

start_controller() {
    log_header "Starting Controller"
    log_info "Port: ${SERVER_PORT:-8080}"
    log_info "Templates: ${AGENT_TEMPLATES_PATH:-./templates}"
    
    cd "$PROJECT_ROOT"
    export AGENT_TEMPLATES_PATH="${AGENT_TEMPLATES_PATH:-./templates}"
    cargo run --bin agent_controller &
    PIDS+=($!)
    log_success "Controller started (PID: ${PIDS[-1]})"
}

start_tools() {
    log_header "Starting Tools Server"
    log_info "Port: ${PORT:-3000}"
    
    cd "$PROJECT_ROOT"
    export SYSTEM_CONFIG_PATH="${SYSTEM_CONFIG_PATH:-./infra/charts/cto/templates/tools}"
    cargo run --bin tools-server &
    PIDS+=($!)
    log_success "Tools server started (PID: ${PIDS[-1]})"
}

start_healer() {
    log_header "Starting Healer Server"
    log_info "Config: ${CTO_CONFIG_PATH:-./cto-config.json}"
    log_info "Templates: ${HEALER_TEMPLATES_DIR:-./templates/healer}"
    
    cd "$PROJECT_ROOT"
    export HEALER_TEMPLATES_DIR="${HEALER_TEMPLATES_DIR:-./templates/healer}"
    export CTO_CONFIG_PATH="${CTO_CONFIG_PATH:-./cto-config.json}"
    cargo run --bin healer -- server &
    PIDS+=($!)
    log_success "Healer server started (PID: ${PIDS[-1]})"
}

# =============================================================================
# Main
# =============================================================================

main() {
    echo -e "${BLUE}"
    echo "╔═══════════════════════════════════════════════════════════════════════╗"
    echo "║        CTO Platform - Local Development Environment                    ║"
    echo "╚═══════════════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
    
    local start_pm=false
    local start_controller=false
    local start_tools=false
    local start_healer=false
    local check_only=false
    local start_all=true
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --pm)
                start_pm=true
                start_all=false
                shift
                ;;
            --controller)
                start_controller=true
                start_all=false
                shift
                ;;
            --tools)
                start_tools=true
                start_all=false
                shift
                ;;
            --healer)
                start_healer=true
                start_all=false
                shift
                ;;
            --check)
                check_only=true
                shift
                ;;
            --help|-h)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --pm          Start only PM server"
                echo "  --controller  Start only controller"
                echo "  --tools       Start only tools server"
                echo "  --healer      Start only healer server"
                echo "  --check       Only validate environment (don't start services)"
                echo "  --help, -h    Show this help"
                echo ""
                echo "Without options, all services are started."
                exit 0
                ;;
            *)
                echo "Unknown option: $1"
                echo "Run '$0 --help' for usage."
                exit 1
                ;;
        esac
    done
    
    # Validate environment
    if ! validate_environment; then
        exit 1
    fi
    
    if [ "$check_only" = true ]; then
        echo ""
        log_success "Environment check complete. Ready to run services."
        exit 0
    fi
    
    # Start services
    if [ "$start_all" = true ]; then
        start_pm
        sleep 1
        start_controller
        sleep 1
        start_tools
        sleep 1
        start_healer
    else
        [ "$start_pm" = true ] && start_pm
        [ "$start_controller" = true ] && start_controller
        [ "$start_tools" = true ] && start_tools
        [ "$start_healer" = true ] && start_healer
    fi
    
    # Print status
    log_header "Services Running"
    echo ""
    if [ "$start_all" = true ] || [ "$start_pm" = true ]; then
        echo -e "  ${GREEN}●${NC} PM Server        http://localhost:${LINEAR_PORT:-8081}"
    fi
    if [ "$start_all" = true ] || [ "$start_controller" = true ]; then
        echo -e "  ${GREEN}●${NC} Controller       http://localhost:${SERVER_PORT:-8080}"
    fi
    if [ "$start_all" = true ] || [ "$start_tools" = true ]; then
        echo -e "  ${GREEN}●${NC} Tools Server     http://localhost:${PORT:-3000}"
    fi
    if [ "$start_all" = true ] || [ "$start_healer" = true ]; then
        echo -e "  ${GREEN}●${NC} Healer Server    http://localhost:8080 (healer)"
    fi
    echo ""
    echo -e "${YELLOW}Press Ctrl+C to stop all services${NC}"
    echo ""
    
    # Wait for services
    wait
}

main "$@"
