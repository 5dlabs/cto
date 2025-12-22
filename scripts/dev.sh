#!/usr/bin/env bash
# =============================================================================
# dev.sh - CTO Platform Development Workflow Helper
# =============================================================================
# Simplifies the dev workflow by combining ArgoCD dev registry toggle with Tilt.
#
# Commands:
#   up      Enable dev registry and start Tilt
#   down    Stop Tilt and disable dev registry
#   status  Show current dev mode status
#   build   Trigger a manual build in Tilt
#   logs    Show Tilt logs
#
# Environment Variables:
#   DEV_TAG         Image tag for dev builds (default: tilt-dev)
#   CTO_DEV_MODE    Force dev mode: true|false|auto (default: auto)
#   LOCAL_REGISTRY  Override local registry URL
#
# Examples:
#   ./scripts/dev.sh up              # Start development
#   ./scripts/dev.sh down            # Stop development
#   ./scripts/dev.sh status          # Check status
#   DEV_TAG=my-feature ./scripts/sh up   # Use custom tag
#
# =============================================================================
set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Configuration
ARGOCD_SCRIPT="$SCRIPT_DIR/argocd-dev-mode.sh"

# Get current branch
get_branch() {
    git -C "$PROJECT_ROOT" rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown"
}

# Check if on protected branch
is_protected_branch() {
    local branch
    branch=$(get_branch)
    [[ "$branch" == "main" || "$branch" == "develop" ]]
}

# Print banner
print_banner() {
    local mode=$1
    local branch
    branch=$(get_branch)
    
    echo ""
    if [[ "$mode" == "dev" ]]; then
        echo -e "${GREEN}╔═══════════════════════════════════════════════════════════════════╗${NC}"
        echo -e "${GREEN}║  🔧 CTO Development Mode                                          ║${NC}"
        echo -e "${GREEN}╠═══════════════════════════════════════════════════════════════════╣${NC}"
        printf "${GREEN}║${NC}  Branch:   ${CYAN}%-54s${NC}${GREEN} ║${NC}\n" "$branch"
        printf "${GREEN}║${NC}  Registry: ${YELLOW}%-54s${NC}${GREEN} ║${NC}\n" "Local (192.168.1.77:30500)"
        printf "${GREEN}║${NC}  Tag:      ${YELLOW}%-54s${NC}${GREEN} ║${NC}\n" "${DEV_TAG:-tilt-dev}"
        echo -e "${GREEN}╚═══════════════════════════════════════════════════════════════════╝${NC}"
    else
        echo -e "${BLUE}╔═══════════════════════════════════════════════════════════════════╗${NC}"
        echo -e "${BLUE}║  🚀 CTO Production Mode                                           ║${NC}"
        echo -e "${BLUE}╠═══════════════════════════════════════════════════════════════════╣${NC}"
        printf "${BLUE}║${NC}  Branch:   ${CYAN}%-54s${NC}${BLUE} ║${NC}\n" "$branch"
        printf "${BLUE}║${NC}  Registry: ${YELLOW}%-54s${NC}${BLUE} ║${NC}\n" "ghcr.io/5dlabs/*"
        echo -e "${BLUE}║${NC}                                                                   ${BLUE}║${NC}"
        echo -e "${BLUE}║${NC}  Builds handled by CI/CD. Local builds disabled.                 ${BLUE}║${NC}"
        echo -e "${BLUE}╚═══════════════════════════════════════════════════════════════════╝${NC}"
    fi
    echo ""
}

# Start development mode
cmd_up() {
    echo -e "${BLUE}Starting CTO development environment...${NC}"
    echo ""
    
    # Check if on protected branch
    if is_protected_branch; then
        local branch
        branch=$(get_branch)
        echo -e "${YELLOW}⚠️  Warning: You're on '$branch' branch${NC}"
        echo -e "${YELLOW}   Builds are normally handled by CI/CD on this branch.${NC}"
        echo ""
        read -p "Continue anyway? (y/N) " -n 1 -r
        echo ""
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo -e "${RED}Aborted.${NC}"
            exit 1
        fi
        # Force dev mode if continuing
        export CTO_DEV_MODE=true
    fi
    
    # Enable dev registry in ArgoCD
    echo -e "${BLUE}1. Enabling dev registry in ArgoCD...${NC}"
    if [[ -x "$ARGOCD_SCRIPT" ]]; then
        "$ARGOCD_SCRIPT" enable
    else
        echo -e "${YELLOW}   Warning: argocd-dev-mode.sh not found or not executable${NC}"
        echo -e "${YELLOW}   Continuing anyway - you may need to enable dev registry manually${NC}"
    fi
    
    # Start Tilt
    echo ""
    echo -e "${BLUE}2. Starting Tilt...${NC}"
    echo -e "${CYAN}   Access Tilt UI at: http://localhost:10350${NC}"
    echo ""
    
    cd "$PROJECT_ROOT"
    
    # Print banner
    print_banner "dev"
    
    # Start Tilt (foreground so user can see output)
    exec tilt up
}

