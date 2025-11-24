#!/bin/bash
# Script to create GitHub Apps for new agents using the Manifest API
# Usage: ./scripts/create-agent-github-app.sh <agent-name> <role-description>
# Example: ./scripts/create-agent-github-app.sh atlas "Integration & Merge Specialist"

set -euo pipefail

AGENT_NAME="${1:-}"
ROLE_DESC="${2:-}"
ORG="${GITHUB_ORG:-5dlabs}"

if [ -z "$AGENT_NAME" ]; then
  echo "❌ Error: Agent name is required"
  echo "Usage: $0 <agent-name> <role-description>"
  echo "Example: $0 atlas 'Integration & Merge Specialist'"
  exit 1
fi

if [ -z "$ROLE_DESC" ]; then
  echo "❌ Error: Role description is required"
  echo "Usage: $0 <agent-name> <role-description>"
  exit 1
fi

# Normalize agent name (lowercase for internal use, capitalized for GitHub)
AGENT_LOWER=$(echo "$AGENT_NAME" | tr '[:upper:]' '[:lower:]')
AGENT_DISPLAY=$(echo "$AGENT_NAME" | awk '{print toupper(substr($0,1,1)) tolower(substr($0,2))}')
GITHUB_APP_NAME="5DLabs-${AGENT_DISPLAY}"

echo "════════════════════════════════════════════════════════════════"
echo "║        GitHub App Creation for Agent: $AGENT_DISPLAY"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "📋 Configuration:"
echo "   Agent Name: $AGENT_DISPLAY"
echo "   GitHub App: $GITHUB_APP_NAME"
echo "   Organization: $ORG"
echo "   Role: $ROLE_DESC"
echo ""

# Generate unique state for callback verification
STATE=$(uuidgen | tr '[:upper:]' '[:lower:]' | tr -d '-')

# Determine permissions based on agent role
if echo "$ROLE_DESC" | grep -qi "merge\|integration"; then
  # Integration agent needs write access to contents and PRs
  PERMISSIONS='{
    "contents": "write",
    "pull_requests": "write",
    "workflows": "read",
    "metadata": "read",
    "checks": "read"
  }'
  EVENTS='["pull_request", "pull_request_review", "push", "check_suite"]'
elif echo "$ROLE_DESC" | grep -qi "devops\|deploy\|infrastructure"; then
  # DevOps agent needs deployment and actions access
  PERMISSIONS='{
    "contents": "write",
    "deployments": "write",
    "actions": "write",
    "metadata": "read",
    "checks": "write",
    "pull_requests": "read"
  }'
  EVENTS='["deployment", "deployment_status", "workflow_run", "check_suite", "push"]'
else
  # Default permissions for other agents
  PERMISSIONS='{
    "contents": "write",
    "pull_requests": "write",
    "issues": "write",
    "metadata": "read"
  }'
  EVENTS='["pull_request", "pull_request_review", "issues", "issue_comment"]'
fi

# Create manifest JSON
MANIFEST=$(cat <<EOF
{
  "name": "$GITHUB_APP_NAME",
  "url": "https://github.com/$ORG/cto",
  "description": "AI $ROLE_DESC at 5D Labs - Automated agent for multi-agent software development orchestration",
  "public": false,
  "default_permissions": $PERMISSIONS,
  "default_events": $EVENTS
}
EOF
)

echo "📄 Generated manifest:"
echo "$MANIFEST" | jq '.'
echo ""

# URL-encode the manifest
ENCODED_MANIFEST=$(echo "$MANIFEST" | jq -c '.' | python3 -c "import sys; from urllib.parse import quote; print(quote(sys.stdin.read()))")

# Create the GitHub App creation URL
CREATION_URL="https://github.com/organizations/$ORG/settings/apps/new?state=$STATE"

echo "════════════════════════════════════════════════════════════════"
echo "║                  STEP 1: Create GitHub App                    ║"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "🌐 Opening GitHub App creation page..."
echo ""
echo "📝 Manual Steps Required:"
echo "   1. GitHub will open in your browser"
echo "   2. Fill in the app details using the manifest above"
echo "   3. Click 'Create GitHub App'"
echo "   4. After creation, you'll see the App ID and Client ID"
echo "   5. Generate a private key"
echo ""
echo "Press ENTER to open GitHub in your browser..."
read -r

