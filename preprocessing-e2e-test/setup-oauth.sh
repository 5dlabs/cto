#!/bin/bash
# OAuth Setup Script for Preprocessing E2E Test
#
# This script helps set up Linear OAuth tokens for the Morgan agent.
#
# Usage: ./setup-oauth.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

echo ""
echo "=============================================="
echo "Linear OAuth Setup for Preprocessing E2E Test"
echo "=============================================="
echo ""

# Check if 1Password CLI is available
if command -v op &> /dev/null; then
    log_info "1Password CLI found"
    
    # Try to fetch Linear OAuth token from 1Password
    log_info "Attempting to fetch Morgan OAuth token from 1Password..."
    
    # Check if user is signed in
    if op account list &> /dev/null; then
        # Try to get the token
        TOKEN=$(op read 'op://Development/Linear App Morgan/access_token' 2>/dev/null || true)
        
        if [[ -n "$TOKEN" ]]; then
            log_success "Found OAuth token in 1Password"
            export LINEAR_APP_MORGAN_ACCESS_TOKEN="$TOKEN"
        else
            log_warn "No token found at 'op://Development/Linear App Morgan/access_token'"
            log_info "Token may need to be refreshed via OAuth flow"
        fi
    else
        log_warn "Not signed in to 1Password. Run: op signin"
    fi
else
    log_warn "1Password CLI not found. Manual token setup required."
fi

# Check current .env.local
ENV_FILE="$PROJECT_ROOT/.env.local"
if [[ -f "$ENV_FILE" ]]; then
    log_info "Found .env.local at: $ENV_FILE"
    
    # Check if LINEAR_ENABLED is set
    if grep -q "LINEAR_ENABLED=true" "$ENV_FILE"; then
        log_success "LINEAR_ENABLED=true is set"
    else
        log_warn "LINEAR_ENABLED=true not found in .env.local"
        echo ""
        read -p "Add LINEAR_ENABLED=true to .env.local? [y/N] " -n 1 -r
        echo ""
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            echo "LINEAR_ENABLED=true" >> "$ENV_FILE"
            log_success "Added LINEAR_ENABLED=true"
        fi
    fi
    
    # Check if access token is set
    if grep -q "LINEAR_APP_MORGAN_ACCESS_TOKEN=" "$ENV_FILE"; then
        log_success "LINEAR_APP_MORGAN_ACCESS_TOKEN is configured"
    else
        log_warn "LINEAR_APP_MORGAN_ACCESS_TOKEN not found in .env.local"
        echo ""
        
        if [[ -n "${LINEAR_APP_MORGAN_ACCESS_TOKEN:-}" ]]; then
            read -p "Add the token from 1Password to .env.local? [y/N] " -n 1 -r
            echo ""
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                echo "LINEAR_APP_MORGAN_ACCESS_TOKEN=$LINEAR_APP_MORGAN_ACCESS_TOKEN" >> "$ENV_FILE"
                log_success "Added LINEAR_APP_MORGAN_ACCESS_TOKEN"
            fi
        else
            log_info "To get a new OAuth token, you need to:"
            echo "  1. Navigate to the OAuth authorization URL"
            echo "  2. Authorize the Morgan app"
            echo "  3. Capture the redirect with the authorization code"
            echo "  4. Exchange the code for an access token"
            echo ""
            echo "OAuth Authorization URL:"
            echo "  https://linear.app/oauth/authorize?client_id=752f67d53b2b0dab2191832ba0aa43d9&redirect_uri=https://pm-dev.5dlabs.ai/oauth/callback&response_type=code&scope=read,write,admin&state=morgan-setup"
            echo ""
        fi
    fi
else
    log_error "No .env.local found at: $ENV_FILE"
    log_info "Create one from env.template: cp $PROJECT_ROOT/env.template $ENV_FILE"
fi

echo ""
echo "=============================================="
echo "Launchd Service Configuration"
echo "=============================================="
echo ""

# Check if launchd service needs to be updated
PLIST_FILE="$HOME/Library/LaunchAgents/ai.5dlabs.cto.pm-server.plist"
if [[ -f "$PLIST_FILE" ]]; then
    if grep -q "LINEAR_ENABLED" "$PLIST_FILE"; then
        log_success "LINEAR_ENABLED found in launchd plist"
    else
        log_warn "LINEAR_ENABLED not in launchd plist"
        log_info "Run 'just launchd-uninstall && just launchd-install' to regenerate"
        log_info "Or manually add environment variables to the plist"
    fi
else
    log_warn "PM server plist not found"
    log_info "Run 'just launchd-install' to create it"
fi

echo ""
echo "=============================================="
echo "Summary"
echo "=============================================="
echo ""

# Final status
if [[ -n "${LINEAR_APP_MORGAN_ACCESS_TOKEN:-}" ]]; then
    log_success "OAuth token is available in environment"
else
    log_warn "OAuth token NOT available - manual setup required"
fi

echo ""
echo "Next steps:"
echo "  1. Ensure LINEAR_ENABLED=true is in .env.local"
echo "  2. Ensure LINEAR_APP_MORGAN_ACCESS_TOKEN is set"
echo "  3. Regenerate launchd services: just launchd-uninstall && just launchd-install"
echo "  4. Verify PM server has Linear enabled: curl http://localhost:8081/health"
echo ""
