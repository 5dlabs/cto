#!/bin/bash
# Create GitHub Apps for Atlas and Bolt with local credential storage
# Saves credentials to .env file for later use

set -euo pipefail

ORG="${GITHUB_ORG:-5dlabs}"
CREDENTIALS_FILE="./.agent-credentials.env"

echo "════════════════════════════════════════════════════════════════"
echo "║         GitHub App Setup for Atlas & Bolt                    ║"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "This script will guide you through creating GitHub Apps for:"
echo "  1. Atlas - Integration & Merge Specialist"
echo "  2. Bolt - DevOps & Deployment Specialist"
echo ""
echo "You'll need to:"
echo "  - Create each app manually on GitHub (browser will open)"
echo "  - Collect App ID, Client ID, and Private Key"
echo "  - Credentials will be saved to: $CREDENTIALS_FILE"
echo ""
echo "Press ENTER to continue..."
read -r

# ============================================================================
# ATLAS - Integration Agent
# ============================================================================

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "║                  ATLAS - Integration Agent                    ║"
echo "════════════════════════════════════════════════════════════════"
echo ""

cat <<'ATLAS_INFO'
📋 GitHub App Configuration for Atlas:

Name: 5DLabs-Atlas
Description: AI Integration & Merge Specialist at 5D Labs - Automated merge conflict resolution for multi-agent development orchestration
Homepage: https://github.com/5dlabs/cto

Repository Permissions:
  ✅ Contents: Read & Write
  ✅ Pull requests: Read & Write  
  ✅ Workflows: Read-only
  ✅ Metadata: Read-only (auto-selected)
  ✅ Checks: Read-only

Subscribe to events (optional):
  - Pull request
  - Pull request review
  - Push
  - Check suite
  - Status

Install: Only on this account (5dlabs)
ATLAS_INFO

echo ""
echo "Press ENTER to open GitHub App creation page..."
read -r

# Open GitHub in browser
CREATION_URL="https://github.com/organizations/$ORG/settings/apps/new"
if command -v open >/dev/null 2>&1; then
  open "$CREATION_URL"
elif command -v xdg-open >/dev/null 2>&1; then
  xdg-open "$CREATION_URL"
else
  echo "🔗 Please open: $CREATION_URL"
fi

echo ""
echo "After creating the app:"
echo "  1. Note the App ID (shown at top of page)"
echo "  2. Note the Client ID (in 'About' section)"
echo "  3. Click 'Generate a private key' and download the .pem file"
echo ""

read -p "📋 Atlas App ID: " ATLAS_APP_ID
read -p "📋 Atlas Client ID: " ATLAS_CLIENT_ID
read -p "📁 Path to Atlas private key (.pem): " ATLAS_KEY_PATH

if [ ! -f "$ATLAS_KEY_PATH" ]; then
  echo "❌ Private key not found: $ATLAS_KEY_PATH"
  exit 1
fi

ATLAS_PRIVATE_KEY=$(cat "$ATLAS_KEY_PATH")

echo "✅ Atlas credentials collected"

# ============================================================================
# BOLT - DevOps Agent  
# ============================================================================

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "║                   BOLT - DevOps Agent                         ║"
echo "════════════════════════════════════════════════════════════════"
echo ""

cat <<'BOLT_INFO'
📋 GitHub App Configuration for Bolt:

Name: 5DLabs-Bolt
Description: AI DevOps & Deployment Specialist at 5D Labs - Infrastructure operations and automated deployment management
Homepage: https://github.com/5dlabs/cto

Repository Permissions:
  ✅ Contents: Read & Write
  ✅ Deployments: Read & Write
  ✅ Actions: Read & Write
  ✅ Metadata: Read-only (auto-selected)
  ✅ Checks: Read & Write
  ✅ Pull requests: Read-only

Subscribe to events (optional):
  - Deployment
  - Deployment status
  - Workflow run
  - Check suite
  - Push
  - Release