# Open URL in browser
if command -v open >/dev/null 2>&1; then
  open "$CREATION_URL"
elif command -v xdg-open >/dev/null 2>&1; then
  xdg-open "$CREATION_URL"
else
  echo "🔗 Please open this URL manually:"
  echo "$CREATION_URL"
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "║              STEP 2: Gather App Credentials                   ║"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "After creating the app, please provide the following information:"
echo ""

# Prompt for App ID
read -p "📋 App ID (found on app settings page): " APP_ID
if [ -z "$APP_ID" ]; then
  echo "❌ App ID is required"
  exit 1
fi

# Prompt for Client ID  
read -p "📋 Client ID (found on app settings page): " CLIENT_ID
if [ -z "$CLIENT_ID" ]; then
  echo "❌ Client ID is required"
  exit 1
fi

echo ""
echo "🔑 Now you need to generate a private key:"
echo "   1. Scroll down to 'Private keys' section on the app page"
echo "   2. Click 'Generate a private key'"
echo "   3. Download the .pem file"
echo ""
read -p "📁 Path to downloaded private key (.pem file): " PRIVATE_KEY_PATH

if [ ! -f "$PRIVATE_KEY_PATH" ]; then
  echo "❌ Private key file not found: $PRIVATE_KEY_PATH"
  exit 1
fi

# Read the private key
PRIVATE_KEY=$(cat "$PRIVATE_KEY_PATH")

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "║           STEP 3: Store Credentials in Vault                  ║"
echo "════════════════════════════════════════════════════════════════"
echo ""

# Check if vault CLI is available
if ! command -v vault >/dev/null 2>&1; then
  echo "⚠️  Vault CLI not found - credentials will be saved to local file"
  echo "   You'll need to manually add them to Vault"
  
  # Create credentials file
  CREDS_FILE="./agent-credentials-${AGENT_LOWER}.json"
  cat > "$CREDS_FILE" <<EOF
{
  "agent": "$AGENT_LOWER",
  "github_app": "$GITHUB_APP_NAME",
  "app_id": "$APP_ID",
  "client_id": "$CLIENT_ID",
  "private_key": $(echo "$PRIVATE_KEY" | jq -Rs .)
}
EOF
  
  echo "✅ Credentials saved to: $CREDS_FILE"
  echo ""
  echo "📝 Manual Vault Setup Required:"
  echo "   vault kv put secret/github-app-$AGENT_LOWER \\"
  echo "     app_id=\"$APP_ID\" \\"
  echo "     client_id=\"$CLIENT_ID\" \\"
  echo "     private_key=@$PRIVATE_KEY_PATH"
  
else
  # Store in Vault
  echo "🔐 Storing credentials in Vault..."
  
  vault kv put "secret/github-app-$AGENT_LOWER" \
    app_id="$APP_ID" \
    client_id="$CLIENT_ID" \
    private_key="$PRIVATE_KEY" || {
    echo "❌ Failed to store in Vault"
    echo "   Saving to local file instead..."
    CREDS_FILE="./agent-credentials-${AGENT_LOWER}.json"
    cat > "$CREDS_FILE" <<EOF
{
  "agent": "$AGENT_LOWER",
  "github_app": "$GITHUB_APP_NAME",
  "app_id": "$APP_ID",
  "client_id": "$CLIENT_ID",
  "private_key": $(echo "$PRIVATE_KEY" | jq -Rs .)
}
EOF
    echo "✅ Credentials saved to: $CREDS_FILE"
  }
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "║         STEP 4: Update Configuration Files                    ║"
echo "════════════════════════════════════════════════════════════════"
echo ""

# Create values.yaml snippet
VALUES_SNIPPET=$(cat <<EOF

  $AGENT_LOWER:
    name: "$AGENT_DISPLAY"
    githubApp: "$GITHUB_APP_NAME"
    cli: "Claude"
    model: "claude-sonnet-4-20250514"
    maxTokens: 4096
    temperature: 0.5
    appId: "$APP_ID"
    clientId: "$CLIENT_ID"
    role: "$ROLE_DESC"
    expertise: []  # TODO: Add specific expertise areas
    description: "AI $ROLE_DESC at 5D Labs"
    systemPrompt: |
      # TODO: Add system prompt for $AGENT_DISPLAY
    tools:
      remote: []  # TODO: Add required MCP tools
EOF
)

