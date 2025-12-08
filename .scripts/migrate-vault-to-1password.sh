#!/usr/bin/env bash
# =============================================================================
# migrate-vault-to-1password.sh - Copy secrets from Vault to 1Password
# =============================================================================
# Reads secrets from Vault and creates corresponding items in 1Password.
# This is a one-time migration for setting up local development.
#
# Prerequisites:
#   - Vault CLI: brew install vault
#   - 1Password CLI: brew install 1password-cli
#   - Port-forward to Vault: kubectl port-forward svc/vault -n vault 8200:8200
#   - Signed into 1Password: op signin
#
# Usage:
#   ./scripts/migrate-vault-to-1password.sh --list       # List Vault secrets
#   ./scripts/migrate-vault-to-1password.sh --migrate    # Migrate all
#   ./scripts/migrate-vault-to-1password.sh --api-keys   # Migrate API keys only
#   ./scripts/migrate-vault-to-1password.sh --github-apps # Migrate GitHub Apps
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

OP_VAULT="${OP_VAULT:-Personal}"

# Vault path -> 1Password item name mapping
declare -A VAULT_TO_1P=(
    # API Keys
    ["api-keys"]="CTO API Keys"
    
    # GitHub Apps
    ["github-app-morgan"]="GitHub-App-Morgan"
    ["github-app-rex"]="GitHub-App-Rex"
    ["github-app-blaze"]="GitHub-App-Blaze"
    ["github-app-cleo"]="GitHub-App-Cleo"
    ["github-app-tess"]="GitHub-App-Tess"
    ["github-app-atlas"]="GitHub-App-Atlas"
    ["github-app-bolt"]="GitHub-App-Bolt"
    ["github-app-cipher"]="GitHub-App-Cipher"
    ["github-app-stitch"]="GitHub-App-Stitch"
    ["github-app-spark"]="GitHub-App-Spark"
    
    # Tools
    ["tools-brave-search"]="Brave Search API Key"
    ["tools-firecrawl"]="Firecrawl API Key"
    ["tools-gemini"]="Google-Gemini API Key"
    ["tools-github"]="GitHub PAT - Tools MCP Server"
    
    # Other
    ["github-webhooks"]="GitHub Webhook Secret"
    ["ghcr"]="GitHub PAT - GHCR"
)

check_prerequisites() {
    log_header "Checking Prerequisites"
    
    # Check Vault CLI
    if ! command -v vault &> /dev/null; then
        log_error "Vault CLI not found. Install with: brew install vault"
        exit 1
    fi
    log_success "Vault CLI found"
    
    # Check 1Password CLI
    if ! command -v op &> /dev/null; then
        log_error "1Password CLI not found. Install with: brew install 1password-cli"
        exit 1
    fi
    log_success "1Password CLI found"
    
    # Check 1Password signed in
    if ! op whoami &> /dev/null; then
        log_error "Not signed in to 1Password. Run: op signin"
        exit 1
    fi
    log_success "Signed in to 1Password"
    
    # Get Vault token from 1Password
    local root_token
    root_token=$(op item get "Vault Unseal Keys and Root Token" --vault "$OP_VAULT" --fields "root_token" 2>/dev/null || echo "")
    
    if [[ -z "$root_token" ]]; then
        log_error "Could not get Vault root token from 1Password"
        exit 1
    fi
    
    export VAULT_TOKEN="$root_token"
    export VAULT_ADDR="${VAULT_ADDR:-http://localhost:8200}"
    
    log_success "Vault token loaded from 1Password"
    log_info "Vault address: $VAULT_ADDR"
    
    # Check Vault connection
    if ! vault status &> /dev/null; then
        log_error "Cannot connect to Vault at $VAULT_ADDR"
        log_info "Start port-forward: kubectl port-forward svc/vault -n vault 8200:8200"
        exit 1
    fi
    log_success "Connected to Vault"
}

list_vault_secrets() {
    log_header "Vault Secrets (secret/)"
    
    echo "Available paths:"
    vault kv list secret/ 2>/dev/null | while read -r path; do
        echo "  - $path"
    done
}

read_vault_secret() {
    local path="$1"
    vault kv get -format=json "secret/$path" 2>/dev/null | jq -r '.data.data'
}

create_1p_api_key_item() {
    local name="$1"
    local key_name="$2"
    local key_value="$3"
    
    # Check if item already exists
    if op item get "$name" --vault "$OP_VAULT" &>/dev/null; then
        log_warn "Item '$name' already exists, skipping (use --force to overwrite)"
        return 0
    fi
    
    log_info "Creating: $name"
    op item create \
        --category="API Credential" \
        --title="$name" \
        --vault="$OP_VAULT" \
        "credential=$key_value" \
        >/dev/null
    log_success "Created: $name"
}

create_1p_github_app_item() {
    local name="$1"
    local app_id="$2"
    local client_id="$3"
    local private_key="$4"
    
    if op item get "$name" --vault "$OP_VAULT" &>/dev/null; then
        log_warn "Item '$name' already exists, skipping"
        return 0
    fi
    
    log_info "Creating: $name"
    
    # Create item with fields
    # Note: Private key needs special handling for multi-line
    op item create \
        --category="API Credential" \
        --title="$name" \
        --vault="$OP_VAULT" \
        "app-id=$app_id" \
        "client-id=$client_id" \
        "private-key[password]=$private_key" \
        >/dev/null
    log_success "Created: $name"
}