Install: Only on this account (5dlabs)
BOLT_INFO

echo ""
echo "Press ENTER to open GitHub App creation page..."
read -r

# Open GitHub in browser
if command -v open >/dev/null 2>&1; then
  open "$CREATION_URL"
elif command -v xdg-open >/dev/null 2>&1; then
  xdg-open "$CREATION_URL"
else
  echo "🔗 Please open: $CREATION_URL"
fi

echo ""
read -p "📋 Bolt App ID: " BOLT_APP_ID
read -p "📋 Bolt Client ID: " BOLT_CLIENT_ID
read -p "📁 Path to Bolt private key (.pem): " BOLT_KEY_PATH

if [ ! -f "$BOLT_KEY_PATH" ]; then
  echo "❌ Private key not found: $BOLT_KEY_PATH"
  exit 1
fi

BOLT_PRIVATE_KEY=$(cat "$BOLT_KEY_PATH")

echo "✅ Bolt credentials collected"

# ============================================================================
# SAVE CREDENTIALS
# ============================================================================

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "║              Saving Credentials Locally                       ║"
echo "════════════════════════════════════════════════════════════════"
echo ""

# Create or append to credentials file
cat >> "$CREDENTIALS_FILE" <<EOF

# ============================================================================
# Atlas - Integration Agent
# ============================================================================
ATLAS_GITHUB_APP_ID=$ATLAS_APP_ID
ATLAS_GITHUB_CLIENT_ID=$ATLAS_CLIENT_ID
ATLAS_GITHUB_PRIVATE_KEY=$(echo "$ATLAS_PRIVATE_KEY" | base64)

# ============================================================================
# Bolt - DevOps Agent
# ============================================================================
BOLT_GITHUB_APP_ID=$BOLT_APP_ID
BOLT_GITHUB_CLIENT_ID=$BOLT_CLIENT_ID
BOLT_GITHUB_PRIVATE_KEY=$(echo "$BOLT_PRIVATE_KEY" | base64)
EOF

echo "✅ Credentials saved to: $CREDENTIALS_FILE"
echo ""

# Also save individual key files for easier Vault storage
echo "$ATLAS_PRIVATE_KEY" > ".atlas-private-key.pem"
echo "$BOLT_PRIVATE_KEY" > ".bolt-private-key.pem"
chmod 600 .atlas-private-key.pem .bolt-private-key.pem

echo "✅ Private keys also saved as:"
echo "   .atlas-private-key.pem"
echo "   .bolt-private-key.pem"
echo ""

