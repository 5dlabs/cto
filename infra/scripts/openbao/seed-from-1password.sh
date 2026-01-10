#!/bin/bash
#
# seed-from-1password.sh
#
# Seeds OpenBao with secrets from 1Password for the CTO platform.
# This is a ONE-TIME bootstrap operation for new environments or after OpenBao reset.
#
# Prerequisites:
#   - 1Password CLI (op) installed and signed in
#   - kubectl configured with cluster access
#   - OpenBao running and unsealed
#
# Usage:
#   ./seed-from-1password.sh [OPTIONS]
#
# Options:
#   --dry-run           Preview changes without applying
#   --category <name>   Only seed specific category (github-apps, linear-apps, api-keys, tools, infrastructure, research)
#   --help              Show this help message
#
# Examples:
#   ./seed-from-1password.sh --dry-run              # Preview all changes
#   ./seed-from-1password.sh                        # Seed all secrets
#   ./seed-from-1password.sh --category github-apps # Only seed GitHub Apps
#

set -euo pipefail

# =============================================================================
# Configuration
# =============================================================================

DRY_RUN=false
CATEGORY=""

# Agent list (used for GitHub Apps and Linear Apps)
AGENTS=(atlas blaze bolt cipher cleo grizz morgan nova rex spark stitch tap tess vex)

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# =============================================================================
# Helper Functions
# =============================================================================

log_info() { echo -e "${BLUE}ℹ${NC} $1"; }
log_success() { echo -e "${GREEN}✓${NC} $1"; }
log_warning() { echo -e "${YELLOW}⚠${NC} $1"; }
log_error() { echo -e "${RED}✗${NC} $1"; }
log_header() { echo -e "\n${BLUE}═══════════════════════════════════════════════════════════════${NC}\n$1\n${BLUE}═══════════════════════════════════════════════════════════════${NC}"; }

show_help() {
    head -30 "$0" | grep -E "^#" | sed 's/^# \?//'
    exit 0
}

# Get OpenBao root token from Kubernetes secret
get_bao_token() {
    kubectl get secret openbao-token -n openbao -o jsonpath='{.data.token}' | base64 -d
}

