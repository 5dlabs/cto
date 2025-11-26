#!/usr/bin/env bash
# vault-init.sh
#
# Initializes HashiCorp Vault after deployment.
# This script handles initialization, unsealing, and configuring Kubernetes auth.
#
# Prerequisites:
#   - Vault is deployed via ArgoCD
#   - Vault CLI is installed locally
#   - kubectl configured with cluster access
#
# Usage:
#   ./vault-init.sh
#
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

VAULT_NAMESPACE="vault"
VAULT_POD="vault-0"
KEY_SHARES=5
KEY_THRESHOLD=3

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."

    if ! command -v vault &> /dev/null; then
        log_error "vault CLI not found. Install from https://developer.hashicorp.com/vault/downloads"
        exit 1
    fi

    if ! command -v kubectl &> /dev/null; then
        log_error "kubectl not found"
        exit 1
    fi

    if ! command -v jq &> /dev/null; then
        log_error "jq not found. Install with: brew install jq"
        exit 1
    fi

    # Check Vault pod is running
    if ! kubectl get pod "$VAULT_POD" -n "$VAULT_NAMESPACE" &> /dev/null; then
        log_error "Vault pod $VAULT_POD not found in namespace $VAULT_NAMESPACE"
        log_error "Ensure Vault is deployed via ArgoCD first"
        exit 1
    fi

    local pod_status
    pod_status=$(kubectl get pod "$VAULT_POD" -n "$VAULT_NAMESPACE" -o jsonpath='{.status.phase}')
    if [[ "$pod_status" != "Running" ]]; then
        log_error "Vault pod is not running (status: $pod_status)"
        exit 1
    fi

    log_info "All prerequisites met"
}

# Start port forward
start_port_forward() {
    log_info "Starting port forward to Vault..."

    # Kill any existing port forwards
    pkill -f "kubectl port-forward.*vault" || true
    sleep 1

    kubectl port-forward svc/vault -n "$VAULT_NAMESPACE" 8200:8200 &
    PF_PID=$!
    sleep 3

    export VAULT_ADDR="http://127.0.0.1:8200"
    log_info "Port forward started (PID: $PF_PID)"
}

# Initialize Vault
initialize_vault() {
    log_step "Initializing Vault..."

    # Check if already initialized
    if vault status 2>&1 | grep -q "Initialized.*true"; then
        log_warn "Vault is already initialized"
        return 0
    fi

    log_info "Initializing with $KEY_SHARES key shares and $KEY_THRESHOLD threshold"

    # Initialize and capture output
    local init_output
    init_output=$(vault operator init -key-shares="$KEY_SHARES" -key-threshold="$KEY_THRESHOLD" -format=json)

    # Save keys to file (SECURE THIS FILE!)
    local keys_file
    keys_file="vault-keys-$(date +%Y%m%d-%H%M%S).json"
    echo "$init_output" > "$keys_file"
    chmod 600 "$keys_file"

    log_info ""
    log_info "=================================================="
    log_info "CRITICAL: Vault initialization keys saved to:"
    log_info "  $keys_file"
    log_info ""
    log_info "Store these keys securely (password manager, etc.)"
    log_info "DELETE this file after securing the keys!"
    log_info "=================================================="
    log_info ""

    # Extract root token for later use
    ROOT_TOKEN=$(echo "$init_output" | jq -r '.root_token')
    export ROOT_TOKEN

    # Extract unseal keys
    UNSEAL_KEYS=$(echo "$init_output" | jq -r '.unseal_keys_b64[]')
}

# Unseal Vault
unseal_vault() {
    log_step "Unsealing Vault..."

    # Check if already unsealed
    if vault status 2>&1 | grep -q "Sealed.*false"; then
        log_warn "Vault is already unsealed"
        return 0
    fi

    if [[ -z "${UNSEAL_KEYS:-}" ]]; then
        log_error "No unseal keys available. Run initialization first or provide keys manually."
        exit 1
    fi

    local count=0
    for key in $UNSEAL_KEYS; do
        if [[ $count -ge $KEY_THRESHOLD ]]; then
            break
        fi
        log_info "Applying unseal key $((count + 1))/$KEY_THRESHOLD..."
        vault operator unseal "$key"
        count=$((count + 1))
    done

    log_info "Vault unsealed successfully"
}

# Enable KV v2 secrets engine
enable_kv_engine() {
    log_step "Enabling KV v2 secrets engine..."

    export VAULT_TOKEN="$ROOT_TOKEN"

    # Check if already enabled
    if vault secrets list 2>/dev/null | grep -q "^secret/"; then
        log_warn "KV secrets engine already enabled at 'secret/'"
        return 0
    fi

    vault secrets enable -path=secret kv-v2
    log_info "KV v2 secrets engine enabled at 'secret/'"
}

# Configure Kubernetes authentication
configure_k8s_auth() {
    log_step "Configuring Kubernetes authentication..."

    export VAULT_TOKEN="$ROOT_TOKEN"

    # Enable Kubernetes auth method
    if vault auth list 2>/dev/null | grep -q "^kubernetes/"; then
        log_warn "Kubernetes auth method already enabled"
    else
        vault auth enable kubernetes
        log_info "Kubernetes auth method enabled"
    fi

    # Configure Kubernetes auth to use in-cluster config
    vault write auth/kubernetes/config \
        kubernetes_host="https://kubernetes.default.svc:443"
    log_info "Kubernetes auth configured with in-cluster endpoint"

    # Create policy for Vault Secrets Operator
    log_info "Creating vault-secrets-operator policy..."
    vault policy write vault-secrets-operator - <<EOF
# Policy for Vault Secrets Operator (VSO)
# Allows reading secrets from KV v2 engine

path "secret/data/*" {
  capabilities = ["read", "list"]
}

path "secret/metadata/*" {
  capabilities = ["read", "list"]
}
EOF
    log_info "Policy 'vault-secrets-operator' created"

    # Create role for Vault Secrets Operator
    # Uses default service account which exists in all namespaces
    log_info "Creating vault-secrets-operator role..."
    vault write auth/kubernetes/role/vault-secrets-operator \
        bound_service_account_names=default \
        'bound_service_account_namespaces=*' \
        policies=vault-secrets-operator \
        ttl=1h
    log_info "Role 'vault-secrets-operator' created"
}

# Cleanup
cleanup() {
    log_info "Cleaning up port forward..."
    pkill -f "kubectl port-forward.*vault" || true
}

# Main function
main() {
    trap cleanup EXIT

    log_info "=========================================="
    log_info "HashiCorp Vault Initialization Script"
    log_info "=========================================="

    check_prerequisites
    start_port_forward

    # Check current status
    log_info "Checking Vault status..."
    vault status || true

    initialize_vault
    unseal_vault
    enable_kv_engine
    configure_k8s_auth

    log_info ""
    log_info "=========================================="
    log_info "Vault initialization complete!"
    log_info "=========================================="
    log_info ""
    log_info "Root token: $ROOT_TOKEN"
    log_info ""
    log_info "Next steps:"
    log_info "  1. Securely store the keys file and delete it"
    log_info "  2. Access Vault UI: kubectl port-forward svc/vault -n vault 8200:8200"
    log_info "  3. Open http://localhost:8200 and login with root token"
    log_info "  4. Add secrets at secret/ path (see infra/vault/README.md for list)"
    log_info "  5. VaultStaticSecrets will automatically sync to K8s"
    log_info ""
}

main "$@"