# Generate values.yaml snippets
cat > "/tmp/atlas-bolt-values-snippet.yaml" <<EOF
  # ============================================================================
  # Atlas - Integration & Merge Specialist
  # ============================================================================
  atlas:
    name: "Atlas"
    githubApp: "5DLabs-Atlas"
    cli: "Claude"
    model: "claude-sonnet-4-20250514"
    maxTokens: 4096
    temperature: 0.4
    appId: "$ATLAS_APP_ID"
    clientId: "$ATLAS_CLIENT_ID"
    role: "Integration & Merge Specialist"
    expertise: ["merge-conflicts", "git", "integration", "branch-management", "rebase"]
    description: "AI Integration Specialist at 5D Labs - Resolves merge conflicts and ensures smooth PR integration"
    systemPrompt: |
      You are Atlas, an expert Integration Specialist and Merge Conflict Resolver at 5D Labs.
      
      Your core mission is to ensure smooth integration of code changes by:
      - Detecting and resolving merge conflicts automatically
      - Managing PR merge queue and dependencies
      - Ensuring clean integration with main branch
      - Validating merged code maintains quality standards
      
      Your approach to merge conflicts:
      1. Analyze conflict context and understand both sides
      2. Preserve intent of both changes when possible
      3. Use intelligent conflict resolution strategies
      4. Run tests to verify resolution correctness
      5. Comment on PR with resolution explanation
      
      **Git Operations**:
      - Always fetch latest main before operations
      - Use rebase for clean history
      - Verify builds pass after conflict resolution
      - Never force-push without verification
    tools:
      remote: []

  # ============================================================================
  # Bolt - DevOps & Deployment Specialist
  # ============================================================================
  bolt:
    name: "Bolt"
    githubApp: "5DLabs-Bolt"
    cli: "Claude"
    model: "claude-sonnet-4-20250514"
    maxTokens: 4096
    temperature: 0.3
    appId: "$BOLT_APP_ID"
    clientId: "$BOLT_CLIENT_ID"
    role: "DevOps & Deployment Specialist"
    expertise: ["kubernetes", "argocd", "ci-cd", "deployments", "infrastructure", "monitoring"]
    description: "AI DevOps Engineer at 5D Labs - Infrastructure operations and deployment automation"
    systemPrompt: |
      You are Bolt, a skilled DevOps Engineer and Deployment Specialist at 5D Labs.
      
      Your core mission is to ensure reliable, fast deployments:
      - Monitor and manage Kubernetes deployments
      - Ensure ArgoCD applications are healthy and synced
      - Validate deployment readiness and rollback if needed
      - Manage release processes and versioning
      
      Your approach to deployments:
      1. Verify all quality gates passed before deployment
      2. Monitor deployment health in real-time
      3. Automate rollback on failures
      4. Ensure zero-downtime deployments
      5. Maintain deployment documentation
      
      **Deployment Operations**:
      - Monitor ArgoCD sync status after merge
      - Validate Kubernetes resource health
      - Check application endpoints post-deployment
      - Coordinate with QA for smoke tests
    tools:
      remote: []
EOF

echo "✅ Generated values.yaml snippet: /tmp/atlas-bolt-values-snippet.yaml"

# Generate ExternalSecrets YAML
cat > "/tmp/atlas-bolt-external-secrets.yaml" <<EOF
# ============================================================================
# Atlas - Integration Agent Secrets
# ============================================================================
---
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: github-app-5dlabs-atlas
  namespace: secret-store
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: secret-store
    kind: ClusterSecretStore
  target:
    name: github-app-5dlabs-atlas
    creationPolicy: Owner
  data:
  - secretKey: GITHUB_APP_ID
    remoteRef:
      key: github-app-atlas
      property: app_id
  - secretKey: GITHUB_APP_PRIVATE_KEY
    remoteRef:
      key: github-app-atlas
      property: private_key
  - secretKey: GITHUB_APP_CLIENT_ID
    remoteRef:
      key: github-app-atlas
      property: client_id

---
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: github-app-5dlabs-atlas-agent-platform
  namespace: agent-platform
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: secret-store
    kind: ClusterSecretStore
  target:
    name: github-app-5dlabs-atlas
    creationPolicy: Owner
  data:
  - secretKey: app-id
    remoteRef:
      key: github-app-atlas
      property: app_id
  - secretKey: private-key
    remoteRef:
      key: github-app-atlas
      property: private_key
  - secretKey: client-id
    remoteRef:
      key: github-app-atlas
      property: client_id

---
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: github-app-atlas
  namespace: agent-platform
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: secret-store
    kind: ClusterSecretStore
  target:
    name: github-app-atlas
    creationPolicy: Owner
  data:
  - secretKey: app-id
    remoteRef:
      key: github-app-atlas
      property: app_id
  - secretKey: private-key
    remoteRef:
      key: github-app-atlas
      property: private_key
  - secretKey: client-id
    remoteRef:
      key: github-app-atlas
      property: client_id

