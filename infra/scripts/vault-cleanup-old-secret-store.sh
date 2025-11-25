#!/usr/bin/env bash
# vault-cleanup-old-secret-store.sh
#
# Cleans up the old Kubernetes-backed secret store after Vault migration is verified.
#
# WARNING: Only run this script AFTER:
#   1. All ExternalSecrets are successfully syncing from Vault
#   2. All applications are verified to be working correctly
#   3. You have confirmed no secrets are being read from the old secret-store namespace
#
# Usage:
#   ./vault-cleanup-old-secret-store.sh [--dry-run]
#
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

DRY_RUN="${1:-}"

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if this is a dry run
is_dry_run() {
    [[ "$DRY_RUN" == "--dry-run" ]]
}

run_cmd() {
    if is_dry_run; then
        echo "DRY RUN: $*"
    else
        "$@"
    fi
}

# Pre-flight checks
preflight_checks() {
    log_info "Running pre-flight checks..."

    # Check Vault is running and unsealed
    if ! kubectl exec vault-0 -n vault -- vault status 2>/dev/null | grep -q "Sealed.*false"; then
        log_error "Vault is not unsealed. Please unseal Vault first."
        exit 1
    fi
    log_info "✓ Vault is unsealed"

    # Check vault-secret-store ClusterSecretStore exists and is ready
    if ! kubectl get clustersecretstore vault-secret-store -o jsonpath='{.status.conditions[?(@.type=="Ready")].status}' 2>/dev/null | grep -q "True"; then
        log_error "vault-secret-store ClusterSecretStore is not ready"
        exit 1
    fi
    log_info "✓ vault-secret-store is ready"

    # Check all ExternalSecrets are syncing
    local failing_es
    failing_es=$(kubectl get externalsecret -A -o json | jq -r '.items[] | select(.status.conditions[]? | select(.type=="Ready" and .status!="True")) | "\(.metadata.namespace)/\(.metadata.name)"')
    
    if [[ -n "$failing_es" ]]; then
        log_error "The following ExternalSecrets are not ready:"
        echo "$failing_es"
        exit 1
    fi
    log_info "✓ All ExternalSecrets are ready"

    log_info "Pre-flight checks passed!"
}

# Cleanup old resources
cleanup() {
    log_warn "=================================================="
    log_warn "This will remove the old Kubernetes-backed secret store"
    log_warn "=================================================="
    
    if ! is_dry_run; then
        read -r -p "Are you sure you want to proceed? (yes/no): " confirm
        if [[ "$confirm" != "yes" ]]; then
            log_info "Aborted."
            exit 0
        fi
    fi

    log_info "Removing old ClusterSecretStore..."
    run_cmd kubectl delete clustersecretstore secret-store --ignore-not-found

    log_info "Removing secrets from secret-store namespace..."
    run_cmd kubectl delete secret --all -n secret-store --ignore-not-found

    log_info "Removing RBAC resources..."
    run_cmd kubectl delete clusterrolebinding external-secrets-secret-reader --ignore-not-found
    run_cmd kubectl delete clusterrolebinding external-secrets-secret-manager --ignore-not-found
    run_cmd kubectl delete clusterrole external-secrets-secret-reader --ignore-not-found
    run_cmd kubectl delete clusterrole external-secrets-secret-manager --ignore-not-found
    run_cmd kubectl delete rolebinding external-secrets-reader -n secret-store --ignore-not-found
    run_cmd kubectl delete role secret-reader -n secret-store --ignore-not-found
    run_cmd kubectl delete serviceaccount external-secrets-reader -n secret-store --ignore-not-found

    log_info "Removing secret-store namespace..."
    run_cmd kubectl delete namespace secret-store --ignore-not-found

    log_info ""
    log_info "Cleanup complete!"
    log_info ""
    log_info "Next steps:"
    log_info "  1. Remove these files from the repository:"
    log_info "     - infra/secret-store/cluster-secret-store.yaml"
    log_info "     - infra/secret-store/namespace-and-rbac.yaml"
    log_info "  2. Update infra/gitops/applications/secret-store.yaml to remove old resources"
    log_info "  3. Commit and push the changes"
}

main() {
    log_info "=========================================="
    log_info "Old Secret Store Cleanup Script"
    log_info "=========================================="
    
    if is_dry_run; then
        log_warn "Running in DRY RUN mode - no changes will be made"
    fi

    preflight_checks
    cleanup
}

main "$@"

