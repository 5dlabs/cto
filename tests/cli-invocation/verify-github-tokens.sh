#!/usr/bin/env bash
# =============================================================================
# Verify GitHub App OAuth Tokens
# =============================================================================
#
# Tests that each agent's GitHub App token can access the GitHub API.
#
# Usage:
#   ./verify-github-tokens.sh           # Test all agents
#   ./verify-github-tokens.sh bolt      # Test single agent
#
# Requirements:
#   - op (1Password CLI) authenticated
#   - gh (GitHub CLI) or curl
#
# =============================================================================

set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Function to get GitHub App name for an agent
get_github_app() {
    local agent="$1"
    case "$agent" in
        atlas)  echo "GitHub-App-Atlas" ;;
        blaze)  echo "GitHub-App-Blaze" ;;
        bolt)   echo "GitHub-App-Bolt" ;;
        cipher) echo "GitHub-App-Cipher" ;;
        cleo)   echo "GitHub-App-Cleo" ;;
        grizz)  echo "GitHub-App-Grizz" ;;
        morgan) echo "GitHub-App-Morgan" ;;
        nova)   echo "GitHub-App-Nova" ;;
        rex)    echo "GitHub-App-Rex" ;;
        spark)  echo "GitHub-App-Spark" ;;
        stitch) echo "GitHub-App-Stitch" ;;
        tap)    echo "GitHub-App-Tap" ;;
        tess)   echo "GitHub-App-Tess" ;;
        vex)    echo "" ;;  # No GitHub App for vex
        *)      echo "" ;;
    esac
}

# All agents
ALL_AGENTS=(atlas blaze bolt cipher cleo grizz morgan nova rex spark stitch tap tess vex)

# If specific agent provided, use only that
if [ $# -ge 1 ] && [ -n "$1" ]; then
    AGENTS=("$1")
else
    AGENTS=("${ALL_AGENTS[@]}")
fi

echo "🔐 Verifying GitHub App tokens for ${#AGENTS[@]} agent(s)..."
echo ""

# Check 1Password CLI is available
if ! command -v op &> /dev/null; then
    echo "Error: 1Password CLI (op) is required"
    exit 1
fi

# Check we're signed in
if ! op whoami &> /dev/null; then
    echo "Error: Not signed in to 1Password. Run: op signin"
    exit 1
fi

# Stats
PASSED=0
FAILED=0
SKIPPED=0

for agent in "${AGENTS[@]}"; do
    APP_NAME=$(get_github_app "$agent")
    
    if [ -z "${APP_NAME}" ]; then
        echo "⏭️  ${agent}: No GitHub App configured (skipped)"
        ((SKIPPED++))
        continue
    fi
    
    echo "🔍 Testing: ${agent} (${APP_NAME})"
    
    # Check if the item exists
    if ! op item get "${APP_NAME}" --vault "Automation" &> /dev/null; then
        echo "   ❌ Item not found in 1Password: ${APP_NAME}"
        ((FAILED++))
        continue
    fi
    
    # Try to read the credential field (user access token or installation token)
    CREDENTIAL=$(op item get "${APP_NAME}" --vault "Automation" --fields label=credential --reveal 2>/dev/null || echo "")
    
    if [ -z "${CREDENTIAL}" ]; then
        echo "   ⚠️  No credential field found, checking for private-key..."
        
        # Check if private-key exists (can be used to generate tokens)
        if op item get "${APP_NAME}" --vault "Automation" --fields label=private-key &> /dev/null; then
            echo "   ✓ Has private-key (can generate tokens)"
            ((PASSED++))
        else
            echo "   ❌ No credential or private-key found"
            ((FAILED++))
        fi
        continue
    fi
    
    # Test the credential against GitHub API
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
        -H "Authorization: Bearer ${CREDENTIAL}" \
        -H "Accept: application/vnd.github+json" \
        -H "X-GitHub-Api-Version: 2022-11-28" \
        "https://api.github.com/user" 2>/dev/null || echo "000")
    
    if [ "${HTTP_CODE}" = "200" ]; then
        # Get username
        USERNAME=$(curl -s \
            -H "Authorization: Bearer ${CREDENTIAL}" \
            -H "Accept: application/vnd.github+json" \
            -H "X-GitHub-Api-Version: 2022-11-28" \
            "https://api.github.com/user" 2>/dev/null | jq -r '.login // "unknown"')
        echo "   ✓ Token valid (user: ${USERNAME})"
        ((PASSED++))
    elif [ "${HTTP_CODE}" = "401" ]; then
        echo "   ❌ Token invalid or expired (401)"
        ((FAILED++))
    elif [ "${HTTP_CODE}" = "403" ]; then
        echo "   ⚠️  Token has limited permissions (403) - may still work for repos"
        ((PASSED++))
    else
        echo "   ❌ API call failed (HTTP ${HTTP_CODE})"
        ((FAILED++))
    fi
done

echo ""
echo "=============================================="
echo "📊 Summary"
echo "   Passed:  ${PASSED}"
echo "   Failed:  ${FAILED}"
echo "   Skipped: ${SKIPPED}"
echo "=============================================="

# Report missing apps
echo ""
echo "📝 Notes:"
for agent in "${ALL_AGENTS[@]}"; do
    APP_NAME=$(get_github_app "$agent")
    if [ -z "${APP_NAME}" ]; then
        echo "   - ${agent}: No GitHub App exists (needs creation)"
    fi
done

# Exit with error if any failed
if [ ${FAILED} -gt 0 ]; then
    exit 1
fi
