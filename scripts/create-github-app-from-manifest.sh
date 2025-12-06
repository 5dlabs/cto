#!/usr/bin/env bash
# =============================================================================
# create-github-app-from-manifest.sh - Create GitHub App with auto-credential capture
# =============================================================================
# Creates a GitHub App using the manifest flow and automatically captures
# all credentials (including private key) for storage in 1Password.
#
# Usage:
#   ./scripts/create-github-app-from-manifest.sh <agent-name> [role]
#
# Examples:
#   ./scripts/create-github-app-from-manifest.sh spark coder
#   ./scripts/create-github-app-from-manifest.sh nova coder
#   ./scripts/create-github-app-from-manifest.sh grizz coder
#
# The script will:
#   1. Generate a manifest matching Rex's permissions
#   2. Start a local callback server on port 9999
#   3. Open the GitHub App creation page in your browser
#   4. Capture the callback code and exchange it for credentials
#   5. Save credentials to a local file (for 1Password import)
#
# Prerequisites:
#   - Python 3 (for URL encoding and callback server)
#   - gh CLI (authenticated)
#   - Browser access to GitHub
# =============================================================================

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

log_info() { echo -e "${BLUE}ℹ${NC} $1"; }
log_success() { echo -e "${GREEN}✓${NC} $1"; }
log_warn() { echo -e "${YELLOW}⚠${NC} $1"; }
log_error() { echo -e "${RED}✗${NC} $1"; }
log_header() { 
    echo -e "\n${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}  $1${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"
}

# =============================================================================
# Configuration
# =============================================================================

ORG="${GITHUB_ORG:-5dlabs}"
CALLBACK_PORT="${CALLBACK_PORT:-9999}"
CALLBACK_URL="http://localhost:${CALLBACK_PORT}/callback"
OUTPUT_DIR="${OUTPUT_DIR:-./keys}"

# Agent role descriptions
declare -A AGENT_DESCRIPTIONS=(
    ["spark"]="Spark ⚡ - Electron Desktop Engineer. Implements desktop applications with native integration, system tray, auto-update, and professional-grade Electron features."
    ["nova"]="Nova 🌟 - Node.js Backend Engineer. Implements scalable server-side applications with Express, Fastify, and modern Node.js patterns."
    ["grizz"]="Grizz 🐻 - Go Backend Engineer. Implements high-performance backend services with Go, focusing on concurrency, efficiency, and reliability."
    ["tap"]="Tap 📱 - Mobile Engineer. Implements React Native and Expo applications for iOS and Android with native integrations."
    ["blaze"]="Blaze 🔥 - Frontend Engineer. Implements modern web interfaces with React, TypeScript, and cutting-edge UI/UX patterns."
    ["rex"]="Rex 🦖 - Rust Backend Engineer. Implements robust, performant backend systems with Rust, focusing on safety and efficiency."
    ["morgan"]="Morgan 📋 - Project Manager & Documentation. Manages project planning, documentation, and coordinates multi-agent workflows."
    ["cleo"]="Cleo 🔍 - Code Quality Agent. Reviews code quality, runs linters, and ensures best practices are followed."
    ["tess"]="Tess 🧪 - QA Testing Agent. Runs tests, validates acceptance criteria, and ensures quality before deployment."
    ["cipher"]="Cipher 🔐 - Security Agent. Performs security reviews, vulnerability scanning, and compliance checks."
    ["atlas"]="Atlas 🗺️ - PR Guardian & Integration. Monitors PR lifecycle, resolves conflicts, handles CI failures, and manages merges."
    ["bolt"]="Bolt ⚡ - DevOps & Deployment. Handles deployment automation, infrastructure, and CI/CD operations."
    ["stitch"]="Stitch 🧵 - Remediation Agent. Fixes CI failures, resolves build issues, and automates code repairs."
)

# =============================================================================
# Rex's Full Permission Set (the reference for all coder agents)
# =============================================================================

generate_manifest() {
    local agent_name="$1"
    local agent_role="${2:-coder}"
    
    # Get description or use a default
    local description="${AGENT_DESCRIPTIONS[$agent_name]:-"${agent_name^} - CTO Platform Agent"}"
    
    cat << EOF
{
  "name": "${agent_name}-5dlabs",
  "url": "https://github.com/5dlabs/cto",
  "description": "${description}",
  "hook_attributes": {
    "url": "https://webhooks.5dlabs.com/agents/${agent_name}",
    "active": true
  },
  "redirect_url": "${CALLBACK_URL}",
  "callback_urls": [
    "https://platform.5dlabs.com/oauth/callback"
  ],
  "public": false,
  "default_permissions": {
    "actions": "write",
    "actions_variables": "write",
    "administration": "write",
    "attestations": "write",
    "checks": "write",
    "codespaces": "write",
    "codespaces_lifecycle_admin": "write",
    "codespaces_metadata": "read",
    "codespaces_secrets": "write",
    "contents": "write",
    "dependabot_secrets": "write",
    "deployments": "write",
    "discussions": "write",
    "issues": "write",
    "merge_queues": "write",
    "metadata": "read",
    "packages": "write",
    "pages": "write",
    "pull_requests": "write",
    "secret_scanning_alerts": "write",
    "secrets": "write",
    "security_events": "write",
    "statuses": "write",
    "vulnerability_alerts": "write",
    "workflows": "write",
    "repository_hooks": "write",
    "repository_projects": "admin"
  },
  "default_events": [
    "check_run",
    "check_suite",
    "issues",
    "issue_comment",
    "pull_request",
    "pull_request_review",
    "pull_request_review_comment",
    "push",
    "workflow_job",
    "workflow_run"
  ]
}
EOF
}