echo "📝 Add this to infra/charts/controller/values.yaml under 'agents:':"
echo "$VALUES_SNIPPET"
echo ""

# Create ExternalSecret YAML for secret-store namespace
EXTERNAL_SECRET_STORE=$(cat <<EOF
---
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: github-app-5dlabs-$AGENT_LOWER
  namespace: secret-store
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: secret-store
    kind: ClusterSecretStore
  target:
    name: github-app-5dlabs-$AGENT_LOWER
    creationPolicy: Owner
  data:
  - secretKey: GITHUB_APP_ID
    remoteRef:
      key: github-app-$AGENT_LOWER
      property: app_id
  - secretKey: GITHUB_APP_PRIVATE_KEY
    remoteRef:
      key: github-app-$AGENT_LOWER
      property: private_key
  - secretKey: GITHUB_APP_CLIENT_ID
    remoteRef:
      key: github-app-$AGENT_LOWER
      property: client_id
EOF
)

# Create ExternalSecret YAML for agent-platform namespace  
EXTERNAL_SECRET_PLATFORM=$(cat <<EOF

---
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: github-app-5dlabs-$AGENT_LOWER-agent-platform
  namespace: agent-platform
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: secret-store
    kind: ClusterSecretStore
  target:
    name: github-app-5dlabs-$AGENT_LOWER
    creationPolicy: Owner
  data:
  - secretKey: app-id
    remoteRef:
      key: github-app-$AGENT_LOWER
      property: app_id
  - secretKey: private-key
    remoteRef:
      key: github-app-$AGENT_LOWER
      property: private_key
  - secretKey: client-id
    remoteRef:
      key: github-app-$AGENT_LOWER
      property: client_id
EOF
)

# Create alias ExternalSecret
EXTERNAL_SECRET_ALIAS=$(cat <<EOF

---
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: github-app-$AGENT_LOWER
  namespace: agent-platform
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: secret-store
    kind: ClusterSecretStore
  target:
    name: github-app-$AGENT_LOWER
    creationPolicy: Owner
  data:
  - secretKey: app-id
    remoteRef:
      key: github-app-$AGENT_LOWER
      property: app_id
  - secretKey: private-key
    remoteRef:
      key: github-app-$AGENT_LOWER
      property: private_key
  - secretKey: client-id
    remoteRef:
      key: github-app-$AGENT_LOWER
      property: client_id
EOF
)

echo "📝 Add these to infra/secret-store/agent-secrets-external-secrets.yaml:"
echo "$EXTERNAL_SECRET_STORE"
echo "$EXTERNAL_SECRET_PLATFORM"
echo "$EXTERNAL_SECRET_ALIAS"
echo ""

# Save to temporary file for easy copying
TEMP_CONFIG="/tmp/agent-${AGENT_LOWER}-config.yaml"
cat > "$TEMP_CONFIG" <<EOF
# Configuration snippets for $GITHUB_APP_NAME

# ========================================
# Add to: infra/charts/controller/values.yaml
# Location: Under 'agents:' section
# ========================================
$VALUES_SNIPPET

# ========================================
# Add to: infra/secret-store/agent-secrets-external-secrets.yaml  
# Location: At the end of the file
# ========================================
$EXTERNAL_SECRET_STORE
$EXTERNAL_SECRET_PLATFORM
$EXTERNAL_SECRET_ALIAS
EOF

echo "✅ Configuration saved to: $TEMP_CONFIG"
echo ""

echo "════════════════════════════════════════════════════════════════"
echo "║                    SETUP COMPLETE                             ║"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "✅ GitHub App created: $GITHUB_APP_NAME"
echo "✅ Credentials collected and ready"
echo ""
echo "📋 Next Steps:"
echo "   1. Review configuration in: $TEMP_CONFIG"
echo "   2. Add values.yaml snippet to Helm chart"
echo "   3. Add ExternalSecrets to secret-store config"
echo "   4. Store credentials in Vault (if not done automatically)"
echo "   5. Install the GitHub App to your repositories"
echo ""
echo "🔗 GitHub App URL:"
echo "   https://github.com/organizations/$ORG/settings/apps/$GITHUB_APP_NAME"
echo ""
echo "════════════════════════════════════════════════════════════════"





