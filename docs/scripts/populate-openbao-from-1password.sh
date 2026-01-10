#!/usr/bin/env bash
#
# populate-openbao-from-1password.sh
#
# Comprehensive script to populate ALL OpenBao secrets from 1Password.
# This is the primary recovery script for restoring secrets after cluster recovery
# or OpenBao data loss.
#
# Prerequisites:
#   - 1Password CLI (op) installed and signed in
#   - kubectl configured with cluster access
#   - OpenBao unsealed and accessible
#   - Port-forward to OpenBao: kubectl port-forward svc/openbao -n openbao 8200:8200
#
# Usage:
#   ./docs/scripts/populate-openbao-from-1password.sh [--dry-run]
#
# After running this script, also run:
#   ./docs/scripts/extract-cluster-secrets.sh
#
# To refresh all ExternalSecrets after populating:
#   kubectl get externalsecrets -A -o name | xargs -I {} kubectl annotate {} force-sync=$(date +%s) --overwrite
#

set -euo pipefail

DRY_RUN=false
if [[ "${1:-}" == "--dry-run" ]]; then
    DRY_RUN=true
    echo "🔍 Dry run mode - no changes will be made"
fi

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
SUCCESS_COUNT=0
SKIP_COUNT=0
FAIL_COUNT=0

echo "════════════════════════════════════════════════════════════════════════════"
echo "  OpenBao Secret Population from 1Password"
echo "════════════════════════════════════════════════════════════════════════════"
echo ""

# Check prerequisites
if ! command -v op &> /dev/null; then
    echo -e "${RED}Error: 1Password CLI (op) not found${NC}"
    echo "Install with: brew install 1password-cli"
    exit 1
fi

if ! command -v vault &> /dev/null && ! command -v bao &> /dev/null; then
    echo -e "${RED}Error: Neither vault nor bao CLI found${NC}"
    echo "Install vault CLI or use kubectl exec to OpenBao pod"
    exit 1
fi

if ! op account list &> /dev/null; then
    echo -e "${YELLOW}1Password not signed in. Running: eval \$(op signin)${NC}"
    eval "$(op signin)"
fi

# Get OpenBao root token - prefer Kubernetes secret, fallback to 1Password
echo "📋 Getting OpenBao root token..."

# First try to get from Kubernetes (most reliable, always current)
OPENBAO_TOKEN=$(kubectl get secret openbao-token -n openbao -o jsonpath='{.data.token}' 2>/dev/null | base64 -d || echo "")

if [[ -z "$OPENBAO_TOKEN" ]]; then
    echo "  Kubernetes secret not found, trying 1Password..."
    OPENBAO_TOKEN=$(op item get "OpenBao Unseal Keys - CTO Platform" --fields "Root Token" --reveal 2>/dev/null || \
                    op item get "OpenBao Unseal Keys - CTO Platform" --fields "password" --reveal 2>/dev/null || echo "")
fi

if [[ -z "$OPENBAO_TOKEN" ]]; then
    echo -e "${RED}Error: Could not get OpenBao root token${NC}"
    echo "Tried: 1) kubectl get secret openbao-token -n openbao"
    echo "       2) 1Password item 'OpenBao Unseal Keys - CTO Platform'"
    exit 1
fi
echo -e "${GREEN}✓ Got OpenBao token${NC}"

# Set up vault/bao CLI
export VAULT_ADDR="${VAULT_ADDR:-http://localhost:8200}"
export VAULT_TOKEN="$OPENBAO_TOKEN"

# Check OpenBao connectivity
if ! vault status &>/dev/null; then
    echo -e "${RED}Error: Cannot connect to OpenBao at $VAULT_ADDR${NC}"
    echo "Make sure OpenBao is port-forwarded:"
    echo "  kubectl port-forward svc/openbao -n openbao 8200:8200"
    exit 1
fi
echo -e "${GREEN}✓ Connected to OpenBao${NC}"
echo ""

# Function to update OpenBao secret
# Usage: update_openbao <path> <key1>=<value1> [<key2>=<value2> ...]
update_openbao() {
    local path=$1
    shift
    
    if [[ "$DRY_RUN" == "true" ]]; then
        echo "    (dry run - would update secret/${path})"
        return 0
    fi
    
    # Pass arguments directly to vault kv put
    if vault kv put "secret/${path}" "$@" > /dev/null 2>&1; then
        echo -e "    ${GREEN}✓ Updated secret/${path}${NC}"
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
        return 0
    else
        echo -e "    ${RED}✗ Failed to update secret/${path}${NC}"
        FAIL_COUNT=$((FAIL_COUNT + 1))
        return 1
    fi
}

