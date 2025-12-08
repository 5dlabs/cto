#!/bin/bash
# Comprehensive agent setup script
# Creates GitHub App, configures secrets, and updates all necessary files
# Usage: ./scripts/setup-new-agent.sh <agent-name> <role-description> <avatar-url>

set -euo pipefail

AGENT_NAME="${1:-}"
ROLE_DESC="${2:-}"
AVATAR_URL="${3:-}"
ORG="${GITHUB_ORG:-5dlabs}"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_header() {
  echo ""
  echo "════════════════════════════════════════════════════════════════"
  echo "║ $1"
  echo "════════════════════════════════════════════════════════════════"
  echo ""
}

print_success() {
  echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
  echo -e "${RED}❌ $1${NC}"
}

print_warning() {
  echo -e "${YELLOW}⚠️  $1${NC}"
}

print_info() {
  echo -e "${BLUE}ℹ️  $1${NC}"
}

if [ -z "$AGENT_NAME" ]; then
  print_error "Agent name is required"
  echo "Usage: $0 <agent-name> <role-description> <avatar-url>"
  echo "Example: $0 atlas 'Integration & Merge Specialist' 'https://raw.githubusercontent.com/5dlabs/.github/main/profile/assets/atlas-avatar.png'"
  exit 1
fi

if [ -z "$ROLE_DESC" ]; then
  print_error "Role description is required"
  exit 1
fi

# Normalize names
AGENT_LOWER=$(echo "$AGENT_NAME" | tr '[:upper:]' '[:lower:]')
AGENT_DISPLAY=$(echo "$AGENT_NAME" | awk '{print toupper(substr($0,1,1)) tolower(substr($0,2))}')
GITHUB_APP_NAME="5DLabs-${AGENT_DISPLAY}"

print_header "AGENT SETUP: $AGENT_DISPLAY"

print_info "Agent Name: $AGENT_DISPLAY"
print_info "GitHub App: $GITHUB_APP_NAME"
print_info "Organization: $ORG"
print_info "Role: $ROLE_DESC"
if [ -n "$AVATAR_URL" ]; then
  print_info "Avatar: $AVATAR_URL"
fi

# Determine permissions and events based on role
if echo "$ROLE_DESC" | grep -qi "merge\|integration"; then
  PERMISSIONS='{"contents":"write","pull_requests":"write","workflows":"read","metadata":"read","checks":"read"}'
  EVENTS='["pull_request","pull_request_review","push","check_suite","status"]'
  EXPERTISE='["merge-conflicts","git","integration","branch-management","rebase","conflict-resolution"]'
  SYSTEM_PROMPT="You are $AGENT_DISPLAY, an expert Integration Specialist and Merge Conflict Resolver at 5D Labs.

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

Your personality is systematic, reliable, and solution-oriented. Most conflicts
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
- PR has all required approvals from upstream agents"

elif echo "$ROLE_DESC" | grep -qi "devops\|deploy\|infrastructure"; then
  PERMISSIONS='{"contents":"write","deployments":"write","actions":"write","metadata":"read","checks":"write","pull_requests":"read"}'
  EVENTS='["deployment","deployment_status","workflow_run","check_suite","push","release"]'
  EXPERTISE='["kubernetes","argocd","ci-cd","deployments","infrastructure","monitoring","sre"]'
  SYSTEM_PROMPT="You are $AGENT_DISPLAY, a skilled DevOps Engineer and Deployment Specialist at 5D Labs.

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
- Monitor ArgoCD sync status after merge
- Validate Kubernetes resource health
- Check application endpoints post-deployment
- Coordinate with QA for smoke tests
- Document any deployment issues or rollbacks

**Infrastructure Quality Gates**:
- All Helm charts must validate
- Kubernetes manifests must apply cleanly
- Health checks must pass
- Resource limits properly configured
- Monitoring alerts configured for new services"

else
  PERMISSIONS='{"contents":"write","pull_requests":"write","issues":"write","metadata":"read"}'
  EVENTS='["pull_request","pull_request_review","issues","issue_comment"]'
  EXPERTISE='[]'
  SYSTEM_PROMPT="You are $AGENT_DISPLAY, an AI agent at 5D Labs specialized in $ROLE_DESC."
fi

# Generate manifest
MANIFEST=$(cat <<EOF
{
  "name": "$GITHUB_APP_NAME",
  "url": "https://github.com/$ORG/cto",
  "description": "AI $ROLE_DESC at 5D Labs - Multi-agent development orchestration",
  "public": false,
  "default_permissions": $PERMISSIONS,
  "default_events": $EVENTS
}
EOF
)

