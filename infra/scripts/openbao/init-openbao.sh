#!/usr/bin/env bash
# OpenBao Initialization Script
# Initializes a new OpenBao instance and stores credentials in 1Password
#
# Prerequisites:
#   - kubectl configured with cluster access
#   - 1Password CLI (op) installed and authenticated
#   - jq installed
#
# Usage:
#   ./init-openbao.sh [OPTIONS]
#
# Options:
#   -n, --namespace     Kubernetes namespace (default: openbao)
#   -p, --pod           Pod name (default: openbao-0)
#   -v, --vault         1Password vault name (default: auto-detect)
#   -t, --title         1Password item title (default: OpenBao - <namespace>)
#   -k, --key-shares    Number of key shares (default: 5)
#   -T, --threshold     Key threshold for unsealing (default: 3)
#   --dev               Use dev mode settings (1 share, 1 threshold)
#   -h, --help          Show this help message

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
NAMESPACE="openbao"
POD_NAME="openbao-0"
OP_VAULT=""
OP_TITLE=""
KEY_SHARES=5
KEY_THRESHOLD=3

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -n|--namespace)
            NAMESPACE="$2"
            shift 2
            ;;
        -p|--pod)
            POD_NAME="$2"
            shift 2
            ;;
        -v|--vault)
            OP_VAULT="$2"
            shift 2
            ;;
        -t|--title)
            OP_TITLE="$2"
            shift 2
            ;;
        -k|--key-shares)
            KEY_SHARES="$2"
            shift 2
            ;;
        -T|--threshold)
            KEY_THRESHOLD="$2"
            shift 2
            ;;
        --dev)
            KEY_SHARES=1
            KEY_THRESHOLD=1
            shift
            ;;
        -h|--help)
            head -30 "$0" | tail -n +2 | sed 's/^# //' | sed 's/^#//'
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Set defaults based on namespace if not provided
OP_TITLE="${OP_TITLE:-OpenBao - ${NAMESPACE}}"

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
    
    # Auto-detect 1Password vault if not specified
    if [[ -z "$OP_VAULT" ]]; then
        OP_VAULT=$(op vault list --format=json | jq -r '.[0].name')
        log "Auto-detected 1Password vault: $OP_VAULT"
    fi
    
    success "Prerequisites check passed"
}

# Check if OpenBao pod is running
check_pod() {
    log "Checking OpenBao pod status..."
    
    if ! kubectl get pod "$POD_NAME" -n "$NAMESPACE" &> /dev/null; then
        error "Pod $POD_NAME not found in namespace $NAMESPACE"
    fi
    
    POD_STATUS=$(kubectl get pod "$POD_NAME" -n "$NAMESPACE" -o jsonpath='{.status.phase}')
    if [[ "$POD_STATUS" != "Running" ]]; then
        error "Pod $POD_NAME is not running (status: $POD_STATUS)"
    fi
    
    success "Pod $POD_NAME is running"
}

# Check current OpenBao status
check_openbao_status() {
    log "Checking OpenBao status..."
    
    # bao status exits with code 2 when sealed, but still returns valid JSON
    STATUS=$(kubectl exec -n "$NAMESPACE" "$POD_NAME" -- bao status -format=json 2>/dev/null) || true
    
    # If we got no output, OpenBao isn't responding
    if [[ -z "$STATUS" ]]; then
        STATUS='{"initialized": false, "sealed": true}'
    fi
    
    echo "$STATUS"
}

# Initialize OpenBao
initialize_openbao() {
    log "Initializing OpenBao with $KEY_SHARES key shares and threshold of $KEY_THRESHOLD..."
    
    INIT_OUTPUT=$(kubectl exec -n "$NAMESPACE" "$POD_NAME" -- bao operator init \
        -key-shares="$KEY_SHARES" \
        -key-threshold="$KEY_THRESHOLD" \
        -format=json)
    
    echo "$INIT_OUTPUT"
}

