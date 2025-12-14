#!/usr/bin/env bash
# OpenBao Unseal Script
# Retrieves unseal keys from 1Password and unseals OpenBao
#
# Prerequisites:
#   - kubectl configured with cluster access
#   - 1Password CLI (op) installed and authenticated
#   - jq installed
#
# Usage:
#   ./unseal-openbao.sh [OPTIONS]
#
# Options:
#   -n, --namespace     Kubernetes namespace (default: openbao)
#   -p, --pod           Pod name (default: openbao-0)
#   -v, --vault         1Password vault name (default: auto-detect)
#   -t, --title         1Password item title (default: OpenBao - <namespace>)
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
        -h|--help)
            head -20 "$0" | tail -n +2 | sed 's/^# //' | sed 's/^#//'
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Set defaults
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
        error "1Password CLI (op) is not installed"
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
}

# Check OpenBao status
check_status() {
    log "Checking OpenBao status..."
    
    # bao status exits with code 2 when sealed, but still returns valid JSON
    STATUS=$(kubectl exec -n "$NAMESPACE" "$POD_NAME" -- bao status -format=json 2>/dev/null) || true
    
    # If we got no output, OpenBao isn't responding
    if [[ -z "$STATUS" ]]; then
        STATUS='{"initialized": false, "sealed": true, "t": 3}'
    fi
    
    INITIALIZED=$(echo "$STATUS" | jq -r '.initialized')
    SEALED=$(echo "$STATUS" | jq -r '.sealed // true')
    
    if [[ "$INITIALIZED" != "true" ]]; then
        error "OpenBao is not initialized. Run init-openbao.sh first."
    fi
    
    if [[ "$SEALED" != "true" ]]; then
        success "OpenBao is already unsealed"
        exit 0
    fi
    
    # Get threshold from status
    THRESHOLD=$(echo "$STATUS" | jq -r '.t // 3')
    PROGRESS=$(echo "$STATUS" | jq -r '.progress // 0')
    
    log "OpenBao is sealed. Threshold: $THRESHOLD, Progress: $PROGRESS"
    echo "$THRESHOLD"
}

# Retrieve keys from 1Password
get_keys_from_1password() {
    log "Retrieving unseal keys from 1Password..."
    
    ITEM=$(op item get "$OP_TITLE" --vault "$OP_VAULT" --format=json 2>/dev/null) || {
        error "Could not find '$OP_TITLE' in vault '$OP_VAULT'"
    }
    
    # Extract unseal keys (they're stored as "Unseal Key 1", "Unseal Key 2", etc.)
    KEYS=$(echo "$ITEM" | jq -r '.fields[] | select(.label | startswith("Unseal Key")) | .value')
    
    if [[ -z "$KEYS" ]]; then
        error "No unseal keys found in 1Password item"
    fi
    
    echo "$KEYS"
}

# Unseal OpenBao
unseal() {
    local threshold="$1"
    local keys="$2"
    
    log "Unsealing OpenBao..."
    
    local count=0
    while IFS= read -r key; do
        if [[ -n "$key" ]]; then
            count=$((count + 1))
            log "Applying unseal key $count..."
            
            # Note: -format=json must come BEFORE the key
            # Capture both stdout and stderr, allow non-zero exit
            if RESULT=$(kubectl exec -n "$NAMESPACE" "$POD_NAME" -- bao operator unseal -format=json "$key" 2>&1); then
                SEALED=$(echo "$RESULT" | jq -r '.sealed // true')
            else
                # Even on error, try to parse the output
                SEALED=$(echo "$RESULT" | jq -r '.sealed // true' 2>/dev/null || echo "true")
            fi
            
            if [[ "$SEALED" == "false" ]]; then
                success "OpenBao unsealed after $count keys"
                return 0
            fi
            
            if [[ $count -ge $threshold ]]; then
                break
            fi
        fi
    done <<< "$keys"
    
    # Final check
    FINAL_STATUS=$(kubectl exec -n "$NAMESPACE" "$POD_NAME" -- bao status -format=json 2>/dev/null) || true
    if [[ -n "$FINAL_STATUS" ]] && [[ $(echo "$FINAL_STATUS" | jq -r '.sealed') == "false" ]]; then
        success "OpenBao unsealed successfully"
    else
        error "Failed to unseal OpenBao after $count keys"
    fi
}

# Main function
main() {
    echo ""
    echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║        OpenBao Unseal Script               ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
    echo ""
    
    check_prerequisites
    
    # Check status and get threshold
    THRESHOLD=$(check_status)
    
    # Get keys from 1Password
    KEYS=$(get_keys_from_1password)
    
    # Unseal
    unseal "$THRESHOLD" "$KEYS"
    
    # Show final status
    echo ""
    log "Final status:"
    kubectl exec -n "$NAMESPACE" "$POD_NAME" -- bao status
    echo ""
}

main

