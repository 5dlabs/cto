#!/usr/bin/env bash
# =============================================================================
# Linear Agent OAuth Setup Helper
# =============================================================================
# This script helps you set up Linear OAuth apps for CTO platform agents.
#
# MANUAL STEPS REQUIRED:
# 1. Create each OAuth app at: https://linear.app/settings/api/applications/new
# 2. Run this script to store credentials and generate install URLs
# 3. Visit each install URL to authorize the app in your workspace
#
# Usage:
#   ./scripts/setup-linear-agents.sh add <agent>      # Add a single agent
#   ./scripts/setup-linear-agents.sh add-all          # Interactive setup for all agents
#   ./scripts/setup-linear-agents.sh urls             # Generate install URLs
#   ./scripts/setup-linear-agents.sh status           # Check which agents are configured
#   ./scripts/setup-linear-agents.sh install <agent>  # Open install URL in browser
# =============================================================================

set -euo pipefail

# All CTO platform agents
AGENTS=(morgan rex blaze grizz nova tap spark cleo cipher tess atlas bolt vex)

# OAuth configuration
REDIRECT_URI="https://cto.5dlabs.ai/oauth/callback"
WEBHOOK_URL="https://cto.5dlabs.ai/webhooks/linear"
SCOPES="read,write,app:assignable,app:mentionable"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_header() {
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo ""
}

print_step() {
    echo -e "${GREEN}▶${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

# Check if bao CLI is available
check_bao() {
    if ! command -v bao &> /dev/null; then
        print_error "OpenBao CLI (bao) not found. Please install it or ensure it's in your PATH."
        echo "  You may need to: export VAULT_ADDR=... and authenticate first"
        exit 1
    fi
}

# Capitalize first letter
capitalize() {
    echo "$1" | awk '{print toupper(substr($0,1,1)) tolower(substr($0,2))}'
}

# Add a single agent's credentials
add_agent() {
    local agent="$1"
    local display_name="5DLabs-$(capitalize "$agent")"
    
    print_header "Setting up $display_name"
    
    echo "First, create the OAuth app in Linear if you haven't already:"
    echo ""
    echo -e "  ${BLUE}https://linear.app/settings/api/applications/new${NC}"
    echo ""
    echo "Configure with these settings:"
    echo "  • Name: $display_name"
    echo "  • Callback URL: $REDIRECT_URI"
    echo "  • Webhook URL: $WEBHOOK_URL"
    echo "  • Webhook Events: ✓ Agent session events, ✓ Permission changes"
    echo ""
    
    read -rp "Have you created the app? (y/n): " created
    if [[ "$created" != "y" && "$created" != "Y" ]]; then
        echo "Please create the app first, then run this command again."
        exit 1
    fi
    
    echo ""
    print_step "Enter the credentials from Linear:"
    echo ""
    
    read -rp "  Client ID (32 hex chars): " client_id
    read -rp "  Client Secret (32 hex chars): " client_secret
    read -rp "  Webhook Secret (lin_wh_...): " webhook_secret
    
    # Validate client_id format
    if [[ ! "$client_id" =~ ^[a-f0-9]{32}$ ]]; then
        print_error "Invalid client_id format. Expected 32 hex characters."
        exit 1
    fi
    
    # Validate client_secret format
    if [[ ! "$client_secret" =~ ^[a-f0-9]{32}$ ]]; then
        print_error "Invalid client_secret format. Expected 32 hex characters."
        exit 1
    fi
    
    # Validate webhook_secret format
    if [[ ! "$webhook_secret" =~ ^lin_wh_ ]]; then
        print_error "Invalid webhook_secret format. Expected to start with 'lin_wh_'."
        exit 1
    fi
    
    echo ""
    print_step "Storing credentials in OpenBao..."
    
    bao kv put "linear-app-$agent" \
        client_id="$client_id" \
        client_secret="$client_secret" \
        webhook_secret="$webhook_secret"
    
    print_success "Credentials stored for $display_name"
    
    # Generate and show the install URL
    echo ""
    print_step "OAuth Install URL (visit this to authorize the app):"
    echo ""
    local install_url="https://linear.app/oauth/authorize?client_id=${client_id}&redirect_uri=$(urlencode "$REDIRECT_URI")&response_type=code&scope=${SCOPES}&actor=app"
    echo -e "  ${BLUE}${install_url}${NC}"
    echo ""
    
    read -rp "Open in browser now? (y/n): " open_browser
    if [[ "$open_browser" == "y" || "$open_browser" == "Y" ]]; then
        if command -v open &> /dev/null; then
            open "$install_url"
        elif command -v xdg-open &> /dev/null; then
            xdg-open "$install_url"
        else
            echo "Could not detect browser opener. Please visit the URL manually."
        fi
    fi
}

# URL encode helper
urlencode() {
    python3 -c "import urllib.parse; print(urllib.parse.quote('$1', safe=''))"
}

# Add all agents interactively
add_all() {
    print_header "Setup All Linear Agent OAuth Apps"
    
    echo "This will guide you through setting up all ${#AGENTS[@]} agents."
    echo ""
    echo "For each agent, you'll need to:"
    echo "  1. Create an OAuth app in Linear"
    echo "  2. Enter the credentials here"
    echo "  3. Authorize the app in your workspace"
    echo ""
    
    for agent in "${AGENTS[@]}"; do
        echo ""
        echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        read -rp "Set up $(capitalize "$agent")? (y/n/q to quit): " choice
        case "$choice" in
            y|Y) add_agent "$agent" ;;
            q|Q) echo "Quitting."; exit 0 ;;
            *) echo "Skipping $(capitalize "$agent")..." ;;
        esac
    done
    
    print_header "Setup Complete!"
    echo "Run './scripts/setup-linear-agents.sh status' to verify configuration."
}

