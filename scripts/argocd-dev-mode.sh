#!/usr/bin/env bash
# =============================================================================
# argocd-dev-mode.sh - Toggle dev registry mode in ArgoCD
# =============================================================================
# Enables or disables the local dev registry for the CTO application in ArgoCD.
#
# When enabled:
#   - Images pulled from local registry (e.g., 192.168.1.72:30500)
#   - imagePullPolicy set to Always (fresh pulls on restart)
#   - Works with Tilt for continuous development
#
# When disabled:
#   - Images pulled from GHCR (production)
#   - Normal ArgoCD GitOps flow
#
# Usage:
#   ./scripts/argocd-dev-mode.sh enable   # Enable dev registry
#   ./scripts/argocd-dev-mode.sh disable  # Disable dev registry
#   ./scripts/argocd-dev-mode.sh status   # Check current status
#
# =============================================================================
set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
APP_NAME="${ARGOCD_APP:-cto}"
NAMESPACE="${ARGOCD_NAMESPACE:-argocd}"
DEV_TAG="${DEV_TAG:-tilt-dev}"

# Get registry URL from cluster
get_registry_url() {
    local node_ip
    node_ip=$(kubectl get nodes -o jsonpath='{.items[0].status.addresses[?(@.type=="InternalIP")].address}' 2>/dev/null)
    echo "${node_ip}:30500"
}

# Check current status
check_status() {
    echo -e "${BLUE}Checking ArgoCD app status...${NC}"
    echo ""
    
    # Get current parameters
    local params
    params=$(kubectl get application "$APP_NAME" -n "$NAMESPACE" -o jsonpath='{.spec.source.helm.parameters}' 2>/dev/null || echo "[]")
    
    local enabled
    enabled=$(echo "$params" | jq -r '.[] | select(.name == "global.devRegistry.enabled") | .value' 2>/dev/null || echo "false")
    
    local url
    url=$(echo "$params" | jq -r '.[] | select(.name == "global.devRegistry.url") | .value' 2>/dev/null || echo "not set")
    
    local tag
    tag=$(echo "$params" | jq -r '.[] | select(.name == "global.devRegistry.tag") | .value' 2>/dev/null || echo "not set")
    
    if [[ "$enabled" == "true" ]]; then
        echo -e "  Dev Registry: ${GREEN}ENABLED${NC}"
        echo -e "  Registry URL: ${YELLOW}$url${NC}"
        echo -e "  Image Tag:    ${YELLOW}$tag${NC}"
    else
        echo -e "  Dev Registry: ${RED}DISABLED${NC} (using GHCR)"
    fi
    
    echo ""
    
    # Show sync status
    local sync_status
    sync_status=$(kubectl get application "$APP_NAME" -n "$NAMESPACE" -o jsonpath='{.status.sync.status}' 2>/dev/null || echo "Unknown")
    
    local health_status
    health_status=$(kubectl get application "$APP_NAME" -n "$NAMESPACE" -o jsonpath='{.status.health.status}' 2>/dev/null || echo "Unknown")
    
    echo -e "  Sync Status:   $sync_status"
    echo -e "  Health Status: $health_status"
    echo ""
}

# Enable dev registry
enable_dev() {
    local registry_url
    registry_url=$(get_registry_url)
    
    echo -e "${GREEN}Enabling dev registry for ArgoCD app '$APP_NAME'...${NC}"
    echo ""
    echo -e "  Registry: ${YELLOW}$registry_url${NC}"
    echo -e "  Tag:      ${YELLOW}$DEV_TAG${NC}"
    echo ""
    
    # Check if argocd CLI is available
    if command -v argocd &>/dev/null; then
        echo -e "${BLUE}Using argocd CLI...${NC}"
        argocd app set "$APP_NAME" \
            -p global.devRegistry.enabled=true \
            -p global.devRegistry.url="$registry_url" \
            -p global.devRegistry.tag="$DEV_TAG" \
            -p global.devRegistry.pullPolicy=Always
    else
        echo -e "${BLUE}Using kubectl patch...${NC}"
        # Build the parameters array
        kubectl patch application "$APP_NAME" -n "$NAMESPACE" --type=merge -p "{
            \"spec\": {
                \"source\": {
                    \"helm\": {
                        \"parameters\": [
                            {\"name\": \"global.devRegistry.enabled\", \"value\": \"true\"},
                            {\"name\": \"global.devRegistry.url\", \"value\": \"$registry_url\"},
                            {\"name\": \"global.devRegistry.tag\", \"value\": \"$DEV_TAG\"},
                            {\"name\": \"global.devRegistry.pullPolicy\", \"value\": \"Always\"}
                        ]
                    }
                }
            }
        }"
    fi
    
    echo ""
    echo -e "${GREEN}✓ Dev registry enabled!${NC}"
    echo ""
    echo "ArgoCD will sync automatically. To force sync:"
    echo "  argocd app sync $APP_NAME"
    echo ""
    echo "Now run Tilt to start developing:"
    echo "  tilt up"
    echo ""
}

# Disable dev registry
disable_dev() {
    echo -e "${YELLOW}Disabling dev registry for ArgoCD app '$APP_NAME'...${NC}"
    echo ""
    
    if command -v argocd &>/dev/null; then
        echo -e "${BLUE}Using argocd CLI...${NC}"
        argocd app unset "$APP_NAME" \
            -p global.devRegistry.enabled \
            -p global.devRegistry.url \
            -p global.devRegistry.tag \
            -p global.devRegistry.pullPolicy
    else
        echo -e "${BLUE}Using kubectl patch...${NC}"
        # Remove the parameters by setting to empty
        kubectl patch application "$APP_NAME" -n "$NAMESPACE" --type=merge -p '{
            "spec": {
                "source": {
                    "helm": {
                        "parameters": []
                    }
                }
            }
        }'
    fi
    
    echo ""
    echo -e "${GREEN}✓ Dev registry disabled!${NC}"
    echo ""
    echo "ArgoCD will sync automatically to use GHCR images."
    echo "To force sync:"
    echo "  argocd app sync $APP_NAME --force"
    echo ""
}

# Print help
print_help() {
    echo "Usage: $0 <command>"
    echo ""
    echo "Commands:"
    echo "  enable   Enable dev registry (use local images)"
    echo "  disable  Disable dev registry (use GHCR images)"
    echo "  status   Show current dev registry status"
    echo ""
    echo "Environment variables:"
    echo "  ARGOCD_APP        ArgoCD application name (default: cto)"
    echo "  ARGOCD_NAMESPACE  ArgoCD namespace (default: argocd)"
    echo "  DEV_TAG           Image tag for dev builds (default: tilt-dev)"
    echo ""
    echo "Examples:"
    echo "  $0 enable                    # Enable with defaults"
    echo "  DEV_TAG=my-feature $0 enable # Enable with custom tag"
    echo "  $0 status                    # Check current status"
    echo ""
}

# Main
case "${1:-}" in
    enable)
        enable_dev
        ;;
    disable)
        disable_dev
        ;;
    status)
        check_status
        ;;
    -h|--help)
        print_help
        ;;
    *)
        echo -e "${RED}Error: Unknown command '${1:-}'${NC}"
        echo ""
        print_help
        exit 1
        ;;
esac

