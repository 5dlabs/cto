#!/usr/bin/env bash
# =============================================================================
# kind-secrets-from-1password.sh - Populate K8s secrets from 1Password
# =============================================================================
#
# ⚠️  LOCAL DEVELOPMENT / TESTING ONLY ⚠️
#
# This script is for local Kind cluster development ONLY.
# Production uses HashiCorp Vault with the Vault Secrets Operator.
#
# This script:
#   - Reads secrets from 1Password using the `op` CLI
#   - Creates Kubernetes secrets directly (no operator)
#   - Is NOT intended for production use
#
# Usage:
#   ./scripts/kind-secrets-from-1password.sh --all        # All secrets
#   ./scripts/kind-secrets-from-1password.sh --api-keys   # Just API keys
#   ./scripts/kind-secrets-from-1password.sh --github-apps # GitHub App secrets
#   ./scripts/kind-secrets-from-1password.sh --tools      # Tools server secrets
#   ./scripts/kind-secrets-from-1password.sh --list       # List what will be created
#
# Prerequisites:
#   - 1Password CLI: brew install 1password-cli
#   - Signed in: op signin
#   - kubectl configured for Kind cluster (NOT production!)
#
# Production secrets: See infra/vault/secrets/ for Vault-based approach
# =============================================================================

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}ℹ${NC} $1"; }
log_success() { echo -e "${GREEN}✓${NC} $1"; }
log_warn() { echo -e "${YELLOW}⚠${NC} $1"; }
log_error() { echo -e "${RED}✗${NC} $1"; }
log_header() { echo -e "\n${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"; echo -e "${BLUE}  $1${NC}"; echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"; }

NAMESPACE="${NAMESPACE:-cto}"
OP_VAULT="${OP_VAULT:-Personal}"
DRY_RUN="${DRY_RUN:-false}"

# =============================================================================
# Secret Definitions
# =============================================================================

# API Keys secret (cto-secrets)
# Maps 1Password items to Kubernetes secret keys
declare -A API_KEYS=(
    ["ANTHROPIC_API_KEY"]="Anthropic API Key/credential"
    ["OPENAI_API_KEY"]="OpenAI API Key/credential"
    ["GEMINI_API_KEY"]="Google-Gemini API Key/api_key"
    ["GOOGLE_API_KEY"]="Google API Key/credential"
    ["XAI_API_KEY"]="xAI API Key/credential"
    ["CONTEXT7_API_KEY"]="Context7 API Key/credential"
    ["PERPLEXITY_API_KEY"]="Perplexity API Key/credential"
    ["FIRECRAWL_API_KEY"]="Firecrawl API Key/credential"
)

# GitHub Apps (format: app-id/client-id/private-key from 1Password)
declare -a GITHUB_APPS=(
    "morgan"
    "rex"
    "blaze"
    "cleo"
    "tess"
    "atlas"
    "bolt"
    "cipher"
    "stitch"
    "spark"
    "grizz"
    "nova"
    "tap"
)

# Tools server secrets
declare -A TOOLS_SECRETS=(
    ["tools-brave-search-secrets/BRAVE_API_KEY"]="Brave Search API Key/api_key"
    ["tools-context7-secrets/CONTEXT7_API_KEY"]="Context7 API Key/credential"
    ["tools-github-secrets/GITHUB_PERSONAL_ACCESS_TOKEN"]="GitHub PAT - Tools MCP Server/credential"
    ["tools-firecrawl-secrets/FIRECRAWL_API_KEY"]="Firecrawl API Key/credential"
    ["tools-cloudflare-secrets/CLOUDFLARE_API_TOKEN"]="CloudFlare API/api_token"
    ["tools-gemini-secrets/GEMINI_API_KEY"]="Google-Gemini API Key/api_key"
)

# Webhook secret
WEBHOOK_SECRET_ITEM="GitHub Webhook Secret/secret"

# =============================================================================
# Helper Functions  
# =============================================================================

check_prerequisites() {
    log_header "Checking Prerequisites"
    
    echo -e "${YELLOW}⚠️  LOCAL DEVELOPMENT / TESTING ONLY ⚠️${NC}"
    echo "Production uses Vault Secrets Operator (see infra/vault/)"
    echo ""
    
    # Check 1Password CLI
    if ! command -v op &> /dev/null; then
        log_error "1Password CLI not found. Install with: brew install 1password-cli"
        exit 1
    fi
    log_success "1Password CLI found"
    
    # Check signed in
    if ! op whoami &> /dev/null; then
        log_error "Not signed in to 1Password. Run: op signin"
        exit 1
    fi
    local user
    user=$(op whoami --format json | jq -r '.email')
    log_success "Signed in as: $user"
    
    # Check kubectl
    if ! command -v kubectl &> /dev/null; then
        log_error "kubectl not found"
        exit 1
    fi
    log_success "kubectl found"
    
    # Check context
    local context
    context=$(kubectl config current-context)
    log_success "kubectl context: $context"
    
    # SAFETY: Block production clusters
    if [[ "$context" == *"prod"* ]] || [[ "$context" == *"production"* ]] || [[ "$context" == "admin@simple-cluster" ]]; then
        log_error "BLOCKED: This script is for local Kind development only!"
        log_error "Current context '$context' appears to be a production cluster."
        log_error "Use Vault Secrets Operator for production. See: infra/vault/secrets/"
        exit 1
    fi
    
    # Verify Kind cluster
    if [[ "$context" != *"kind"* ]]; then
        log_warn "Current context doesn't appear to be Kind: $context"
        log_warn "This script is intended for local Kind development only."
        read -p "Continue anyway? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
    
    # Check namespace exists
    if ! kubectl get namespace "$NAMESPACE" &> /dev/null; then
        log_info "Creating namespace: $NAMESPACE"
        kubectl create namespace "$NAMESPACE"
    fi
    log_success "Namespace exists: $NAMESPACE"
}

op_read() {
    local item_path="$1"
    # item_path format: "Item Name/field"
    local item_name="${item_path%/*}"
    local field="${item_path##*/}"
    
    op item get "$item_name" --vault "$OP_VAULT" --fields "$field" 2>/dev/null || echo ""
}

op_read_file() {
    local item_path="$1"
    local item_name="${item_path%/*}"
    local field="${item_path##*/}"
    
    # For private keys, we need to get the file content
    op item get "$item_name" --vault "$OP_VAULT" --fields "$field" 2>/dev/null || echo ""
}

create_secret() {
    local name="$1"
    local namespace="${2:-$NAMESPACE}"
    shift 2
    local literals=("$@")
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would create secret: $name"
        return 0
    fi
    
    # Delete existing secret if present
    kubectl delete secret "$name" -n "$namespace" 2>/dev/null || true
    
    # Build kubectl command
    local cmd="kubectl create secret generic $name -n $namespace"
    for lit in "${literals[@]}"; do
        cmd+=" --from-literal=$lit"
    done
    
    eval "$cmd"
    log_success "Created secret: $name"
}

create_secret_from_file() {
    local name="$1"
    local namespace="$2"
    local key="$3"
    local content="$4"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would create secret: $name (from file)"
        return 0
    fi
    
    # Delete existing
    kubectl delete secret "$name" -n "$namespace" 2>/dev/null || true
    
    # Create from stdin
    echo "$content" | kubectl create secret generic "$name" -n "$namespace" --from-file="$key=/dev/stdin"
    log_success "Created secret: $name"
}

# =============================================================================
# Secret Creation Functions
# =============================================================================

create_api_keys_secret() {
    log_header "Creating API Keys Secret (cto-secrets)"
    
    local literals=()
    local missing=()
    
    for key in "${!API_KEYS[@]}"; do
        local item_path="${API_KEYS[$key]}"
        local value
        value=$(op_read "$item_path")
        
        if [[ -n "$value" ]]; then
            literals+=("$key=$value")
            log_success "  Found: $key"
        else
            missing+=("$key ($item_path)")
            log_warn "  Missing: $key"
        fi
    done
    
    if [[ ${#literals[@]} -gt 0 ]]; then
        create_secret "cto-secrets" "$NAMESPACE" "${literals[@]}"
    fi
    
    if [[ ${#missing[@]} -gt 0 ]]; then
        log_warn "Missing API keys (create in 1Password):"
        for m in "${missing[@]}"; do
            echo "    - $m"
        done
    fi
}

create_github_app_secrets() {
    log_header "Creating GitHub App Secrets"
    
    for agent in "${GITHUB_APPS[@]}"; do
        local item_name="GitHub-App-${agent^}"  # Capitalize first letter
        local secret_name="github-app-5dlabs-${agent}"
        
        log_info "Processing: $agent"
        
        # Try to get fields from 1Password
        local app_id client_id private_key
        app_id=$(op item get "$item_name" --vault "$OP_VAULT" --fields "app-id" 2>/dev/null || echo "")
        client_id=$(op item get "$item_name" --vault "$OP_VAULT" --fields "client-id" 2>/dev/null || echo "")
        private_key=$(op item get "$item_name" --vault "$OP_VAULT" --fields "private-key" 2>/dev/null || echo "")
        
        if [[ -n "$app_id" && -n "$private_key" ]]; then
            if [[ "$DRY_RUN" == "true" ]]; then
                log_info "[DRY RUN] Would create: $secret_name"
            else
                kubectl delete secret "$secret_name" -n "$NAMESPACE" 2>/dev/null || true
                kubectl create secret generic "$secret_name" -n "$NAMESPACE" \
                    --from-literal="app-id=$app_id" \
                    --from-literal="client-id=${client_id:-$app_id}" \
                    --from-literal="private-key=$private_key"
                log_success "  Created: $secret_name"
            fi
        else
            log_warn "  Missing: $item_name (app-id or private-key not found)"
        fi
    done
}

create_tools_secrets() {
    log_header "Creating Tools Server Secrets"
    
    for secret_key in "${!TOOLS_SECRETS[@]}"; do
        local secret_name="${secret_key%/*}"
        local env_key="${secret_key##*/}"
        local item_path="${TOOLS_SECRETS[$secret_key]}"
        
        local value
        value=$(op_read "$item_path")
        
        if [[ -n "$value" ]]; then
            create_secret "$secret_name" "$NAMESPACE" "$env_key=$value"
        else
            log_warn "Missing: $secret_name (1Password: $item_path)"
        fi
    done
}

create_webhook_secret() {
    log_header "Creating Webhook Secret"
    
    local secret
    secret=$(op_read "$WEBHOOK_SECRET_ITEM")
    
    if [[ -n "$secret" ]]; then
        # Webhook secret goes in automation namespace
        kubectl create namespace automation 2>/dev/null || true
        create_secret "github-webhook-secret" "automation" "secret=$secret"
    else
        log_warn "Missing webhook secret (1Password: $WEBHOOK_SECRET_ITEM)"
    fi
}

create_ghcr_secret() {
    log_header "Creating GHCR Pull Secret"
    
    local token
    token=$(op item get "GitHub PAT - GHCR" --vault "$OP_VAULT" --fields "credential" 2>/dev/null || echo "")
    
    if [[ -n "$token" ]]; then
        if [[ "$DRY_RUN" != "true" ]]; then
            kubectl delete secret ghcr-secret -n "$NAMESPACE" 2>/dev/null || true
            kubectl create secret docker-registry ghcr-secret -n "$NAMESPACE" \
                --docker-server=ghcr.io \
                --docker-username=5dlabs \
                --docker-password="$token"
            log_success "Created: ghcr-secret"
        fi
    else
        log_warn "Missing GHCR PAT (not needed for local builds)"
    fi
}

list_secrets() {
    log_header "Secrets to be Created"
    
    echo "API Keys (cto-secrets):"
    for key in "${!API_KEYS[@]}"; do
        echo "  - $key"
    done
    
    echo ""
    echo "GitHub App Secrets:"
    for agent in "${GITHUB_APPS[@]}"; do
        echo "  - github-app-5dlabs-${agent}"
    done
    
    echo ""
    echo "Tools Server Secrets:"
    for secret_key in "${!TOOLS_SECRETS[@]}"; do
        echo "  - ${secret_key%/*}"
    done
    
    echo ""
    echo "Other Secrets:"
    echo "  - github-webhook-secret (automation namespace)"
    echo "  - ghcr-secret (optional for local builds)"
}

show_status() {
    log_header "Current Secrets Status"
    
    echo "Secrets in namespace '$NAMESPACE':"
    kubectl get secrets -n "$NAMESPACE" -o name | while read -r secret; do
        echo "  ${GREEN}✓${NC} ${secret#secret/}"
    done
    
    echo ""
    echo "Secrets in namespace 'automation':"
    kubectl get secrets -n automation -o name 2>/dev/null | while read -r secret; do
        echo "  ${GREEN}✓${NC} ${secret#secret/}"
    done || echo "  (namespace not found)"
}

# =============================================================================
# Main
# =============================================================================

main() {
    case "${1:-}" in
        --all|-a)
            check_prerequisites
            create_api_keys_secret
            create_github_app_secrets
            create_tools_secrets
            create_webhook_secret
            create_ghcr_secret
            show_status
            ;;
        --api-keys)
            check_prerequisites
            create_api_keys_secret
            ;;
        --github-apps)
            check_prerequisites
            create_github_app_secrets
            ;;
        --tools)
            check_prerequisites
            create_tools_secrets
            ;;
        --webhooks)
            check_prerequisites
            create_webhook_secret
            ;;
        --list|-l)
            list_secrets
            ;;
        --status|-s)
            show_status
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            main "${1:---all}"
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --all, -a       Create all secrets"
            echo "  --api-keys      Create only API keys secret (cto-secrets)"
            echo "  --github-apps   Create only GitHub App secrets"
            echo "  --tools         Create only Tools server secrets"
            echo "  --webhooks      Create only webhook secret"
            echo "  --list, -l      List secrets that will be created"
            echo "  --status, -s    Show current secrets status"
            echo "  --dry-run       Show what would be done without doing it"
            echo "  --help, -h      Show this help"
            echo ""
            echo "Environment variables:"
            echo "  NAMESPACE       Kubernetes namespace (default: cto)"
            echo "  OP_VAULT        1Password vault name (default: Personal)"
            echo ""
            echo "1Password Item Names Expected:"
            echo "  - 'Anthropic API Key' with field 'credential'"
            echo "  - 'GitHub-App-Rex' with fields 'app-id', 'client-id', 'private-key'"
            echo "  - etc."
            ;;
        "")
            echo "No option specified. Use --help for usage."
            echo ""
            echo "Quick start:"
            echo "  $0 --list     # See what secrets will be created"
            echo "  $0 --all      # Create all secrets"
            ;;
        *)
            log_error "Unknown option: $1"
            echo "Use --help for usage."
            exit 1
            ;;
    esac
}

main "$@"