# =============================================================================
# Callback Server (Python)
# =============================================================================

start_callback_server() {
    local port="$1"
    local output_file="$2"
    
    python3 << PYTHON_EOF &
import http.server
import socketserver
import urllib.parse
import json
import sys
import os

PORT = ${port}
OUTPUT_FILE = "${output_file}"

class CallbackHandler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        parsed = urllib.parse.urlparse(self.path)
        if parsed.path == '/callback':
            params = urllib.parse.parse_qs(parsed.query)
            code = params.get('code', [None])[0]
            
            if code:
                # Save the code
                with open(OUTPUT_FILE, 'w') as f:
                    f.write(code)
                
                # Success response
                self.send_response(200)
                self.send_header('Content-type', 'text/html')
                self.end_headers()
                response = """
                <html>
                <head><title>Success!</title></head>
                <body style="font-family: system-ui; text-align: center; padding: 50px;">
                    <h1>✅ GitHub App Created Successfully!</h1>
                    <p>The callback code has been captured.</p>
                    <p>You can close this window and return to your terminal.</p>
                    <script>setTimeout(() => window.close(), 3000);</script>
                </body>
                </html>
                """
                self.wfile.write(response.encode())
                
                # Shutdown server after handling
                def shutdown():
                    self.server.shutdown()
                import threading
                threading.Thread(target=shutdown).start()
            else:
                self.send_response(400)
                self.send_header('Content-type', 'text/html')
                self.end_headers()
                self.wfile.write(b"<h1>Error: No code received</h1>")
        else:
            self.send_response(404)
            self.end_headers()
    
    def log_message(self, format, *args):
        pass  # Suppress logging

with socketserver.TCPServer(("", PORT), CallbackHandler) as httpd:
    print(f"Callback server listening on port {PORT}...", flush=True)
    httpd.serve_forever()
PYTHON_EOF
    
    echo $!
}

# =============================================================================
# Main Flow
# =============================================================================