# Stop development mode
cmd_down() {
    echo -e "${BLUE}Stopping CTO development environment...${NC}"
    echo ""
    
    # Stop Tilt if running
    echo -e "${BLUE}1. Stopping Tilt...${NC}"
    if pgrep -f "tilt up" > /dev/null 2>&1; then
        tilt down 2>/dev/null || true
        echo -e "${GREEN}   ✓ Tilt stopped${NC}"
    else
        echo -e "${YELLOW}   Tilt not running${NC}"
    fi
    
    # Disable dev registry in ArgoCD
    echo ""
    echo -e "${BLUE}2. Disabling dev registry in ArgoCD...${NC}"
    if [[ -x "$ARGOCD_SCRIPT" ]]; then
        "$ARGOCD_SCRIPT" disable
    else
        echo -e "${YELLOW}   Warning: argocd-dev-mode.sh not found${NC}"
    fi
    
    echo ""
    echo -e "${GREEN}✓ Development environment stopped${NC}"
    echo -e "${CYAN}  ArgoCD will sync to use GHCR images automatically.${NC}"
    echo ""
}

# Show status
cmd_status() {
    local branch
    branch=$(get_branch)
    
    echo -e "${BLUE}CTO Development Environment Status${NC}"
    echo -e "${BLUE}══════════════════════════════════${NC}"
    echo ""
    
    # Git info
    echo -e "${CYAN}Git:${NC}"
    echo -e "  Branch: $branch"
    if is_protected_branch; then
        echo -e "  Mode:   ${YELLOW}Protected (CI/CD builds)${NC}"
    else
        echo -e "  Mode:   ${GREEN}Feature branch (local builds)${NC}"
    fi
    echo ""
    
    # ArgoCD status
    echo -e "${CYAN}ArgoCD:${NC}"
    if [[ -x "$ARGOCD_SCRIPT" ]]; then
        "$ARGOCD_SCRIPT" status 2>/dev/null | sed 's/^/  /'
    else
        echo -e "  ${YELLOW}argocd-dev-mode.sh not found${NC}"
    fi
    echo ""
    
    # Tilt status
    echo -e "${CYAN}Tilt:${NC}"
    if pgrep -f "tilt up" > /dev/null 2>&1; then
        echo -e "  Status: ${GREEN}Running${NC}"
        echo -e "  UI:     http://localhost:10350"
    else
        echo -e "  Status: ${YELLOW}Not running${NC}"
    fi
    echo ""
    
    # Cluster deployments
    echo -e "${CYAN}Deployments (cto namespace):${NC}"
    kubectl get deployments -n cto -o custom-columns='NAME:.metadata.name,IMAGE:.spec.template.spec.containers[0].image,READY:.status.readyReplicas' 2>/dev/null | sed 's/^/  /' || echo -e "  ${RED}Could not fetch deployments${NC}"
    echo ""
}

# Trigger manual build
cmd_build() {
    local service="${1:-all}"
    
    if ! pgrep -f "tilt up" > /dev/null 2>&1; then
        echo -e "${RED}Error: Tilt is not running${NC}"
        echo -e "${CYAN}Start with: ./scripts/dev.sh up${NC}"
        exit 1
    fi
    
    echo -e "${BLUE}Triggering build for: $service${NC}"
    
    if [[ "$service" == "all" ]]; then
        tilt trigger build-pm build-controller build-tools build-healer build-research
    else
        tilt trigger "build-$service"
    fi
}

# Show logs
cmd_logs() {
    local service="${1:-}"
    
    if ! pgrep -f "tilt up" > /dev/null 2>&1; then
        echo -e "${RED}Error: Tilt is not running${NC}"
        exit 1
    fi
    
    if [[ -n "$service" ]]; then
        tilt logs "$service"
    else
        tilt logs
    fi
}

# Print help
print_help() {
    echo "Usage: $0 <command> [args]"
    echo ""
    echo "Commands:"
    echo "  up              Enable dev registry and start Tilt"
    echo "  down            Stop Tilt and disable dev registry"
    echo "  status          Show current development status"
    echo "  build [svc]     Trigger build (all services if no arg)"
    echo "  logs [svc]      Show Tilt logs"
    echo ""
    echo "Environment Variables:"
    echo "  DEV_TAG         Image tag for dev builds (default: tilt-dev)"
    echo "  CTO_DEV_MODE    Force mode: true|false|auto (default: auto)"
    echo "  LOCAL_REGISTRY  Override local registry URL"
    echo ""
    echo "Examples:"
    echo "  $0 up                      # Start development"
    echo "  $0 down                    # Stop development"
    echo "  $0 status                  # Check status"
    echo "  $0 build controller        # Rebuild controller only"
    echo "  DEV_TAG=feat-1 $0 up       # Use custom tag"
    echo ""
}

# Main
case "${1:-}" in
    up)
        cmd_up
        ;;
    down)
        cmd_down
        ;;
    status)
        cmd_status
        ;;
    build)
        cmd_build "${2:-all}"
        ;;
    logs)
        cmd_logs "${2:-}"
        ;;
    -h|--help|help)
        print_help
        ;;
    *)
        if [[ -n "${1:-}" ]]; then
            echo -e "${RED}Error: Unknown command '${1}'${NC}"
            echo ""
        fi
        print_help
        exit 1
        ;;
esac

