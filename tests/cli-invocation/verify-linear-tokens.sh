#!/usr/bin/env bash
# =============================================================================
# Verify Linear OAuth Tokens
# =============================================================================
#
# Tests that each agent's Linear OAuth token can access the Linear API
# and optionally tests the refresh mechanism.
#
# Usage:
#   ./verify-linear-tokens.sh           # Test all agents
#   ./verify-linear-tokens.sh bolt      # Test single agent
#   ./verify-linear-tokens.sh --refresh # Test with token refresh
#
# Requirements:
#   - op (1Password CLI) authenticated
#   - curl and jq
#
# =============================================================================

set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Parse arguments
TEST_REFRESH=false
AGENTS_TO_TEST=()

for arg in "$@"; do
    case "$arg" in
        --refresh)
            TEST_REFRESH=true
            ;;
        *)
            AGENTS_TO_TEST+=("$arg")
            ;;
    esac
done

# Function to get Linear OAuth item name for an agent
get_linear_oauth_item() {
    local agent="$1"
    # Map to expected 1Password item names
    case "$agent" in
        atlas)  echo "Linear Atlas OAuth" ;;
        blaze)  echo "Linear Blaze OAuth" ;;
        bolt)   echo "Linear Bolt OAuth" ;;
        cipher) echo "Linear Cipher OAuth" ;;
        cleo)   echo "Linear Cleo OAuth" ;;
        grizz)  echo "Linear Grizz OAuth" ;;
        morgan) echo "Linear Morgan OAuth" ;;
        nova)   echo "Linear Nova OAuth" ;;
        rex)    echo "Linear Rex OAuth" ;;
        spark)  echo "Linear Spark OAuth" ;;
        stitch) echo "Linear Stitch OAuth" ;;
        tap)    echo "Linear Tap OAuth" ;;
        tess)   echo "Linear Tess OAuth" ;;
        vex)    echo "Linear Vex OAuth" ;;
        *)      echo "" ;;
    esac
}

# All agents
ALL_AGENTS=(atlas blaze bolt cipher cleo grizz morgan nova rex spark stitch tap tess vex)

