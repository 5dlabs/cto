#!/bin/bash
# =============================================================================
# GitHub App Authentication Script
# =============================================================================
# Converts GitHub App credentials (app-id, private-key, installation-id) into
# an installation access token that can be used with the `gh` CLI.
#
# Required environment variables:
#   GITHUB_APP_ID           - The GitHub App ID
#   GITHUB_APP_PRIVATE_KEY  - The RSA private key (PEM format)
#   GITHUB_APP_INSTALLATION_ID - The installation ID for the org/user
#
# Output:
#   Sets GH_TOKEN and GITHUB_TOKEN environment variables
#   Configures `gh` CLI authentication
#
# Usage:
#   source scripts/github-app-auth.sh
#   # or
#   eval "$(scripts/github-app-auth.sh)"
# =============================================================================

set -euo pipefail

# Check required environment variables
if [[ -z "${GITHUB_APP_ID:-}" ]]; then
    echo "ERROR: GITHUB_APP_ID is not set" >&2
    exit 1
fi

if [[ -z "${GITHUB_APP_PRIVATE_KEY:-}" ]]; then
    echo "ERROR: GITHUB_APP_PRIVATE_KEY is not set" >&2
    exit 1
fi

if [[ -z "${GITHUB_APP_INSTALLATION_ID:-}" ]]; then
    echo "ERROR: GITHUB_APP_INSTALLATION_ID is not set" >&2
    exit 1
fi

# Generate JWT
generate_jwt() {
    local app_id="$1"
    local private_key="$2"
    
    local now
    now=$(date +%s)
    local iat=$((now - 60))  # 60 seconds in the past for clock drift
    local exp=$((now + 600)) # 10 minutes from now (max allowed)
    
    # JWT Header
    local header
    header=$(echo -n '{"alg":"RS256","typ":"JWT"}' | base64 | tr -d '=' | tr '/+' '_-' | tr -d '\n')
    
    # JWT Payload
    local payload
    payload=$(echo -n "{\"iat\":${iat},\"exp\":${exp},\"iss\":\"${app_id}\"}" | base64 | tr -d '=' | tr '/+' '_-' | tr -d '\n')
    
    # Sign with private key
    local signature
    signature=$(echo -n "${header}.${payload}" | openssl dgst -sha256 -sign <(echo "$private_key") | base64 | tr -d '=' | tr '/+' '_-' | tr -d '\n')
    
    echo "${header}.${payload}.${signature}"
}

# Exchange JWT for installation access token
get_installation_token() {
    local jwt="$1"
    local installation_id="$2"
    
    local response
    response=$(curl -s -X POST \
        -H "Authorization: Bearer ${jwt}" \
        -H "Accept: application/vnd.github+json" \
        -H "X-GitHub-Api-Version: 2022-11-28" \
        "https://api.github.com/app/installations/${installation_id}/access_tokens")
    
    # Extract token from response
    local token
    token=$(echo "$response" | jq -r '.token // empty')
    
    if [[ -z "$token" ]]; then
        echo "ERROR: Failed to get installation token" >&2
        echo "Response: $response" >&2
        exit 1
    fi
    
    echo "$token"
}

# Main
main() {
    echo "🔐 Generating GitHub App JWT..." >&2
    local jwt
    jwt=$(generate_jwt "$GITHUB_APP_ID" "$GITHUB_APP_PRIVATE_KEY")
    
    echo "🔄 Exchanging JWT for installation access token..." >&2
    local token
    token=$(get_installation_token "$jwt" "$GITHUB_APP_INSTALLATION_ID")
    
    echo "✅ GitHub App authentication successful" >&2
    
    # Output export commands (for eval or sourcing)
    echo "export GH_TOKEN='${token}'"
    echo "export GITHUB_TOKEN='${token}'"
    
    # Also configure gh CLI if available
    if command -v gh &> /dev/null; then
        echo "$token" | gh auth login --with-token 2>/dev/null || true
        echo "✅ gh CLI authenticated" >&2
    fi
}

main "$@"
