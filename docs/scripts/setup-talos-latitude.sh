#!/bin/bash
# Setup Talos CLI connection to Latitude cluster
# This script helps retrieve Latitude credentials from 1Password and configure talosctl

set -euo pipefail

echo "🔧 Talos CLI Latitude Setup"
echo ""

# Check if 1Password CLI is available
if ! command -v op &> /dev/null; then
    echo "❌ 1Password CLI (op) not found."
    echo "   Install it with: brew install --cask 1password-cli"
    echo "   Or visit: https://developer.1password.com/docs/cli/get-started"
    exit 1
fi

# Check if signed in
if ! op account list &> /dev/null; then
    echo "📝 Signing in to 1Password..."
    op signin
fi

echo "🔑 Retrieving Latitude credentials from 1Password..."
echo ""

# Default vault and item (can be overridden with env vars)
VAULT="${OP_VAULT:-Personal}"
ITEM="${OP_LATITUDE_ITEM:-Latitude.sh API}"

# Get API key (stored in 'credential' field)
echo "   Fetching API key..."
API_KEY=$(op item get "${ITEM}" --vault "${VAULT}" --fields credential --reveal 2>/dev/null | tail -1 || echo "")

if [ -z "$API_KEY" ]; then
    echo "❌ Failed to retrieve Latitude API key from 1Password"
    echo "   Expected: Item '${ITEM}' in vault '${VAULT}' with field 'credential'"
    echo ""
    echo "   To check available items:"
    echo "   op item list --vault ${VAULT} | grep -i latitude"
    exit 1
fi

# Get project ID (stored in 'Project ID' field)
echo "   Fetching project ID..."
PROJECT_ID=$(op item get "${ITEM}" --vault "${VAULT}" --fields "Project ID" --reveal 2>/dev/null | tail -1 || echo "")

if [ -z "$PROJECT_ID" ]; then
    echo "❌ Failed to retrieve Latitude project ID from 1Password"
    echo "   Expected: Item '${ITEM}' in vault '${VAULT}' with field 'Project ID'"
    exit 1
fi

echo "✅ Credentials retrieved successfully"
echo ""
echo "📋 Latitude Configuration:"
echo "   API Key: ${API_KEY:0:10}... (hidden)"
echo "   Project ID: ${PROJECT_ID}"
echo ""

# Export for use
export LATITUDE_API_KEY="$API_KEY"
export LATITUDE_PROJECT_ID="$PROJECT_ID"

echo "💡 Next steps:"
echo ""
echo "1. Find your Talos cluster endpoints (node IPs):"
echo "   - Check Latitude.sh dashboard for server IPs"
echo "   - Or use: curl -H \"Authorization: Bearer \$LATITUDE_API_KEY\" \\"
echo "              https://api.latitude.sh/servers"
echo ""
echo "2. Configure talosctl for your cluster:"
echo "   talosctl config endpoint <control-plane-ip>"
echo ""
echo "3. Or use an existing talosconfig:"
echo "   talosctl --talosconfig=<path-to-talosconfig> -n <node-ip> version"
echo ""
echo "4. Get kubeconfig:"
echo "   talosctl --talosconfig=<path> kubeconfig -n <control-plane-ip>"
echo ""