# Function to safely get 1Password field
get_op_field() {
    local item="$1"
    local field="$2"
    op item get "$item" --fields "$field" --reveal 2>/dev/null || echo ""
}

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 1: API Keys (for cto-secrets and agent-platform-secrets)
# ═══════════════════════════════════════════════════════════════════════════════
echo "═══════════════════════════════════════════════════════════════════════════════"
echo -e "${BLUE}SECTION 1: API Keys${NC}"
echo "═══════════════════════════════════════════════════════════════════════════════"

ANTHROPIC_KEY=$(get_op_field "Anthropic API Key" "credential")
OPENAI_KEY=$(get_op_field "OpenAI API Key" "credential")
GEMINI_KEY=$(get_op_field "Google-Gemini API Key" "credential")
GOOGLE_KEY=$(get_op_field "Google API Key" "credential")
CURSOR_KEY=$(get_op_field "Cursor API Key" "credential")
CONTEXT7_KEY=$(get_op_field "Context7 API Key" "credential")
PERPLEXITY_KEY=$(get_op_field "Perplexity API Key" "credential")
XAI_KEY=$(get_op_field "xAI API Key" "credential")
FACTORY_KEY=$(get_op_field "Factory API Key" "credential")
BRAVE_KEY=$(get_op_field "Brave Search API Key" "credential")
MINIMAX_KEY=$(get_op_field "MiniMax API Keys" "Main API Key")
MINIMAX_GROUP=$(get_op_field "MiniMax API Keys" "username")

# Build array with only non-empty values
API_KEYS_ARGS=()
[[ -n "$ANTHROPIC_KEY" ]] && API_KEYS_ARGS+=("ANTHROPIC_API_KEY=$ANTHROPIC_KEY")
[[ -n "$OPENAI_KEY" ]] && API_KEYS_ARGS+=("OPENAI_API_KEY=$OPENAI_KEY")
[[ -n "$GEMINI_KEY" ]] && API_KEYS_ARGS+=("GEMINI_API_KEY=$GEMINI_KEY")
[[ -n "$GOOGLE_KEY" ]] && API_KEYS_ARGS+=("GOOGLE_API_KEY=$GOOGLE_KEY")
[[ -n "$CURSOR_KEY" ]] && API_KEYS_ARGS+=("CURSOR_API_KEY=$CURSOR_KEY")
[[ -n "$CONTEXT7_KEY" ]] && API_KEYS_ARGS+=("CONTEXT7_API_KEY=$CONTEXT7_KEY")
[[ -n "$PERPLEXITY_KEY" ]] && API_KEYS_ARGS+=("PERPLEXITY_API_KEY=$PERPLEXITY_KEY")
[[ -n "$XAI_KEY" ]] && API_KEYS_ARGS+=("XAI_API_KEY=$XAI_KEY")
[[ -n "$FACTORY_KEY" ]] && API_KEYS_ARGS+=("FACTORY_API_KEY=$FACTORY_KEY")
[[ -n "$BRAVE_KEY" ]] && API_KEYS_ARGS+=("BRAVE_API_KEY=$BRAVE_KEY")
[[ -n "$MINIMAX_KEY" ]] && API_KEYS_ARGS+=("MINIMAX_API_KEY=$MINIMAX_KEY")
[[ -n "$MINIMAX_GROUP" ]] && API_KEYS_ARGS+=("MINIMAX_GROUP_ID=$MINIMAX_GROUP")

