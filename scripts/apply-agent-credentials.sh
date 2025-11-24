#!/bin/bash
# Apply Atlas and Bolt credentials to configuration files
# Reads from .agent-credentials.env and updates values.yaml

set -euo pipefail

CREDS_FILE=".agent-credentials.env"

if [ ! -f "$CREDS_FILE" ]; then
  echo "❌ Credentials file not found: $CREDS_FILE"
  echo "   Copy .agent-setup-template.env to $CREDS_FILE and fill in values"
  exit 1
fi

# Source credentials
source "$CREDS_FILE"

# Validate required variables
if [ -z "${ATLAS_GITHUB_APP_ID:-}" ] || [ -z "${ATLAS_GITHUB_CLIENT_ID:-}" ]; then
  echo "❌ Atlas credentials incomplete in $CREDS_FILE"
  exit 1
fi

if [ -z "${BOLT_GITHUB_APP_ID:-}" ] || [ -z "${BOLT_GITHUB_CLIENT_ID:-}" ]; then
  echo "❌ Bolt credentials incomplete in $CREDS_FILE"
  exit 1
fi

# Check private keys exist
if [ ! -f ".atlas-private-key.pem" ]; then
  echo "❌ Atlas private key not found: .atlas-private-key.pem"
  exit 1
fi

if [ ! -f ".bolt-private-key.pem" ]; then
  echo "❌ Bolt private key not found: .bolt-private-key.pem"
  exit 1
fi

echo "════════════════════════════════════════════════════════════════"
echo "║         Applying Atlas & Bolt Credentials                    ║"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "✅ Credentials loaded from: $CREDS_FILE"
echo "✅ Atlas App ID: $ATLAS_GITHUB_APP_ID"
echo "✅ Bolt App ID: $BOLT_GITHUB_APP_ID"
echo ""

# Generate values.yaml snippet
cat > "/tmp/atlas-bolt-values-complete.yaml" <<EOF
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
    appId: "$ATLAS_GITHUB_APP_ID"
    clientId: "$ATLAS_GITHUB_CLIENT_ID"
    role: "Integration & Merge Specialist"
    expertise: ["merge-conflicts", "git", "integration", "branch-management", "rebase", "conflict-resolution"]
    description: "AI Integration Specialist at 5D Labs - Resolves merge conflicts and ensures smooth PR integration"
    systemPrompt: |
      You are Atlas, an expert Integration Specialist and Merge Conflict Resolver at 5D Labs.
      
      Your core mission is to ensure smooth integration of code changes by:
      - Detecting and resolving merge conflicts automatically
      - Managing PR merge queue and dependencies
      - Ensuring clean integration with main branch
      - Validating merged code maintains quality standards
      - Coordinating with other agents when manual intervention needed
      
      Your approach to merge conflicts:
      1. Analyze conflict context and understand both sides
      2. Preserve intent of both changes when possible
      3. Use intelligent conflict resolution strategies
      4. Run tests to verify resolution correctness
      5. Comment on PR with resolution explanation
      
      Your personality is systematic, reliable, and solution-oriented. You believe most conflicts
      can be resolved automatically with proper context understanding.
      
      **Git Operations**:
      - Always fetch latest main before operations
      - Use rebase for clean history
      - Verify builds pass after conflict resolution
      - Never force-push without verification
      
      **Integration Quality Gates**:
      - All tests must pass after merge
      - Linting and formatting must be clean
      - No new security vulnerabilities introduced
      - PR has all required approvals from upstream agents (Cleo, Cipher, Tess)
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
    appId: "$BOLT_GITHUB_APP_ID"
    clientId: "$BOLT_GITHUB_CLIENT_ID"
    role: "DevOps & Deployment Specialist"
    expertise: ["kubernetes", "argocd", "ci-cd", "deployments", "infrastructure", "monitoring", "sre"]
    description: "AI DevOps Engineer at 5D Labs - Infrastructure operations and deployment automation"
    systemPrompt: |
      You are Bolt, a skilled DevOps Engineer and Deployment Specialist at 5D Labs.
      
      Your core mission is to ensure reliable, fast deployments and maintain infrastructure health:
      - Monitor and manage Kubernetes deployments
      - Ensure ArgoCD applications are healthy and synced
      - Validate deployment readiness and rollback if needed
      - Manage release processes and versioning
      - Coordinate infrastructure changes with development workflows
      
      Your approach to deployments:
      1. Verify all quality gates passed before deployment
      2. Monitor deployment health in real-time
      3. Automate rollback on failures
      4. Ensure zero-downtime deployments
      5. Maintain deployment documentation and runbooks
      
      Your personality is action-oriented, reliable, and ops-focused. Speed matters,
      but correctness matters more. You believe in automation but verify everything.
      
      **Deployment Operations**:
      - Monitor ArgoCD sync status after merge to main
      - Validate Kubernetes resource health (pods, services, ingresses)
      - Check application endpoints post-deployment
      - Coordinate with QA for smoke tests
      - Document any deployment issues or rollbacks
      
      **Infrastructure Quality Gates**:
      - All Helm charts must validate
      - Kubernetes manifests must apply cleanly
      - Health checks must pass
      - Resource limits properly configured
      - Monitoring alerts configured for new services
    tools:
      remote: []
EOF

echo "✅ Generated complete values.yaml snippet: /tmp/atlas-bolt-values-complete.yaml"

# Generate Vault commands
cat > "/tmp/vault-store-credentials.sh" <<'EOF'
#!/bin/bash
# Store Atlas and Bolt credentials in Vault

set -euo pipefail

if ! command -v vault >/dev/null 2>&1; then
  echo "❌ Vault CLI not found"
  echo "Install: brew install vault"
  exit 1
fi

# Load credentials
if [ ! -f ".agent-credentials.env" ]; then
  echo "❌ .agent-credentials.env not found"
  exit 1
fi

source .agent-credentials.env

echo "Storing Atlas credentials in Vault..."
vault kv put secret/github-app-atlas \
  app_id="$ATLAS_GITHUB_APP_ID" \
  client_id="$ATLAS_GITHUB_CLIENT_ID" \
  private_key=@.atlas-private-key.pem

echo "Storing Bolt credentials in Vault..."
vault kv put secret/github-app-bolt \
  app_id="$BOLT_GITHUB_APP_ID" \
  client_id="$BOLT_GITHUB_CLIENT_ID" \
  private_key=@.bolt-private-key.pem

echo "✅ Credentials stored in Vault successfully"
EOF

chmod +x /tmp/vault-store-credentials.sh

echo "✅ Generated Vault storage script: /tmp/vault-store-credentials.sh"

# Generate ExternalSecrets YAML
cat > "/tmp/atlas-bolt-external-secrets-complete.yaml" <<EOF
# ============================================================================
# Atlas - Integration Agent ExternalSecrets
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
# Bolt - DevOps Agent ExternalSecrets
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

echo "✅ Generated complete ExternalSecrets: /tmp/atlas-bolt-external-secrets-complete.yaml"

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "║                    CONFIGURATION READY                        ║"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "📋 Next steps:"
echo "   1. Create GitHub Apps and fill in .agent-setup-template.env"
echo "   2. Rename to .agent-credentials.env"
echo "   3. Add snippets to configuration files"
echo "   4. Store in Vault: bash /tmp/vault-store-credentials.sh"
echo ""







