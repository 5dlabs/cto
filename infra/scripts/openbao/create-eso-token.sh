#!/usr/bin/env bash
# Create OpenBao Token Secret for External Secrets Operator
#
# This script creates the openbao-token Kubernetes secret that the External
# Secrets Operator uses to authenticate with OpenBao and sync secrets.
#
# Prerequisites:
#   - kubectl configured with cluster access
#   - 1Password CLI (op) installed and authenticated
#   - jq installed
#   - OpenBao must be unsealed
#
# Usage:
#   ./create-eso-token.sh

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

NAMESPACE="openbao"
SECRET_NAME="openbao-token"
OP_ITEM="OpenBao Unseal Keys - CTO Platform"

log() {
    echo -e "${BLUE}[INFO]${NC} $1" >&2
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1" >&2
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1" >&2
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
    exit 1
}

# Check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."
    
    if ! command -v kubectl &> /dev/null; then
        error "kubectl is not installed"
    fi
    
    if ! command -v op &> /dev/null; then
        error "1Password CLI (op) is not installed. Install: brew install 1password-cli (macOS) or https://1password.com/downloads/command-line/"
    fi
    
    if ! command -v jq &> /dev/null; then
        error "jq is not installed"
    fi
    
    # Check 1Password authentication
    if ! op account get &> /dev/null; then
        error "Not authenticated to 1Password. Run: eval \$(op signin)"
    fi
    
    success "Prerequisites check passed"
}

# Check OpenBao status
check_openbao() {
    log "Checking OpenBao status..."
    
    STATUS=$(kubectl exec -n "$NAMESPACE" openbao-0 -- bao status -format=json 2>/dev/null) || true
    
    if [[ -z "$STATUS" ]]; then
        error "Cannot connect to OpenBao. Is the pod running?"
    fi
    
    SEALED=$(echo "$STATUS" | jq -r '.sealed')
    if [[ "$SEALED" == "true" ]]; then
        error "OpenBao is sealed. Please unseal it first using unseal-openbao.sh"
    fi
    
    success "OpenBao is unsealed and ready"
}

# Get root token from 1Password
get_root_token() {
    log "Retrieving root token from 1Password..."
    
    ROOT_TOKEN=$(op item get "$OP_ITEM" --format=json | \
        jq -r '.fields[] | select(.label == "password" or .label == "Root Token") | .value')
    
    if [[ -z "$ROOT_TOKEN" ]]; then
        error "Could not find root token in 1Password item: $OP_ITEM"
    fi
    
    echo "$ROOT_TOKEN"
}

# Create or update the secret
create_secret() {
    local token="$1"
    
    log "Creating/updating openbao-token secret..."
    
    kubectl create secret generic "$SECRET_NAME" \
        --from-literal=token="$token" \
        -n "$NAMESPACE" \
        --dry-run=client -o yaml | kubectl apply -f -
    
    success "Secret $SECRET_NAME created/updated in namespace $NAMESPACE"
}

# Verify the secret works
verify_secret() {
    log "Verifying secret connectivity..."
    
    # Check if External Secrets Operator is installed
    if kubectl get crd clustersecretstores.external-secrets.io &> /dev/null; then
        log "External Secrets Operator is installed"
        
        # Check ClusterSecretStore status if it exists
        if kubectl get clustersecretstore openbao &> /dev/null; then
            STATUS=$(kubectl get clustersecretstore openbao -o jsonpath='{.status.conditions[?(@.type=="Ready")].status}' 2>/dev/null || echo "Unknown")
            if [[ "$STATUS" == "True" ]]; then
                success "ClusterSecretStore 'openbao' is ready!"
            else
                warn "ClusterSecretStore 'openbao' status: $STATUS"
                warn "Check: kubectl describe clustersecretstore openbao"
            fi
        else
            warn "ClusterSecretStore 'openbao' not found yet. It will be created by ArgoCD."
        fi
    else
        warn "External Secrets Operator not installed yet. CRDs not found."
    fi
}

# Main function
main() {
    echo ""
    echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║  Create ESO Token Secret for OpenBao       ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
    echo ""
    
    check_prerequisites
    check_openbao
    
    ROOT_TOKEN=$(get_root_token)
    create_secret "$ROOT_TOKEN"
    verify_secret
    
    echo ""
    echo -e "${GREEN}╔════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║     Token Secret Created Successfully!     ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════╝${NC}"
    echo ""
    echo "External Secrets Operator can now sync secrets from OpenBao."
    echo ""
    echo "To check sync status:"
    echo "  kubectl get externalsecrets -A"
    echo ""
}

main