# Show status of all agents
show_status() {
    print_header "Linear Agent OAuth Status"
    
    check_bao
    
    printf "%-12s %-15s %-15s %-15s\n" "Agent" "Credentials" "Access Token" "Status"
    printf "%-12s %-15s %-15s %-15s\n" "────────" "───────────" "────────────" "──────"
    
    for agent in "${AGENTS[@]}"; do
        local creds="❌ Missing"
        local token="❌ Missing"
        local status="Not Ready"
        
        # Check if credentials exist
        if bao kv get "linear-app-$agent" &>/dev/null; then
            local data
            data=$(bao kv get -format=json "linear-app-$agent" 2>/dev/null | jq -r '.data.data // {}')
            local client_id
            client_id=$(echo "$data" | jq -r '.client_id // ""')
            local access_token
            access_token=$(echo "$data" | jq -r '.access_token // ""')
            
            if [[ -n "$client_id" && "$client_id" != "null" ]]; then
                creds="✅ OK"
            fi
            
            if [[ -n "$access_token" && "$access_token" != "null" ]]; then
                token="✅ Installed"
                status="Ready"
            else
                status="Need Install"
            fi
        fi
        
        printf "%-12s %-15s %-15s %-15s\n" "$(capitalize "$agent")" "$creds" "$token" "$status"
    done
    
    echo ""
}

# Generate install URLs for all configured agents
generate_urls() {
    print_header "OAuth Install URLs"
    
    check_bao
    
    echo "Visit each URL to install the agent in your Linear workspace."
    echo "You must be a workspace admin to complete installation."
    echo ""
    
    local found=0
    for agent in "${AGENTS[@]}"; do
        local client_id
        client_id=$(bao kv get -format=json "linear-app-$agent" 2>/dev/null | jq -r '.data.data.client_id // ""' 2>/dev/null || echo "")
        
        if [[ -n "$client_id" && "$client_id" != "null" ]]; then
            found=$((found + 1))
            local display_name="5DLabs-$(capitalize "$agent")"
            local install_url="https://linear.app/oauth/authorize?client_id=${client_id}&redirect_uri=$(urlencode "$REDIRECT_URI")&response_type=code&scope=${SCOPES}&actor=app"
            
            echo -e "${GREEN}$display_name${NC}"
            echo "  $install_url"
            echo ""
        fi
    done
    
    if [[ $found -eq 0 ]]; then
        print_warn "No agents configured yet. Run './scripts/setup-linear-agents.sh add <agent>' first."
    else
        echo "Found $found configured agent(s)."
    fi
}