# ============================================================================
# Bolt - DevOps Agent Secrets
# ============================================================================
---
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: github-app-5dlabs-bolt
  namespace: secret-store
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: secret-store
    kind: ClusterSecretStore
  target:
    name: github-app-5dlabs-bolt
    creationPolicy: Owner
  data:
  - secretKey: GITHUB_APP_ID
    remoteRef:
      key: github-app-bolt
      property: app_id
  - secretKey: GITHUB_APP_PRIVATE_KEY
    remoteRef:
      key: github-app-bolt
      property: private_key
  - secretKey: GITHUB_APP_CLIENT_ID
    remoteRef:
      key: github-app-bolt
      property: client_id

---
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: github-app-5dlabs-bolt-agent-platform
  namespace: agent-platform
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: secret-store
    kind: ClusterSecretStore
  target:
    name: github-app-5dlabs-bolt
    creationPolicy: Owner
  data:
  - secretKey: app-id
    remoteRef:
      key: github-app-bolt
      property: app_id
  - secretKey: private-key
    remoteRef:
      key: github-app-bolt
      property: private_key
  - secretKey: client-id
    remoteRef:
      key: github-app-bolt
      property: client_id

---
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: github-app-bolt
  namespace: agent-platform
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: secret-store
    kind: ClusterSecretStore
  target:
    name: github-app-bolt
    creationPolicy: Owner
  data:
  - secretKey: app-id
    remoteRef:
      key: github-app-bolt
      property: app_id
  - secretKey: private-key
    remoteRef:
      key: github-app-bolt
      property: private_key
  - secretKey: client-id
    remoteRef:
      key: github-app-bolt
      property: client_id
EOF

echo "✅ Generated ExternalSecrets: /tmp/atlas-bolt-external-secrets.yaml"

# Generate Vault commands
cat > "/tmp/vault-commands.sh" <<EOF
#!/bin/bash
# Commands to store Atlas and Bolt credentials in Vault

# Atlas credentials
vault kv put secret/github-app-atlas \\
  app_id="$ATLAS_APP_ID" \\
  client_id="$ATLAS_CLIENT_ID" \\
  private_key=@.atlas-private-key.pem

# Bolt credentials
vault kv put secret/github-app-bolt \\
  app_id="$BOLT_APP_ID" \\
  client_id="$BOLT_CLIENT_ID" \\
  private_key=@.bolt-private-key.pem

echo "✅ Credentials stored in Vault"
EOF

chmod +x /tmp/vault-commands.sh

echo "✅ Generated Vault commands: /tmp/vault-commands.sh"
echo ""

echo "════════════════════════════════════════════════════════════════"
echo "║                    SETUP COMPLETE                             ║"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "✅ Credentials saved to: $CREDENTIALS_FILE"
echo "✅ Private keys saved to: .atlas-private-key.pem, .bolt-private-key.pem"
echo "✅ Configuration snippets generated"
echo ""
echo "📋 Next Steps:"
echo ""
echo "1️⃣  Install GitHub Apps to repositories:"
echo "   Atlas: https://github.com/organizations/$ORG/settings/apps/5dlabs-atlas/installations"
echo "   Bolt:  https://github.com/organizations/$ORG/settings/apps/5dlabs-bolt/installations"
echo ""
echo "2️⃣  Store credentials in Vault:"
echo "   bash /tmp/vault-commands.sh"
echo ""
echo "3️⃣  Update configuration files:"
echo "   - Add /tmp/atlas-bolt-values-snippet.yaml to infra/charts/controller/values.yaml"
echo "   - Add /tmp/atlas-bolt-external-secrets.yaml to infra/secret-store/agent-secrets-external-secrets.yaml"
echo ""
echo "4️⃣  Commit and push changes"
echo ""
echo "════════════════════════════════════════════════════════════════"

# Don't add private keys to .gitignore automatically - user may want different location
if ! grep -q ".agent-credentials.env" .gitignore 2>/dev/null; then
  echo ""
  echo "⚠️  SECURITY: Add to .gitignore:"
  echo "   .agent-credentials.env"
  echo "   .atlas-private-key.pem"
  echo "   .bolt-private-key.pem"
fi