migrate_api_keys() {
    log_header "Migrating API Keys"
    
    local data
    data=$(read_vault_secret "api-keys")
    
    if [[ -z "$data" || "$data" == "null" ]]; then
        log_warn "No api-keys secret found in Vault"
        return 1
    fi
    
    # Parse each key and create 1Password items
    echo "$data" | jq -r 'to_entries[] | "\(.key)=\(.value)"' | while IFS='=' read -r key value; do
        local item_name=""
        case "$key" in
            ANTHROPIC_API_KEY) item_name="Anthropic API Key" ;;
            OPENAI_API_KEY) item_name="OpenAI API Key" ;;
            GEMINI_API_KEY) item_name="Google-Gemini API Key" ;;
            GOOGLE_API_KEY) item_name="Google API Key" ;;
            XAI_API_KEY) item_name="xAI API Key" ;;
            PERPLEXITY_API_KEY) item_name="Perplexity API Key" ;;
            *) item_name="$key" ;;
        esac
        
        if [[ -n "$item_name" && -n "$value" ]]; then
            create_1p_api_key_item "$item_name" "$key" "$value"
        fi
    done
}

migrate_github_apps() {
    log_header "Migrating GitHub App Secrets"
    
    for agent in morgan rex blaze cleo tess atlas bolt cipher stitch spark; do
        local vault_path="github-app-${agent}"
        local op_name="GitHub-App-${agent^}"  # Capitalize
        
        log_info "Reading: $vault_path"
        local data
        data=$(read_vault_secret "$vault_path" 2>/dev/null || echo "")
        
        if [[ -z "$data" || "$data" == "null" ]]; then
            log_warn "Not found in Vault: $vault_path"
            continue
        fi
        
        local app_id client_id private_key
        app_id=$(echo "$data" | jq -r '.["app-id"] // .appId // .APP_ID // ""')
        client_id=$(echo "$data" | jq -r '.["client-id"] // .clientId // .CLIENT_ID // ""')
        private_key=$(echo "$data" | jq -r '.["private-key"] // .privateKey // .PRIVATE_KEY // ""')
        
        if [[ -n "$app_id" && -n "$private_key" ]]; then
            create_1p_github_app_item "$op_name" "$app_id" "${client_id:-$app_id}" "$private_key"
        else
            log_warn "Missing required fields for $vault_path"
        fi
    done
}

migrate_tools_secrets() {
    log_header "Migrating Tools Secrets"
    
    # Brave Search
    local brave_data
    brave_data=$(read_vault_secret "tools-brave-search" 2>/dev/null || echo "")
    if [[ -n "$brave_data" && "$brave_data" != "null" ]]; then
        local brave_key
        brave_key=$(echo "$brave_data" | jq -r '.BRAVE_API_KEY // ""')
        if [[ -n "$brave_key" ]]; then
            create_1p_api_key_item "Brave Search API Key" "BRAVE_API_KEY" "$brave_key"
        fi
    fi
    
    # Firecrawl
    local firecrawl_data
    firecrawl_data=$(read_vault_secret "tools-firecrawl" 2>/dev/null || echo "")
    if [[ -n "$firecrawl_data" && "$firecrawl_data" != "null" ]]; then
        local firecrawl_key
        firecrawl_key=$(echo "$firecrawl_data" | jq -r '.FIRECRAWL_API_KEY // ""')
        if [[ -n "$firecrawl_key" ]]; then
            create_1p_api_key_item "Firecrawl API Key" "FIRECRAWL_API_KEY" "$firecrawl_key"
        fi
    fi
    
    # Webhook secret
    local webhook_data
    webhook_data=$(read_vault_secret "github-webhooks" 2>/dev/null || echo "")
    if [[ -n "$webhook_data" && "$webhook_data" != "null" ]]; then
        local webhook_secret
        webhook_secret=$(echo "$webhook_data" | jq -r '.secret // .SECRET // ""')
        if [[ -n "$webhook_secret" ]]; then
            create_1p_api_key_item "GitHub Webhook Secret" "secret" "$webhook_secret"
        fi
    fi
}

migrate_all() {
    migrate_api_keys
    migrate_github_apps
    migrate_tools_secrets
    
    log_header "Migration Complete"
    echo "Run the following to verify:"
    echo "  op item list --vault $OP_VAULT | grep -E '(API|GitHub|Key)'"
}

# =============================================================================
# Main
# =============================================================================

main() {
    case "${1:-}" in
        --list|-l)
            check_prerequisites
            list_vault_secrets
            ;;
        --migrate|--all|-a)
            check_prerequisites
            migrate_all
            ;;
        --api-keys)
            check_prerequisites
            migrate_api_keys
            ;;
        --github-apps)
            check_prerequisites
            migrate_github_apps
            ;;
        --tools)
            check_prerequisites
            migrate_tools_secrets
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --list, -l       List Vault secrets"
            echo "  --migrate, -a    Migrate all secrets"
            echo "  --api-keys       Migrate API keys only"
            echo "  --github-apps    Migrate GitHub Apps only"
            echo "  --tools          Migrate tools secrets only"
            echo "  --help, -h       Show this help"
            echo ""
            echo "Prerequisites:"
            echo "  1. Port-forward Vault: kubectl port-forward svc/vault -n vault 8200:8200"
            echo "  2. Sign in to 1Password: op signin"
            echo ""
            ;;
        "")
            echo "Usage: $0 [--list|--migrate|--api-keys|--github-apps|--tools]"
            echo ""
            echo "Prerequisites:"
            echo "  1. Port-forward Vault: kubectl port-forward svc/vault -n vault 8200:8200"
            echo "  2. Sign in to 1Password: op signin"
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
}

main "$@"