# Open install URL for a specific agent
install_agent() {
    local agent="$1"
    
    check_bao
    
    local client_id
    client_id=$(bao kv get -format=json "linear-app-$agent" 2>/dev/null | jq -r '.data.data.client_id // ""' 2>/dev/null || echo "")
    
    if [[ -z "$client_id" || "$client_id" == "null" ]]; then
        print_error "Agent '$agent' not configured. Run './scripts/setup-linear-agents.sh add $agent' first."
        exit 1
    fi
    
    local install_url="https://linear.app/oauth/authorize?client_id=${client_id}&redirect_uri=$(urlencode "$REDIRECT_URI")&response_type=code&scope=${SCOPES}&actor=app"
    
    print_step "Opening install URL for 5DLabs-$(capitalize "$agent")..."
    echo "  $install_url"
    
    if command -v open &> /dev/null; then
        open "$install_url"
    elif command -v xdg-open &> /dev/null; then
        xdg-open "$install_url"
    else
        echo "Could not detect browser opener. Please visit the URL manually."
    fi
}

# Print Linear app creation instructions
print_linear_instructions() {
    print_header "Linear OAuth App Creation Guide"
    
    echo "For each agent, create an OAuth app at:"
    echo -e "  ${BLUE}https://linear.app/settings/api/applications/new${NC}"
    echo ""
    echo "Use these settings:"
    echo ""
    printf "%-12s %-20s\n" "Agent" "App Name"
    printf "%-12s %-20s\n" "────────" "────────────────"
    for agent in "${AGENTS[@]}"; do
        printf "%-12s %-20s\n" "$(capitalize "$agent")" "5DLabs-$(capitalize "$agent")"
    done
    echo ""
    echo "Common settings for ALL apps:"
    echo "  • Callback URL:    $REDIRECT_URI"
    echo "  • Webhook URL:     $WEBHOOK_URL"
    echo "  • Webhook Events:  ✓ Agent session events"
    echo "                     ✓ Permission changes"
    echo "                     ✓ Inbox notifications"
    echo ""
}

# Main command handler
case "${1:-}" in
    add)
        if [[ -z "${2:-}" ]]; then
            echo "Usage: $0 add <agent>"
            echo "Available agents: ${AGENTS[*]}"
            exit 1
        fi
        check_bao
        add_agent "$2"
        ;;
    add-all)
        check_bao
        add_all
        ;;
    urls)
        generate_urls
        ;;
    status)
        show_status
        ;;
    install)
        if [[ -z "${2:-}" ]]; then
            echo "Usage: $0 install <agent>"
            echo "Available agents: ${AGENTS[*]}"
            exit 1
        fi
        install_agent "$2"
        ;;
    guide)
        print_linear_instructions
        ;;
    *)
        echo "Linear Agent OAuth Setup Helper"
        echo ""
        echo "Usage: $0 <command> [options]"
        echo ""
        echo "Commands:"
        echo "  guide              Show Linear app creation instructions"
        echo "  add <agent>        Add credentials for a single agent"
        echo "  add-all            Interactive setup for all agents"
        echo "  urls               Generate OAuth install URLs for configured agents"
        echo "  status             Show configuration status of all agents"
        echo "  install <agent>    Open OAuth install URL in browser"
        echo ""
        echo "Available agents: ${AGENTS[*]}"
        echo ""
        echo "Quick Start:"
        echo "  1. Run '$0 guide' to see Linear app creation instructions"
        echo "  2. Create apps in Linear at https://linear.app/settings/api/applications/new"
        echo "  3. Run '$0 add <agent>' to store each agent's credentials"
        echo "  4. Run '$0 install <agent>' to authorize each app"
        echo ""
        exit 1
        ;;
esac