if [[ ${#API_KEYS_ARGS[@]} -gt 0 ]]; then
    echo "📥 Found API keys in 1Password"
    echo "  Keys found: ${#API_KEYS_ARGS[@]}"
    update_openbao "api-keys" "${API_KEYS_ARGS[@]}"
else
    echo -e "${YELLOW}⚠ No API keys found in 1Password${NC}"
    SKIP_COUNT=$((SKIP_COUNT + 1))
fi

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 2: GHCR Docker Registry Credentials
# ═══════════════════════════════════════════════════════════════════════════════
echo ""
echo "═══════════════════════════════════════════════════════════════════════════════"
echo -e "${BLUE}SECTION 2: GHCR Docker Registry${NC}"
echo "═══════════════════════════════════════════════════════════════════════════════"

# GHCR credential may already contain full dockerconfigjson
GHCR_CRED=$(get_op_field "GHCR Pull Secret" "credential")

if [[ -n "$GHCR_CRED" ]]; then
    echo "📥 Found GHCR credentials in 1Password"
    # Check if credential is already dockerconfigjson format
    if echo "$GHCR_CRED" | grep -q '"auths"'; then
        echo "  Format: dockerconfigjson"
        update_openbao "ghcr-secret" ".dockerconfigjson=$GHCR_CRED"
    else
        # Assume it's a PAT and build dockerconfigjson
        GHCR_USER=$(get_op_field "GHCR Pull Secret" "username")
        if [[ -n "$GHCR_USER" ]]; then
            AUTH_B64=$(echo -n "${GHCR_USER}:${GHCR_CRED}" | base64)
            DOCKER_CONFIG="{\"auths\":{\"ghcr.io\":{\"auth\":\"${AUTH_B64}\"}}}"
            update_openbao "ghcr-secret" ".dockerconfigjson=$DOCKER_CONFIG"
        else
            echo -e "${YELLOW}⚠ GHCR username not found${NC}"
            SKIP_COUNT=$((SKIP_COUNT + 1))
        fi
    fi
else
    echo -e "${YELLOW}⚠ GHCR credentials not found in 1Password${NC}"
    echo "   Expected: Item 'GHCR Pull Secret' with field 'credential'"
    SKIP_COUNT=$((SKIP_COUNT + 1))
fi

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 3: Cloudflare API Credentials
# ═══════════════════════════════════════════════════════════════════════════════
echo ""
echo "═══════════════════════════════════════════════════════════════════════════════"
echo -e "${BLUE}SECTION 3: Cloudflare API${NC}"
echo "═══════════════════════════════════════════════════════════════════════════════"

# Try to get Cloudflare API credentials
# Note: If multiple items match "Cloudflare API", use item ID yljt57qq5vo5eocnb4fl6epf3q
CF_KEY=$(op item get "yljt57qq5vo5eocnb4fl6epf3q" --fields credential --reveal 2>/dev/null || echo "")
CF_EMAIL=$(op item get "yljt57qq5vo5eocnb4fl6epf3q" --fields username --reveal 2>/dev/null || echo "")

if [[ -n "$CF_KEY" && -n "$CF_EMAIL" ]]; then
    echo "📥 Found Cloudflare credentials in 1Password"
    echo "  Email: $CF_EMAIL"
    update_openbao "cloudflare" "api-key=$CF_KEY" "email=$CF_EMAIL"
else
    echo -e "${YELLOW}⚠ Cloudflare credentials not found in 1Password${NC}"
    echo "   Expected: Item 'CloudFlare API' (ID: yljt57qq5vo5eocnb4fl6epf3q)"
    SKIP_COUNT=$((SKIP_COUNT + 1))
fi

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 4: Linear API Credentials
# ═══════════════════════════════════════════════════════════════════════════════
echo ""
echo "═══════════════════════════════════════════════════════════════════════════════"
echo -e "${BLUE}SECTION 4: Linear API Credentials${NC}"
echo "═══════════════════════════════════════════════════════════════════════════════"

LINEAR_API_KEY=$(get_op_field "Linear API Credentials" "credential")
LINEAR_CLIENT_ID=$(get_op_field "Linear API Credentials" "Client ID")
LINEAR_CLIENT_SECRET=$(get_op_field "Linear API Credentials" "Client Secret")
LINEAR_OAUTH_TOKEN=$(get_op_field "Linear API Credentials" "OAuth Token")
LINEAR_WEBHOOK_SECRET=$(get_op_field "Linear API Credentials" "Webhook Signing Secret")

LINEAR_ARGS=()
[[ -n "$LINEAR_API_KEY" ]] && LINEAR_ARGS+=("LINEAR_API_KEY=$LINEAR_API_KEY")
[[ -n "$LINEAR_CLIENT_ID" ]] && LINEAR_ARGS+=("LINEAR_CLIENT_ID=$LINEAR_CLIENT_ID")
[[ -n "$LINEAR_CLIENT_SECRET" ]] && LINEAR_ARGS+=("LINEAR_CLIENT_SECRET=$LINEAR_CLIENT_SECRET")
[[ -n "$LINEAR_OAUTH_TOKEN" ]] && LINEAR_ARGS+=("LINEAR_OAUTH_TOKEN=$LINEAR_OAUTH_TOKEN")
[[ -n "$LINEAR_WEBHOOK_SECRET" ]] && LINEAR_ARGS+=("LINEAR_WEBHOOK_SECRET=$LINEAR_WEBHOOK_SECRET")

if [[ ${#LINEAR_ARGS[@]} -gt 0 ]]; then
    echo "📥 Found Linear credentials in 1Password"
    update_openbao "linear-sync" "${LINEAR_ARGS[@]}"
else
    echo -e "${YELLOW}⚠ Linear credentials not found in 1Password${NC}"
    SKIP_COUNT=$((SKIP_COUNT + 1))
fi

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 5: GitHub Apps (one per agent)
# ═══════════════════════════════════════════════════════════════════════════════
echo ""
echo "═══════════════════════════════════════════════════════════════════════════════"
echo -e "${BLUE}SECTION 5: GitHub Apps${NC}"
echo "═══════════════════════════════════════════════════════════════════════════════"

AGENTS=(morgan rex blaze bolt cleo tess atlas cipher spark grizz nova tap stitch vex)

for agent in "${AGENTS[@]}"; do
    # Capitalize first letter (bash-compatible)
    AGENT_TITLE="$(tr '[:lower:]' '[:upper:]' <<< ${agent:0:1})${agent:1}"
    OP_ITEM="GitHub-App-${AGENT_TITLE}"
    
    APP_ID=$(get_op_field "$OP_ITEM" "app-id")
    CLIENT_ID=$(get_op_field "$OP_ITEM" "client-id")
    PRIVATE_KEY=$(get_op_field "$OP_ITEM" "private-key")
    
    if [[ -n "$APP_ID" && -n "$CLIENT_ID" && -n "$PRIVATE_KEY" ]]; then
        echo "📥 Found ${OP_ITEM}"
        update_openbao "github-app-${agent}" "app-id=$APP_ID" "client-id=$CLIENT_ID" "private-key=$PRIVATE_KEY"
    else
        echo -e "${YELLOW}⚠ ${OP_ITEM} not found or incomplete${NC}"
        SKIP_COUNT=$((SKIP_COUNT + 1))
    fi
done

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 6: Alertmanager Discord Webhook
# ═══════════════════════════════════════════════════════════════════════════════
echo ""
echo "═══════════════════════════════════════════════════════════════════════════════"
echo -e "${BLUE}SECTION 6: Alertmanager Discord Webhook${NC}"
echo "═══════════════════════════════════════════════════════════════════════════════"

DISCORD_WEBHOOK=$(get_op_field "Discord Alertmanager Webhook" "credential")

if [[ -n "$DISCORD_WEBHOOK" ]]; then
    echo "📥 Found Discord webhook in 1Password"
    update_openbao "alertmanager-discord" "webhook-url=$DISCORD_WEBHOOK"
else
    echo -e "${YELLOW}⚠ Discord webhook not found in 1Password${NC}"
    SKIP_COUNT=$((SKIP_COUNT + 1))
fi

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 7: Twitter/X Research Auth
# ═══════════════════════════════════════════════════════════════════════════════
echo ""
echo "═══════════════════════════════════════════════════════════════════════════════"
echo -e "${BLUE}SECTION 7: Twitter/X Research Auth${NC}"
echo "═══════════════════════════════════════════════════════════════════════════════"

TWITTER_AUTH=$(get_op_field "Twitter/X Research Auth" "Auth Token")
TWITTER_CT0=$(get_op_field "Twitter/X Research Auth" "CT0")

if [[ -n "$TWITTER_AUTH" && -n "$TWITTER_CT0" ]]; then
    echo "📥 Found Twitter auth in 1Password"
    update_openbao "research-twitter" "TWITTER_AUTH_TOKEN=$TWITTER_AUTH" "TWITTER_CT0=$TWITTER_CT0"
else
    echo -e "${YELLOW}⚠ Twitter auth not found in 1Password${NC}"
    echo "   Expected: Item 'Twitter/X Research Auth' with fields 'Auth Token' and 'CT0'"
    SKIP_COUNT=$((SKIP_COUNT + 1))
fi

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 8: Research Digest Gmail
# ═══════════════════════════════════════════════════════════════════════════════
echo ""
echo "═══════════════════════════════════════════════════════════════════════════════"
echo -e "${BLUE}SECTION 8: Research Digest (Gmail)${NC}"
echo "═══════════════════════════════════════════════════════════════════════════════"

# Try multiple possible item names
GMAIL_USER=$(get_op_field "Research Digest Gmail" "username")
GMAIL_PASS=$(get_op_field "Research Digest Gmail" "credential")
DIGEST_TO=$(get_op_field "Research Digest Gmail" "To Email")

if [[ -z "$GMAIL_USER" ]]; then
    GMAIL_USER=$(get_op_field "Gmail App Password" "username")
    GMAIL_PASS=$(get_op_field "Gmail App Password" "credential")
fi

if [[ -n "$GMAIL_USER" && -n "$GMAIL_PASS" ]]; then
    echo "📥 Found Gmail credentials in 1Password"
    DIGEST_ARGS=("GMAIL_USERNAME=$GMAIL_USER" "GMAIL_APP_PASSWORD=$GMAIL_PASS")
    [[ -n "$DIGEST_TO" ]] && DIGEST_ARGS+=("DIGEST_TO_EMAIL=$DIGEST_TO")
    DIGEST_ARGS+=("DIGEST_BURST_THRESHOLD=10" "DIGEST_MIN_ENTRIES=3")
    update_openbao "research-digest" "${DIGEST_ARGS[@]}"
else
    echo -e "${YELLOW}⚠ Gmail credentials not found in 1Password${NC}"
    echo "   Expected: Item 'Research Digest Gmail' with fields 'username' and 'credential'"
    SKIP_COUNT=$((SKIP_COUNT + 1))
fi

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 9: App Store Connect API
# ═══════════════════════════════════════════════════════════════════════════════
echo ""
echo "═══════════════════════════════════════════════════════════════════════════════"
echo -e "${BLUE}SECTION 9: App Store Connect API${NC}"
echo "═══════════════════════════════════════════════════════════════════════════════"

ASC_KEY_ID=$(get_op_field "App Store Connect API" "Key ID")
ASC_ISSUER_ID=$(get_op_field "App Store Connect API" "Issuer ID")
ASC_P8_KEY=$(get_op_field "App Store Connect API" "credential")

if [[ -n "$ASC_KEY_ID" && -n "$ASC_ISSUER_ID" && -n "$ASC_P8_KEY" ]]; then
    echo "📥 Found App Store Connect credentials in 1Password"
    # Base64 encode the p8 key
    ASC_P8_B64=$(echo -n "$ASC_P8_KEY" | base64)
    update_openbao "tools-appstore-connect" "APP_STORE_KEY_ID=$ASC_KEY_ID" "APP_STORE_ISSUER_ID=$ASC_ISSUER_ID" "APP_STORE_P8_KEY=$ASC_P8_B64"
else
    echo -e "${YELLOW}⚠ App Store Connect credentials not found in 1Password${NC}"
    echo "   Expected: Item 'App Store Connect API' with fields 'Key ID', 'Issuer ID', 'credential'"
    SKIP_COUNT=$((SKIP_COUNT + 1))
fi

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 10: Tools - GitHub PAT (already handled by existing script but include)
# ═══════════════════════════════════════════════════════════════════════════════
echo ""
echo "═══════════════════════════════════════════════════════════════════════════════"
echo -e "${BLUE}SECTION 10: Tools - GitHub PAT${NC}"
echo "═══════════════════════════════════════════════════════════════════════════════"

GITHUB_PAT=$(get_op_field "GitHub PAT - Tools MCP Server" "credential")

if [[ -n "$GITHUB_PAT" ]]; then
    echo "📥 Found GitHub PAT in 1Password"
    update_openbao "tools-github" "GITHUB_PERSONAL_ACCESS_TOKEN=$GITHUB_PAT"
else
    echo -e "${YELLOW}⚠ GitHub PAT not found in 1Password${NC}"
    SKIP_COUNT=$((SKIP_COUNT + 1))
fi

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 11: Tools - Firecrawl
# ═══════════════════════════════════════════════════════════════════════════════
echo ""
echo "═══════════════════════════════════════════════════════════════════════════════"
echo -e "${BLUE}SECTION 11: Tools - Firecrawl${NC}"
echo "═══════════════════════════════════════════════════════════════════════════════"

FIRECRAWL_KEY=$(get_op_field "Firecrawl API Key" "credential")

if [[ -n "$FIRECRAWL_KEY" ]]; then
    echo "📥 Found Firecrawl API key in 1Password"
    update_openbao "tools-firecrawl" "FIRECRAWL_API_KEY=$FIRECRAWL_KEY"
else
    echo -e "${YELLOW}⚠ Firecrawl API key not found in 1Password${NC}"
    SKIP_COUNT=$((SKIP_COUNT + 1))
fi

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 12: Tools - Latitude
# ═══════════════════════════════════════════════════════════════════════════════
echo ""
echo "═══════════════════════════════════════════════════════════════════════════════"
echo -e "${BLUE}SECTION 12: Tools - Latitude${NC}"
echo "═══════════════════════════════════════════════════════════════════════════════"

LATITUDE_KEY=$(get_op_field "Latitude.sh API" "credential")

if [[ -n "$LATITUDE_KEY" ]]; then
    echo "📥 Found Latitude API key in 1Password"
    update_openbao "tools-latitude" "LATITUDE_API_KEY=$LATITUDE_KEY"
else
    echo -e "${YELLOW}⚠ Latitude API key not found in 1Password${NC}"
    SKIP_COUNT=$((SKIP_COUNT + 1))
fi

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 13: Linear OAuth Apps (per-agent)
# ═══════════════════════════════════════════════════════════════════════════════
echo ""
echo "═══════════════════════════════════════════════════════════════════════════════"
echo -e "${BLUE}SECTION 13: Linear OAuth Apps (per-agent)${NC}"
echo "═══════════════════════════════════════════════════════════════════════════════"

# Note: Linear OAuth apps are typically stored with specific naming convention
# For now, we check for the vex agent which was identified as missing
for agent in vex; do
    # Capitalize first letter (bash-compatible)
    AGENT_TITLE="$(tr '[:lower:]' '[:upper:]' <<< ${agent:0:1})${agent:1}"
    OP_ITEM="Linear-App-${AGENT_TITLE}"
    
    CLIENT_ID=$(get_op_field "$OP_ITEM" "client_id")
    CLIENT_SECRET=$(get_op_field "$OP_ITEM" "client_secret")
    WEBHOOK_SECRET=$(get_op_field "$OP_ITEM" "webhook_secret")
    ACCESS_TOKEN=$(get_op_field "$OP_ITEM" "access_token")
    
    if [[ -n "$CLIENT_ID" && -n "$CLIENT_SECRET" ]]; then
        echo "📥 Found ${OP_ITEM}"
        LINEAR_APP_ARGS=("client_id=$CLIENT_ID" "client_secret=$CLIENT_SECRET")
        [[ -n "$WEBHOOK_SECRET" ]] && LINEAR_APP_ARGS+=("webhook_secret=$WEBHOOK_SECRET")
        [[ -n "$ACCESS_TOKEN" ]] && LINEAR_APP_ARGS+=("access_token=$ACCESS_TOKEN")
        update_openbao "linear-app-${agent}" "${LINEAR_APP_ARGS[@]}"
    else
        echo -e "${YELLOW}⚠ ${OP_ITEM} not found in 1Password${NC}"
        echo "   Create Linear OAuth app at: https://linear.app/settings/api"
        SKIP_COUNT=$((SKIP_COUNT + 1))
    fi
done

# ═══════════════════════════════════════════════════════════════════════════════
# SUMMARY
# ═══════════════════════════════════════════════════════════════════════════════
echo ""
echo "════════════════════════════════════════════════════════════════════════════"
echo "  SUMMARY"
echo "════════════════════════════════════════════════════════════════════════════"
echo ""

if [[ "$DRY_RUN" == "true" ]]; then
    echo -e "${YELLOW}Dry run complete. Run without --dry-run to apply changes.${NC}"
else
    echo -e "  ${GREEN}✓ Successful:${NC} $SUCCESS_COUNT secrets"
    echo -e "  ${YELLOW}⚠ Skipped:${NC}    $SKIP_COUNT items (not found in 1Password)"
    echo -e "  ${RED}✗ Failed:${NC}     $FAIL_COUNT secrets"
fi

echo ""
echo "════════════════════════════════════════════════════════════════════════════"
echo "  NEXT STEPS"
echo "════════════════════════════════════════════════════════════════════════════"
echo ""
echo "1. Run the cluster secrets extraction script for infrastructure secrets:"
echo "   ./docs/scripts/extract-cluster-secrets.sh"
echo ""
echo "2. Force refresh all ExternalSecrets:"
echo "   kubectl get externalsecrets -A -o name | xargs -I {} kubectl annotate {} force-sync=\$(date +%s) --overwrite"
echo ""
echo "3. Verify ArgoCD application health:"
echo "   argocd app get external-secrets-config --grpc-web"
echo ""