# Store credentials in 1Password
store_in_1password() {
    local init_output="$1"
    
    log "Storing credentials in 1Password vault: $OP_VAULT"
    
    # Extract values
    ROOT_TOKEN=$(echo "$init_output" | jq -r '.root_token')
    KEY_COUNT=$(echo "$init_output" | jq -r '.unseal_keys_b64 | length')
    
    # Build the 1Password item
    # Create fields for each unseal key
    local key_fields=""
    for i in $(seq 1 "$KEY_COUNT"); do
        KEY=$(echo "$init_output" | jq -r ".unseal_keys_b64[$((i-1))]")
        key_fields="$key_fields 'Unseal Key ${i}[password]=$KEY'"
    done
    
    # Check if item already exists
    if op item get "$OP_TITLE" --vault "$OP_VAULT" &> /dev/null; then
        warn "Item '$OP_TITLE' already exists in 1Password. Creating with timestamp..."
        OP_TITLE="${OP_TITLE} ($(date +%Y-%m-%d-%H%M%S))"
    fi
    
    # Create the item
    # shellcheck disable=SC2086
    eval op item create \
        --category=login \
        --title="\"$OP_TITLE\"" \
        --vault="\"$OP_VAULT\"" \
        "'username=root'" \
        "'password=$ROOT_TOKEN'" \
        $key_fields \
        "'Namespace[text]=$NAMESPACE'" \
        "'Pod[text]=$POD_NAME'" \
        "'Key Shares[text]=$KEY_SHARES'" \
        "'Key Threshold[text]=$KEY_THRESHOLD'" \
        "'notes=OpenBao credentials initialized on $(date -u +%Y-%m-%dT%H:%M:%SZ). Namespace: $NAMESPACE. Requires $KEY_THRESHOLD of $KEY_SHARES keys to unseal.'"
    
    success "Credentials stored in 1Password as '$OP_TITLE'"
}

# Unseal OpenBao
unseal_openbao() {
    local init_output="$1"
    
    log "Unsealing OpenBao..."
    
    # Unseal with threshold number of keys
    for i in $(seq 0 $((KEY_THRESHOLD - 1))); do
        KEY=$(echo "$init_output" | jq -r ".unseal_keys_b64[$i]")
        log "Applying unseal key $((i + 1)) of $KEY_THRESHOLD..."
        kubectl exec -n "$NAMESPACE" "$POD_NAME" -- bao operator unseal "$KEY" > /dev/null
    done
    
    success "OpenBao unsealed successfully"
}

# Verify OpenBao is working
verify_openbao() {
    log "Verifying OpenBao status..."
    
    STATUS=$(kubectl exec -n "$NAMESPACE" "$POD_NAME" -- bao status -format=json)
    
    INITIALIZED=$(echo "$STATUS" | jq -r '.initialized')
    SEALED=$(echo "$STATUS" | jq -r '.sealed')
    VERSION=$(echo "$STATUS" | jq -r '.version')
    
    if [[ "$INITIALIZED" != "true" ]]; then
        error "OpenBao failed to initialize"
    fi
    
    if [[ "$SEALED" != "false" ]]; then
        error "OpenBao is still sealed"
    fi
    
    success "OpenBao v$VERSION is initialized and unsealed"
    
    # Show pod labels
    log "Pod labels:"
    kubectl get pod "$POD_NAME" -n "$NAMESPACE" -o jsonpath='{.metadata.labels}' | jq -r 'to_entries | .[] | select(.key | startswith("openbao-")) | "  \(.key): \(.value)"'
}

# Main function
main() {
    echo ""
    echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║     OpenBao Initialization Script          ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
    echo ""
    
    check_prerequisites
    check_pod
    
    # Check current status
    STATUS_JSON=$(check_openbao_status)
    INITIALIZED=$(echo "$STATUS_JSON" | jq -r '.initialized')
    SEALED=$(echo "$STATUS_JSON" | jq -r '.sealed // true')
    
    if [[ "$INITIALIZED" == "true" ]]; then
        warn "OpenBao is already initialized"
        
        if [[ "$SEALED" == "true" ]]; then
            echo ""
            echo -e "${YELLOW}OpenBao is sealed. To unseal, retrieve keys from 1Password:${NC}"
            echo "  op item get \"$OP_TITLE\" --vault \"$OP_VAULT\" --reveal"
            echo ""
            echo "Then unseal with:"
            echo "  kubectl exec -n $NAMESPACE $POD_NAME -- bao operator unseal <KEY>"
            echo ""
        else
            success "OpenBao is already initialized and unsealed"
            verify_openbao
        fi
        exit 0
    fi
    
    # Initialize
    INIT_OUTPUT=$(initialize_openbao)
    
    # Store in 1Password
    store_in_1password "$INIT_OUTPUT"
    
    # Unseal
    unseal_openbao "$INIT_OUTPUT"
    
    # Verify
    verify_openbao
    
    echo ""
    echo -e "${GREEN}╔════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║     Initialization Complete!               ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════╝${NC}"
    echo ""
    echo "Credentials stored in 1Password:"
    echo "  Vault: $OP_VAULT"
    echo "  Item:  $OP_TITLE"
    echo ""
    echo "To retrieve credentials later:"
    echo "  op item get \"$OP_TITLE\" --vault \"$OP_VAULT\" --reveal"
    echo ""
}

main

