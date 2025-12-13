#!/usr/bin/env bash
#
# sync-credentials-from-1password.sh
#
# Syncs critical credentials from 1Password to OpenBao and refreshes Kubernetes secrets.
# This ensures credentials in the cluster match what's stored in 1Password.
#
# Prerequisites:
#   - 1Password CLI (op) installed and signed in
#   - kubectl configured with cluster access
#   - OpenBao root token stored in 1Password item "OpenBao Unseal Keys - CTO Platform"
#
# Usage:
#   ./scripts/sync-credentials-from-1password.sh [--dry-run]
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
NC='\033[0m' # No Color

echo "🔐 Syncing credentials from 1Password to OpenBao..."
echo ""

# Check prerequisites
if ! command -v op &> /dev/null; then
    echo -e "${RED}Error: 1Password CLI (op) not found${NC}"
    exit 1
fi

if ! op account list &> /dev/null; then
    echo -e "${YELLOW}1Password not signed in. Running: eval \$(op signin)${NC}"
    eval "$(op signin)"
fi

# Get OpenBao root token
echo "📋 Getting OpenBao root token from 1Password..."
OPENBAO_TOKEN=$(op item get "OpenBao Unseal Keys - CTO Platform" --fields "Root Token" --reveal)
if [[ -z "$OPENBAO_TOKEN" ]]; then
    echo -e "${RED}Error: Could not get OpenBao root token${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Got OpenBao token${NC}"

# Function to update OpenBao secret
update_openbao() {
    local path=$1
    shift
    local data="$*"
    
    echo "  Updating OpenBao: secret/${path}"
    if [[ "$DRY_RUN" == "true" ]]; then
        echo "    (dry run - would update with: ${data:0:50}...)"
        return 0
    fi
    
    kubectl exec -n openbao openbao-0 -- sh -c "
        export VAULT_TOKEN='$OPENBAO_TOKEN'
        bao kv put secret/${path} ${data}
    " > /dev/null && echo -e "    ${GREEN}✓ Updated${NC}" || echo -e "    ${RED}✗ Failed${NC}"
}

# Function to refresh ExternalSecret
refresh_externalsecret() {
    local namespace=$1
    local name=$2
    
    echo "  Refreshing ExternalSecret: ${namespace}/${name}"
    if [[ "$DRY_RUN" == "true" ]]; then
        echo "    (dry run - would annotate)"
        return 0
    fi
    
    kubectl annotate externalsecret "$name" -n "$namespace" \
        force-sync="$(date +%s)" --overwrite > /dev/null 2>&1 \
        && echo -e "    ${GREEN}✓ Refreshed${NC}" || echo -e "    ${YELLOW}⚠ Not found${NC}"
}

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "1. Cloudflare API Credentials"
echo "═══════════════════════════════════════════════════════════════"

CF_KEY=$(op item get "CloudFlare API" --fields credential --reveal 2>/dev/null || echo "")
CF_EMAIL=$(op item get "CloudFlare API" --fields username --reveal 2>/dev/null || echo "")

if [[ -n "$CF_KEY" && -n "$CF_EMAIL" ]]; then
    echo "📥 Found in 1Password: CloudFlare API"
    echo "  Email: $CF_EMAIL"
    echo "  Key length: ${#CF_KEY}"
    update_openbao "cloudflare" "api-key='$CF_KEY' email='$CF_EMAIL'"
    refresh_externalsecret "cloudflare-operator-system" "cloudflare-api-credentials"
    refresh_externalsecret "infra" "cloudflare-api-credentials"
else
    echo -e "${YELLOW}⚠ Cloudflare API credentials not found in 1Password${NC}"
fi

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "2. GitHub PAT (Tools MCP Server)"
echo "═══════════════════════════════════════════════════════════════"

GITHUB_TOKEN=$(op item get "GitHub PAT - Tools MCP Server" --fields credential --reveal 2>/dev/null || echo "")

if [[ -n "$GITHUB_TOKEN" ]]; then
    echo "📥 Found in 1Password: GitHub PAT - Tools MCP Server"
    echo "  Token length: ${#GITHUB_TOKEN}"
    update_openbao "tools-github" "GITHUB_PERSONAL_ACCESS_TOKEN='$GITHUB_TOKEN'"
    refresh_externalsecret "cto" "tools-github-secrets"
else
    echo -e "${YELLOW}⚠ GitHub PAT not found in 1Password${NC}"
fi

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "3. Linear API Credentials"
echo "═══════════════════════════════════════════════════════════════"

LINEAR_API_KEY=$(op item get "Linear API Credentials" --fields credential --reveal 2>/dev/null || echo "")
LINEAR_WEBHOOK_SECRET=$(op item get "Linear API Credentials" --fields "Webhook Signing Secret" --reveal 2>/dev/null || echo "")

if [[ -n "$LINEAR_API_KEY" ]]; then
    echo "📥 Found in 1Password: Linear API Credentials"
    echo "  API Key length: ${#LINEAR_API_KEY}"
    echo "  Webhook Secret: ${LINEAR_WEBHOOK_SECRET:+found}"
    update_openbao "linear-sync" "api-key='$LINEAR_API_KEY' webhook-secret='${LINEAR_WEBHOOK_SECRET:-}'"
    refresh_externalsecret "cto" "linear-secrets"
else
    echo -e "${YELLOW}⚠ Linear API credentials not found in 1Password${NC}"
fi

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "4. Firecrawl API"
echo "═══════════════════════════════════════════════════════════════"

FIRECRAWL_KEY=$(op item get "Firecrawl API Key" --fields credential --reveal 2>/dev/null || echo "")

if [[ -n "$FIRECRAWL_KEY" ]]; then
    echo "📥 Found in 1Password: Firecrawl API Key"
    echo "  Key length: ${#FIRECRAWL_KEY}"
    update_openbao "tools-firecrawl" "FIRECRAWL_API_KEY='$FIRECRAWL_KEY'"
    refresh_externalsecret "cto" "tools-firecrawl-secrets"
else
    echo -e "${YELLOW}⚠ Firecrawl API key not found in 1Password${NC}"
fi

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "Done!"
echo "═══════════════════════════════════════════════════════════════"

if [[ "$DRY_RUN" == "true" ]]; then
    echo -e "${YELLOW}Dry run complete. Run without --dry-run to apply changes.${NC}"
else
    echo -e "${GREEN}✓ Credentials synced from 1Password to OpenBao${NC}"
    echo ""
    echo "Next steps if you updated Cloudflare credentials:"
    echo "  kubectl rollout restart deployment/cloudflare-operator-controller-manager -n cloudflare-operator-system"
fi