print_header "STEP 1: Create GitHub App"

print_info "Opening GitHub App creation page..."
print_warning "Manual steps required:"
echo "   1. Review and customize the app settings"
echo "   2. Click 'Create GitHub App'"  
echo "   3. Generate a private key"
echo "   4. Install the app to your repositories"

# Create URL (manifest can be pre-filled via URL but still requires manual confirmation)
CREATION_URL="https://github.com/organizations/$ORG/settings/apps/new"

echo ""
echo "Press ENTER to open GitHub..."
read -r

# Open browser
if command -v open >/dev/null 2>&1; then
  open "$CREATION_URL"
elif command -v xdg-open >/dev/null 2>&1; then
  xdg-open "$CREATION_URL"
else
  print_warning "Please open manually: $CREATION_URL"
fi

print_header "STEP 2: Collect App Credentials"

# Collect credentials
read -p "App ID: " APP_ID
read -p "Client ID: " CLIENT_ID
read -p "Path to private key (.pem file): " PRIVATE_KEY_PATH

if [ ! -f "$PRIVATE_KEY_PATH" ]; then
  print_error "Private key file not found: $PRIVATE_KEY_PATH"
  exit 1
fi

PRIVATE_KEY=$(cat "$PRIVATE_KEY_PATH")

print_header "STEP 3: Update Configuration Files"

# Update values.yaml
print_info "Updating infra/charts/controller/values.yaml..."

# Escape the system prompt for YAML
SYSTEM_PROMPT_ESCAPED=$(echo "$SYSTEM_PROMPT" | sed 's/^/      /')

# Add agent to values.yaml (append before the closing of agents section)
# This is a template - user should review and adjust
cat > "/tmp/agent-${AGENT_LOWER}-values-snippet.yaml" <<EOF
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
    expertise: $EXPERTISE
    description: "AI $ROLE_DESC at 5D Labs"
    systemPrompt: |
$SYSTEM_PROMPT_ESCAPED
    tools:
      remote: []
EOF

print_success "Values snippet saved to: /tmp/agent-${AGENT_LOWER}-values-snippet.yaml"

# Update external secrets
print_info "Creating ExternalSecret configurations..."

cat > "/tmp/agent-${AGENT_LOWER}-external-secrets.yaml" <<EOF
$EXTERNAL_SECRET_STORE
$EXTERNAL_SECRET_PLATFORM
$EXTERNAL_SECRET_ALIAS
EOF

print_success "ExternalSecrets saved to: /tmp/agent-${AGENT_LOWER}-external-secrets.yaml"

print_header "STEP 4: Store Credentials in Vault"

if command -v vault >/dev/null 2>&1; then
  print_info "Storing in Vault..."
  
  vault kv put "secret/github-app-$AGENT_LOWER" \
    app_id="$APP_ID" \
    client_id="$CLIENT_ID" \
    private_key="$PRIVATE_KEY"
  
  print_success "Credentials stored in Vault"
else
  print_warning "Vault CLI not available"
  echo ""
  echo "Manual Vault command:"
  echo "  vault kv put secret/github-app-$AGENT_LOWER \\"
  echo "    app_id='$APP_ID' \\"
  echo "    client_id='$CLIENT_ID' \\"
  echo "    private_key=@$PRIVATE_KEY_PATH"
fi

print_header "SETUP SUMMARY"

print_success "GitHub App: $GITHUB_APP_NAME (ID: $APP_ID)"
print_success "Configuration files generated"

echo ""
echo "📋 Next Steps:"
echo ""
echo "1️⃣  Add to infra/charts/controller/values.yaml:"
echo "    cat /tmp/agent-${AGENT_LOWER}-values-snippet.yaml"
echo ""
echo "2️⃣  Add to infra/vault/secrets/github-apps.yaml:"
echo "    cat /tmp/agent-${AGENT_LOWER}-vault-static-secrets.yaml"
echo ""
echo "3️⃣  Install GitHub App to repositories:"
echo "    https://github.com/organizations/$ORG/settings/apps/$GITHUB_APP_NAME/installations"
echo ""
echo "4️⃣  Commit and push changes:"
echo "    git add infra/charts/controller/values.yaml"
echo "    git add infra/vault/secrets/github-apps.yaml"
echo "    git commit -m 'feat: add $AGENT_DISPLAY agent ($GITHUB_APP_NAME)'"
echo ""

print_success "Agent $AGENT_DISPLAY is ready to integrate!"