main() {
    local agent_name="${1:-}"
    local agent_role="${2:-coder}"
    
    if [[ -z "$agent_name" ]]; then
        echo "Usage: $0 <agent-name> [role]"
        echo ""
        echo "Available agents: spark, nova, grizz, tap, blaze, rex, morgan, cleo, tess, cipher, atlas, bolt, stitch"
        echo "Roles: coder (default), quality, security, deploy, docs"
        exit 1
    fi
    
    log_header "Creating GitHub App: ${agent_name}-5dlabs"
    
    # Create output directory
    mkdir -p "$OUTPUT_DIR"
    local code_file="${OUTPUT_DIR}/.${agent_name}-callback-code"
    local creds_file="${OUTPUT_DIR}/${agent_name}-5dlabs-credentials.json"
    
    # Clean up any previous code file
    rm -f "$code_file"
    
    # Step 1: Generate manifest
    log_info "Generating manifest with Rex-equivalent permissions..."
    local manifest
    manifest=$(generate_manifest "$agent_name" "$agent_role")
    
    # URL encode the manifest
    local encoded_manifest
    encoded_manifest=$(python3 -c "import urllib.parse; import sys; print(urllib.parse.quote(sys.argv[1]))" "$manifest")
    
    local manifest_url="https://github.com/organizations/${ORG}/settings/apps/new?manifest=${encoded_manifest}"
    
    # Step 2: Check if app already exists
    log_info "Checking if ${agent_name}-5dlabs already exists..."
    if gh api "/orgs/${ORG}/installations" --jq ".installations[].app_slug" 2>/dev/null | grep -q "^${agent_name}-5dlabs$"; then
        log_warn "App ${agent_name}-5dlabs is already installed!"
        log_warn "To recreate, first delete it at:"
        echo "    https://github.com/organizations/${ORG}/settings/apps/${agent_name}-5dlabs"
        echo ""
        read -p "Continue anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
    
    # Step 3: Start callback server
    log_info "Starting callback server on port ${CALLBACK_PORT}..."
    local server_pid
    server_pid=$(start_callback_server "$CALLBACK_PORT" "$code_file")
    
    # Ensure server is killed on exit
    trap "kill $server_pid 2>/dev/null || true" EXIT
    
    # Give the server a moment to start
    sleep 1
    
    # Step 4: Open browser
    log_header "Action Required: Create the App in GitHub"
    echo "Opening GitHub in your browser..."
    echo ""
    echo "Steps:"
    echo "  1. Review the pre-filled app settings"
    echo "  2. Click 'Create GitHub App'"
    echo "  3. You'll be redirected back here automatically"
    echo ""
    
    # Open URL in browser
    if command -v open &> /dev/null; then
        open "$manifest_url"
    elif command -v xdg-open &> /dev/null; then
        xdg-open "$manifest_url"
    else
        echo "Please open this URL in your browser:"
        echo "$manifest_url"
    fi
    
    # Step 5: Wait for callback
    log_info "Waiting for GitHub callback..."
    local timeout=300  # 5 minutes
    local elapsed=0
    
    while [[ ! -f "$code_file" ]] && [[ $elapsed -lt $timeout ]]; do
        sleep 1
        ((elapsed++))
        if ((elapsed % 30 == 0)); then
            log_info "Still waiting... (${elapsed}s elapsed)"
        fi
    done
    
    if [[ ! -f "$code_file" ]]; then
        log_error "Timeout waiting for callback. Please try again."
        exit 1
    fi
    
    local code
    code=$(cat "$code_file")
    rm -f "$code_file"
    
    log_success "Received callback code!"
    
    # Step 6: Exchange code for credentials
    log_info "Exchanging code for app credentials..."
    
    local response
    response=$(curl -s -X POST \
        -H "Accept: application/vnd.github+json" \
        "https://api.github.com/app-manifests/${code}/conversions")
    
    # Check for errors
    if echo "$response" | jq -e '.message' &>/dev/null; then
        log_error "GitHub API error: $(echo "$response" | jq -r '.message')"
        exit 1
    fi
    
    # Extract credentials
    local app_id client_id client_secret pem webhook_secret app_slug
    app_id=$(echo "$response" | jq -r '.id')
    client_id=$(echo "$response" | jq -r '.client_id')
    client_secret=$(echo "$response" | jq -r '.client_secret')
    pem=$(echo "$response" | jq -r '.pem')
    webhook_secret=$(echo "$response" | jq -r '.webhook_secret')
    app_slug=$(echo "$response" | jq -r '.slug')
    
    log_success "App created successfully!"
    echo ""
    echo "  App ID:     $app_id"
    echo "  App Slug:   $app_slug"
    echo "  Client ID:  $client_id"
    echo ""
    
    # Step 7: Save credentials
    log_info "Saving credentials..."
    
    # Save full response (for debugging)
    echo "$response" > "${creds_file}"
    
    # Save private key separately
    local pem_file="${OUTPUT_DIR}/${agent_name}-5dlabs.$(date +%Y-%m-%d).private-key.pem"
    echo "$pem" > "$pem_file"
    chmod 600 "$pem_file"
    
    log_success "Credentials saved to:"
    echo "  - Full response: ${creds_file}"
    echo "  - Private key:   ${pem_file}"
    
    # Step 8: Generate 1Password import commands
    log_header "Next Steps: Import to 1Password"
    
    cat << EOF
Run the following to store in 1Password:

# Create 1Password item
op item create \\
    --category="API Credential" \\
    --title="GitHub-App-${agent_name^}" \\
    --vault="Personal" \\
    "app-id=${app_id}" \\
    "client-id=${client_id}" \\
    "client-secret=${client_secret}" \\
    "webhook-secret=${webhook_secret}" \\
    "private-key[file]=${pem_file}"

# Or manually add to 1Password:
#   1. Open 1Password
#   2. Create new "API Credential" item named "GitHub-App-${agent_name^}"
#   3. Add fields: app-id, client-id, client-secret, webhook-secret
#   4. Attach the private key file: ${pem_file}

EOF
    
    # Step 9: Install app to repositories
    log_header "Install the App"
    echo "Visit this URL to install the app on your repositories:"
    echo "  https://github.com/organizations/${ORG}/settings/apps/${app_slug}/installations"
    echo ""
    
    # Generate K8s secret command
    log_header "Kubernetes Secret (for reference)"
    cat << EOF
# Create K8s secret directly (or use kind-secrets-from-1password.sh after 1Password import):
kubectl create secret generic github-app-${agent_name} \\
    --namespace=cto \\
    --from-literal=app-id="${app_id}" \\
    --from-literal=client-id="${client_id}" \\
    --from-literal=client-secret="${client_secret}" \\
    --from-literal=webhook-secret="${webhook_secret}" \\
    --from-file=private-key="${pem_file}"

EOF
    
    log_success "Done! App ${agent_name}-5dlabs is ready."
}

main "$@"