# Put a secret in OpenBao safely using JSON via stdin
# Usage: bao_put "path" "key1" "value1" "key2" "value2" ...
# Each key and value is passed as a separate argument to avoid shell injection
bao_put() {
    local path=$1
    shift
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "(dry-run) Would put secret/$path"
        return 0
    fi
    
    # Build JSON object from key-value pairs
    local json="{"
    local first=true
    while [[ $# -ge 2 ]]; do
        local key=$1
        local value=$2
        shift 2
        
        # Escape JSON special characters in value
        local escaped_value
        escaped_value=$(printf '%s' "$value" | jq -Rs '.')
        
        if [[ "$first" == "true" ]]; then
            first=false
        else
            json="$json,"
        fi
        json="$json\"$key\":$escaped_value"
    done
    json="$json}"
    
    local token
    token=$(get_bao_token)
    
    # Pass token, path, and JSON via stdin using newline delimiters to avoid shell injection
    # Format: TOKEN\nPATH\nJSON - the script inside reads each value safely
    # This avoids interpolating any values into sh -c which could allow injection
    # if values contained shell metacharacters (quotes, backticks, $, etc.)
    local payload
    payload=$(printf '%s\n%s\n%s' "$token" "$path" "$json")
    
    # shellcheck disable=SC2016 # Variables are intentionally expanded inside container, not locally
    if printf '%s' "$payload" | kubectl exec -i -n openbao openbao-0 -- sh -c '
        read -r BAO_TOKEN
        read -r SECRET_PATH
        export BAO_TOKEN
        cat | bao kv put "secret/${SECRET_PATH}" -
    ' >/dev/null 2>&1; then
        log_success "secret/$path"
        return 0
    else
        log_error "Failed to put secret/$path"
        return 1
    fi
}

# Check if 1Password item exists
op_item_exists() {
    op item get "$1" --format json >/dev/null 2>&1
}

# Get field from 1Password item
op_get_field() {
    local item=$1
    local field=$2
    op item get "$item" --fields "$field" --reveal 2>/dev/null || echo ""
}

# Get field from 1Password item by section
op_get_field_section() {
    local item=$1
    local section=$2
    local field=$3
    op item get "$item" --format json 2>/dev/null | jq -r ".fields[] | select(.section.label == \"$section\" and .label == \"$field\") | .value" 2>/dev/null || echo ""
}

# =============================================================================
# Seeding Functions
# =============================================================================

seed_github_apps() {
    log_header "GitHub Apps (14 agents)"
    
    local success=0
    local failed=0
    
    for agent in "${AGENTS[@]}"; do
        # Capitalize first letter (bash-native, portable)
        local agent_title="${agent^}"
        local op_item="GitHub-App-${agent_title}"
        
        if ! op_item_exists "$op_item"; then
            log_warning "1Password item not found: $op_item"
            ((failed++))
            continue
        fi
        
        local app_id client_id private_key
        app_id=$(op_get_field "$op_item" "app-id")
        client_id=$(op_get_field "$op_item" "client-id")
        private_key=$(op_get_field "$op_item" "private-key")
        
        if [[ -z "$app_id" || -z "$client_id" || -z "$private_key" ]]; then
            log_warning "Missing fields for $op_item (app-id: ${#app_id}, client-id: ${#client_id}, private-key: ${#private_key})"
            ((failed++))
            continue
        fi
        
        if bao_put "github-app-$agent" "app-id" "$app_id" "client-id" "$client_id" "private-key" "$private_key"; then
            ((success++))
        else
            ((failed++))
        fi
    done
    
    log_info "GitHub Apps: $success succeeded, $failed failed"
}

seed_linear_apps() {
    log_header "Linear Apps (14 agents)"
    
    local op_item="Linear Agent Client Secrets (Rotated 2026-01-02)"
    
    if ! op_item_exists "$op_item"; then
        log_error "1Password item not found: $op_item"
        return 1
    fi
    
    local success=0
    local failed=0
    
    for agent in "${AGENTS[@]}"; do
        # Capitalize first letter (bash-native, portable)
        local agent_title="${agent^}"
        
        # Get client_id and client_secret from the agent's section
        local client_id client_secret webhook_secret access_token
        client_id=$(op_get_field_section "$op_item" "$agent_title" "client_id")
        client_secret=$(op_get_field_section "$op_item" "$agent_title" "client_secret")
        webhook_secret=$(op_get_field_section "$op_item" "$agent_title" "webhook_secret")
        access_token=$(op_get_field_section "$op_item" "$agent_title" "access_token")
        
        # If client_id is missing, check if it's stored differently
        if [[ -z "$client_id" ]]; then
            # Try getting from a different field name
            client_id=$(op item get "$op_item" --format json 2>/dev/null | jq -r ".fields[] | select(.section.label == \"$agent_title\") | select(.label | test(\"client.*id\"; \"i\")) | .value" 2>/dev/null | head -1 || echo "")
        fi
        
        if [[ -z "$client_secret" ]]; then
            log_warning "Missing client_secret for linear-app-$agent"
            ((failed++))
            continue
        fi
        
        # Use empty strings for missing optional fields
        client_id="${client_id:-}"
        webhook_secret="${webhook_secret:-}"
        access_token="${access_token:-}"
        
        if bao_put "linear-app-$agent" "client_id" "$client_id" "client_secret" "$client_secret" "webhook_secret" "$webhook_secret" "access_token" "$access_token"; then
            ((success++))
        else
            ((failed++))
        fi
    done
    
    log_info "Linear Apps: $success succeeded, $failed failed"
}

seed_api_keys() {
    log_header "API Keys (combined secret)"
    
    # Map of API key name -> 1Password item name -> field name
    declare -A API_KEYS=(
        ["ANTHROPIC_API_KEY"]="Anthropic API Key:credential"  # pragma: allowlist secret
        ["OPENAI_API_KEY"]="OpenAI API Key:credential"  # pragma: allowlist secret
        ["GEMINI_API_KEY"]="Google-Gemini API Key:credential"  # pragma: allowlist secret
        ["GOOGLE_API_KEY"]="Google API Key:credential"  # pragma: allowlist secret
        ["CURSOR_API_KEY"]="Cursor API Key:credential"  # pragma: allowlist secret
        ["CONTEXT7_API_KEY"]="Context7 API Key:credential"  # pragma: allowlist secret
        ["PERPLEXITY_API_KEY"]="Perplexity API Key:credential"  # pragma: allowlist secret
        ["XAI_API_KEY"]="xAI API Key:credential"  # pragma: allowlist secret
        ["FACTORY_API_KEY"]="Factory API Key:credential"  # pragma: allowlist secret
        ["BRAVE_API_KEY"]="Brave Search API Key:credential"  # pragma: allowlist secret
        ["MINIMAX_API_KEY"]="MiniMax API Keys:credential"  # pragma: allowlist secret
        ["MINIMAX_GROUP_ID"]="MiniMax API Keys:Group ID"
    )
    
    # Build array of key-value pairs for bao_put
    local kv_args=()
    local success=0
    local failed=0
    
    for key in "${!API_KEYS[@]}"; do
        IFS=':' read -r item field <<< "${API_KEYS[$key]}"
        
        if ! op_item_exists "$item"; then
            log_warning "1Password item not found: $item (for $key)"
            ((failed++))
            continue
        fi
        
        local value
        value=$(op_get_field "$item" "$field")
        
        if [[ -z "$value" ]]; then
            log_warning "Empty value for $key from $item:$field"
            ((failed++))
            continue
        fi
        
        kv_args+=("$key" "$value")
        ((success++))
    done
    
    if [[ ${#kv_args[@]} -gt 0 ]]; then
        bao_put "api-keys" "${kv_args[@]}"
    fi
    
    log_info "API Keys: $success found, $failed missing"
}

seed_tools() {
    log_header "Tool Secrets"
    
    # tools-github (already synced earlier, but include for completeness)
    local github_token
    github_token=$(op_get_field "GitHub PAT - Tools MCP Server" "credential")
    if [[ -n "$github_token" ]]; then
        bao_put "tools-github" "GITHUB_PERSONAL_ACCESS_TOKEN" "$github_token"
    else
        log_warning "Missing: GitHub PAT - Tools MCP Server"
    fi
    
    # tools-firecrawl
    local firecrawl_key
    firecrawl_key=$(op_get_field "Firecrawl API Key" "credential")
    if [[ -n "$firecrawl_key" ]]; then
        bao_put "tools-firecrawl" "FIRECRAWL_API_KEY" "$firecrawl_key"
    else
        log_warning "Missing: Firecrawl API Key"
    fi
    
    # tools-latitude
    local latitude_key
    latitude_key=$(op_get_field "Latitude.sh API" "credential")
    if [[ -n "$latitude_key" ]]; then
        bao_put "tools-latitude" "LATITUDE_API_KEY" "$latitude_key"
    else
        log_warning "Missing: Latitude.sh API"
    fi
    
    # tools-kubernetes
    local kubeconfig
    kubeconfig=$(op_get_field "Kubeconfig - Latitude cto-dal" "credential")
    if [[ -n "$kubeconfig" ]]; then
        bao_put "tools-kubernetes" "KUBECONFIG" "$kubeconfig"
    else
        log_warning "Missing: Kubeconfig - Latitude cto-dal"
    fi
    
    # tools-appstore-connect
    if op_item_exists "App Store Connect API"; then
        local key_id issuer_id p8_key
        key_id=$(op_get_field "App Store Connect API" "key-id")
        issuer_id=$(op_get_field "App Store Connect API" "issuer-id")
        p8_key=$(op_get_field "App Store Connect API" "private-key")
        
        if [[ -n "$key_id" && -n "$issuer_id" && -n "$p8_key" ]]; then
            bao_put "tools-appstore-connect" "APP_STORE_KEY_ID" "$key_id" "APP_STORE_ISSUER_ID" "$issuer_id" "APP_STORE_P8_KEY" "$p8_key"
        else
            log_warning "Missing fields in App Store Connect API"
        fi
    else
        log_warning "Missing: App Store Connect API"
    fi
}

seed_infrastructure() {
    log_header "Infrastructure Secrets"
    
    # cloudflare
    local cf_email cf_key
    cf_email=$(op item get "Cloudflare API" --fields "username" --reveal 2>/dev/null || echo "")
    cf_key=$(op item get "Cloudflare API" --fields "credential" --reveal 2>/dev/null || echo "")
    if [[ -n "$cf_email" && -n "$cf_key" ]]; then
        bao_put "cloudflare" "email" "$cf_email" "api-key" "$cf_key"
    else
        log_warning "Missing: Cloudflare API credentials"
    fi
    
    # ghcr-secret (Docker config JSON format)
    local ghcr_user ghcr_pass
    ghcr_user=$(op item get "GHCR Pull Secret" --fields "username" --reveal 2>/dev/null || echo "")
    ghcr_pass=$(op item get "GHCR Pull Secret" --fields "credential" --reveal 2>/dev/null || echo "")
    if [[ -n "$ghcr_user" && -n "$ghcr_pass" ]]; then
        # Create dockerconfigjson format
        local auth
        auth=$(echo -n "$ghcr_user:$ghcr_pass" | base64)
        local dockerconfig="{\"auths\":{\"ghcr.io\":{\"username\":\"$ghcr_user\",\"password\":\"$ghcr_pass\",\"auth\":\"$auth\"}}}"
        bao_put "ghcr-secret" ".dockerconfigjson" "$dockerconfig"
    else
        log_warning "Missing: GHCR Pull Secret"
    fi
    
    # alertmanager-discord
    local discord_webhook
    discord_webhook=$(op item get "Discord Alertmanager Webhook" --fields "credential" --reveal 2>/dev/null || echo "")
    if [[ -n "$discord_webhook" ]]; then
        bao_put "alertmanager-discord" "webhook-url" "$discord_webhook"
    else
        log_warning "Missing: Discord Alertmanager Webhook"
    fi
    
    # linear-sync (Linear API for the platform)
    local linear_api_key linear_webhook_secret
    linear_api_key=$(op item get "Linear API Credentials" --fields "credential" --reveal 2>/dev/null || echo "")
    linear_webhook_secret=$(op item get "Linear API Credentials" --fields "Webhook Signing Secret" --reveal 2>/dev/null || echo "")
    if [[ -n "$linear_api_key" ]]; then
        # Get additional fields if they exist
        local linear_client_id linear_client_secret linear_oauth_token github_pat
        linear_client_id=$(op item get "Linear API Credentials" --fields "client_id" --reveal 2>/dev/null || echo "")
        linear_client_secret=$(op item get "Linear API Credentials" --fields "client_secret" --reveal 2>/dev/null || echo "")
        linear_oauth_token=$(op item get "Linear API Credentials" --fields "oauth_token" --reveal 2>/dev/null || echo "")
        github_pat=$(op item get "GitHub PAT - Tools MCP Server" --fields "credential" --reveal 2>/dev/null || echo "")
        
        bao_put "linear-sync" \
            "LINEAR_API_KEY" "$linear_api_key" \
            "LINEAR_WEBHOOK_SECRET" "${linear_webhook_secret:-}" \
            "LINEAR_CLIENT_ID" "${linear_client_id:-}" \
            "LINEAR_CLIENT_SECRET" "${linear_client_secret:-}" \
            "LINEAR_OAUTH_TOKEN" "${linear_oauth_token:-}" \
            "GITHUB_PERSONAL_ACCESS_TOKEN" "${github_pat:-}"
    else
        log_warning "Missing: Linear API Credentials"
    fi
    
    # tailscale-auth - Check if item exists
    log_warning "tailscale-auth: Needs manual configuration (no 1Password item found)"
    
    # mayastor - Check if item exists  
    log_warning "mayastor: Needs manual configuration (no 1Password item found)"
    
    # seaweedfs-s3-credentials - Check if item exists
    log_warning "seaweedfs-s3-credentials: Needs manual configuration (no 1Password item found)"
}

seed_research() {
    log_header "Research Secrets"
    
    # research-twitter
    if op_item_exists "Twitter/X Research Auth"; then
        local auth_token ct0
        auth_token=$(op item get "Twitter/X Research Auth" --fields "auth_token" --reveal 2>/dev/null || echo "")
        ct0=$(op item get "Twitter/X Research Auth" --fields "ct0" --reveal 2>/dev/null || echo "")
        
        if [[ -n "$auth_token" ]]; then
            bao_put "research-twitter" "TWITTER_AUTH_TOKEN" "$auth_token" "TWITTER_CT0" "${ct0:-}"
        else
            log_warning "Missing auth_token in Twitter/X Research Auth"
        fi
    else
        log_warning "Missing: Twitter/X Research Auth"
    fi
    
    # research-digest
    log_warning "research-digest: Needs manual configuration (no 1Password item found)"
}

# =============================================================================
# Main
# =============================================================================

main() {
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            --category)
                CATEGORY="$2"
                shift 2
                ;;
            --help|-h)
                show_help
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                ;;
        esac
    done
    
    echo ""
    echo "╔═══════════════════════════════════════════════════════════════╗"
    echo "║         OpenBao Secret Seeding from 1Password                 ║"
    echo "╚═══════════════════════════════════════════════════════════════╝"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_warning "DRY RUN MODE - No changes will be made"
    fi
    
    # Check prerequisites
    if ! command -v op &> /dev/null; then
        log_error "1Password CLI (op) not found. Install with: brew install 1password-cli"
        exit 1
    fi
    
    if ! op account list &> /dev/null; then
        log_warning "1Password not signed in. Running: eval \$(op signin)"
        eval "$(op signin)"
    fi
    
    if ! kubectl get pods -n openbao &> /dev/null; then
        log_error "Cannot access OpenBao namespace. Check kubectl configuration."
        exit 1
    fi
    
    # Run seeding based on category or all
    case "$CATEGORY" in
        "")
            seed_github_apps
            seed_linear_apps
            seed_api_keys
            seed_tools
            seed_infrastructure
            seed_research
            ;;
        github-apps)
            seed_github_apps
            ;;
        linear-apps)
            seed_linear_apps
            ;;
        api-keys)
            seed_api_keys
            ;;
        tools)
            seed_tools
            ;;
        infrastructure)
            seed_infrastructure
            ;;
        research)
            seed_research
            ;;
        *)
            log_error "Unknown category: $CATEGORY"
            log_info "Valid categories: github-apps, linear-apps, api-keys, tools, infrastructure, research"
            exit 1
            ;;
    esac
    
    echo ""
    log_header "Complete"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "Dry run complete. Run without --dry-run to apply changes."
    else
        log_success "Seeding complete. Refresh ExternalSecrets with:"
        echo "  kubectl annotate externalsecret -A --all force-sync=\"\$(date +%s)\" --overwrite"
    fi
}

main "$@"