# If specific agents provided, use those
if [ ${#AGENTS_TO_TEST[@]} -gt 0 ]; then
    AGENTS=("${AGENTS_TO_TEST[@]}")
else
    AGENTS=("${ALL_AGENTS[@]}")
fi

echo "🔐 Verifying Linear OAuth tokens for ${#AGENTS[@]} agent(s)..."
if [ "$TEST_REFRESH" = true ]; then
    echo "   (with refresh test enabled)"
fi
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
REFRESH_TESTED=0
REFRESH_PASSED=0

for agent in "${AGENTS[@]}"; do
    ITEM_NAME=$(get_linear_oauth_item "$agent")
    
    echo "🔍 Testing: ${agent}"
    
    # Check if the item exists
    if ! op item get "${ITEM_NAME}" --vault "Automation" &> /dev/null 2>&1; then
        echo "   ⏭️  No Linear OAuth item found: ${ITEM_NAME}"
        ((SKIPPED++))
        continue
    fi
    
    echo "   📦 Found: ${ITEM_NAME}"
    
    # Try to read the developer_token (access token)
    ACCESS_TOKEN=$(op item get "${ITEM_NAME}" --vault "Automation" --fields label=developer_token --reveal 2>/dev/null || echo "")
    
    if [ -z "${ACCESS_TOKEN}" ]; then
        # Try password field as fallback
        ACCESS_TOKEN=$(op item get "${ITEM_NAME}" --vault "Automation" --fields label=password --reveal 2>/dev/null || echo "")
    fi
    
    if [ -z "${ACCESS_TOKEN}" ]; then
        echo "   ❌ No access token found (checked developer_token, password)"
        ((FAILED++))
        continue
    fi
    
    # Test the token against Linear API
    RESPONSE=$(curl -s -w "\n%{http_code}" \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" \
        -H "Content-Type: application/json" \
        -d '{"query":"{ viewer { id name email } }"}' \
        "https://api.linear.app/graphql" 2>/dev/null || echo -e "\n000")
    
    HTTP_CODE=$(echo "$RESPONSE" | tail -1)
    BODY=$(echo "$RESPONSE" | sed '$d')
    
    if [ "${HTTP_CODE}" = "200" ]; then
        # Check for GraphQL errors
        if echo "$BODY" | jq -e '.errors' &>/dev/null; then
            ERROR=$(echo "$BODY" | jq -r '.errors[0].message // "Unknown error"')
            echo "   ❌ GraphQL error: ${ERROR}"
            ((FAILED++))
        else
            # Success - extract user info
            USER_NAME=$(echo "$BODY" | jq -r '.data.viewer.name // "unknown"')
            USER_EMAIL=$(echo "$BODY" | jq -r '.data.viewer.email // "unknown"')
            echo "   ✓ Token valid (user: ${USER_NAME} <${USER_EMAIL}>)"
            ((PASSED++))
            
            # Test refresh if requested
            if [ "$TEST_REFRESH" = true ]; then
                REFRESH_TOKEN=$(op item get "${ITEM_NAME}" --vault "Automation" --fields label=refresh_token --reveal 2>/dev/null || echo "")
                CLIENT_ID=$(op item get "${ITEM_NAME}" --vault "Automation" --fields label=client_id --reveal 2>/dev/null || echo "")
                CLIENT_SECRET=$(op item get "${ITEM_NAME}" --vault "Automation" --fields label=client_secret --reveal 2>/dev/null || echo "")
                
                if [ -n "${REFRESH_TOKEN}" ] && [ -n "${CLIENT_ID}" ] && [ -n "${CLIENT_SECRET}" ]; then
                    echo "   🔄 Testing token refresh..."
                    ((REFRESH_TESTED++))
                    
                    REFRESH_RESPONSE=$(curl -s -w "\n%{http_code}" \
                        -X POST \
                        -d "grant_type=refresh_token" \
                        -d "refresh_token=${REFRESH_TOKEN}" \
                        -d "client_id=${CLIENT_ID}" \
                        -d "client_secret=${CLIENT_SECRET}" \
                        "https://api.linear.app/oauth/token" 2>/dev/null || echo -e "\n000")
                    
                    REFRESH_HTTP=$(echo "$REFRESH_RESPONSE" | tail -1)
                    REFRESH_BODY=$(echo "$REFRESH_RESPONSE" | sed '$d')
                    
                    if [ "${REFRESH_HTTP}" = "200" ]; then
                        NEW_TOKEN=$(echo "$REFRESH_BODY" | jq -r '.access_token // empty')
                        if [ -n "${NEW_TOKEN}" ]; then
                            echo "   ✓ Refresh successful (new token obtained)"
                            ((REFRESH_PASSED++))
                        else
                            echo "   ❌ Refresh returned 200 but no token"
                        fi
                    else
                        ERROR=$(echo "$REFRESH_BODY" | jq -r '.error_description // .error // "Unknown"')
                        echo "   ❌ Refresh failed (HTTP ${REFRESH_HTTP}): ${ERROR}"
                    fi
                else
                    echo "   ⚠️  Missing refresh credentials (refresh_token, client_id, or client_secret)"
                fi
            fi
        fi
    elif [ "${HTTP_CODE}" = "401" ]; then
        echo "   ❌ Token invalid or expired (401)"
        ((FAILED++))
    else
        echo "   ❌ API call failed (HTTP ${HTTP_CODE})"
        ((FAILED++))
    fi
    
    echo ""
done

echo "=============================================="
echo "📊 Summary"
echo "   Passed:  ${PASSED}"
echo "   Failed:  ${FAILED}"
echo "   Skipped: ${SKIPPED}"
if [ "$TEST_REFRESH" = true ]; then
    echo "   Refresh tested: ${REFRESH_TESTED}"
    echo "   Refresh passed: ${REFRESH_PASSED}"
fi
echo "=============================================="

# Report missing tokens
echo ""
echo "📝 Notes:"
echo "   Agents need Linear OAuth Apps created in Linear settings."
echo "   OAuth flow: PM server handles /oauth/callback for each agent."
echo ""
echo "   To create a new Linear OAuth App:"
echo "   1. Go to Linear Settings → API → OAuth Applications"
echo "   2. Create app with redirect URI: https://pm.5dlabs.ai/oauth/{agent}/callback"
echo "   3. Store client_id, client_secret in 1Password as 'Linear {Agent} OAuth'"
echo "   4. User authorizes app → PM server stores access_token + refresh_token"

# Exit with error if any failed
if [ ${FAILED} -gt 0 ]; then
    exit 1
fi
