#!/usr/bin/env bash
# migrate-secrets-to-vault.sh
#
# Migrates secrets from the Kubernetes secret-store namespace to HashiCorp Vault.
# This script exports existing secrets and imports them into Vault's KV v2 engine.
#
# Prerequisites:
#   - Vault is deployed, initialized, and unsealed
#   - Vault CLI is installed locally
#   - kubectl configured with cluster access
#   - jq installed for JSON processing
#
# Usage:
#   export VAULT_ADDR=http://127.0.0.1:8200
#   export VAULT_TOKEN=<root_token>
#   kubectl port-forward svc/vault -n vault 8200:8200 &
#   ./migrate-secrets-to-vault.sh
#
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
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

    if [[ -z "${VAULT_ADDR:-}" ]]; then
        log_error "VAULT_ADDR environment variable not set"
        exit 1
    fi

    if [[ -z "${VAULT_TOKEN:-}" ]]; then
        log_error "VAULT_TOKEN environment variable not set"
        exit 1
    fi

    # Check Vault connectivity
    if ! vault status &> /dev/null; then
        log_error "Cannot connect to Vault at $VAULT_ADDR"
        exit 1
    fi

    log_info "All prerequisites met"
}

# Migrate a single secret from Kubernetes to Vault
migrate_secret() {
    local k8s_secret_name="$1"
    local vault_path="$2"
    local namespace="${3:-secret-store}"

    log_info "Migrating $k8s_secret_name -> $vault_path"

    # Check if secret exists in Kubernetes
    if ! kubectl get secret "$k8s_secret_name" -n "$namespace" &> /dev/null; then
        log_warn "Secret $k8s_secret_name not found in namespace $namespace, skipping"
        return 0
    fi

    # Export secret data and convert from base64
    local secret_data
    secret_data=$(kubectl get secret "$k8s_secret_name" -n "$namespace" -o json | \
        jq -r '.data | to_entries | map({(.key): (.value | @base64d)}) | add // {}')

    if [[ "$secret_data" == "{}" || "$secret_data" == "null" ]]; then
        log_warn "Secret $k8s_secret_name has no data, skipping"
        return 0
    fi

    # Write to Vault
    echo "$secret_data" | vault kv put "secret/$vault_path" -

    log_info "Successfully migrated $k8s_secret_name to secret/$vault_path"
}

# Main migration function
main() {
    log_info "Starting secret migration from Kubernetes to Vault"
    log_info "Vault address: $VAULT_ADDR"

    check_prerequisites

    # Ensure KV v2 is enabled at 'secret' path
    log_info "Verifying KV v2 secrets engine..."
    if ! vault secrets list | grep -q "^secret/"; then
        log_info "Enabling KV v2 at 'secret' path..."
        vault secrets enable -path=secret kv-v2
    fi

    log_info "Starting migration..."

    # API Keys
    migrate_secret "api-keys" "api-keys"

    # GitHub App secrets for each agent
    for agent in rex blaze morgan cipher cleo tess stitch atlas bolt; do
        migrate_secret "github-app-$agent" "github-app-$agent"
    done

    # GitHub PAT for ARC
    migrate_secret "github-pat" "github-pat"

    # ngrok credentials
    migrate_secret "ngrok-credentials" "ngrok-credentials"

    # Cloudflare credentials (ExternalSecret expects key: cloudflare)
    migrate_secret "cloudflare" "cloudflare"

    # GHCR credentials
    migrate_secret "ghcr-credentials" "ghcr-credentials"

    # ArgoCD repository credentials
    migrate_secret "argocd-repo-credentials" "argocd-repo-credentials"

    # Toolman secrets (individual secrets per service)
    migrate_secret "toolman-brave-search-secrets" "toolman-brave-search-secrets"
    migrate_secret "toolman-kubernetes-secrets" "toolman-kubernetes-secrets"
    migrate_secret "toolman-context7-secrets" "toolman-context7-secrets"
    migrate_secret "toolman-github-secrets" "toolman-github-secrets"

    # Doc server secrets
    migrate_secret "doc-server-secrets" "doc-server-secrets"
    migrate_secret "doc-server-config" "doc-server-config"

    # Redis authentication (used by doc-server databases)
    migrate_secret "redis-auth" "redis-auth"

    # GitHub webhooks secrets
    migrate_secret "github-webhooks" "github-webhooks"

    log_info "Migration complete!"
    log_info ""
    log_info "Next steps:"
    log_info "  1. Verify secrets in Vault: vault kv list secret/"
    log_info "  2. Update ExternalSecrets to use vault-secret-store"
    log_info "  3. Verify ExternalSecrets sync successfully"
    log_info "  4. Remove old secrets from secret-store namespace"
}

main "$@"

