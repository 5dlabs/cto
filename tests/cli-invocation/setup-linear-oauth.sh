#!/usr/bin/env bash
# =============================================================================
# Setup Linear OAuth for All Agents
# =============================================================================
#
# Creates and validates Linear OAuth tokens for all CTO agents.
# Tests authentication and attempts token refresh where possible.
#
# Usage:
#   ./setup-linear-oauth.sh              # Audit all agents
#   ./setup-linear-oauth.sh --refresh    # Attempt refresh for expired tokens
#   ./setup-linear-oauth.sh bolt         # Test single agent
#
# =============================================================================

set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Parse arguments
DO_REFRESH=false
SPECIFIC_AGENT=""

for arg in "$@"; do
    case "$arg" in
        --refresh) DO_REFRESH=true ;;
        *)         SPECIFIC_AGENT="$arg" ;;
    esac
done

# All agents (from PM deployment)
ALL_AGENTS=(morgan rex blaze grizz nova tap spark cleo cipher tess atlas bolt stitch vex)

if [ -n "$SPECIFIC_AGENT" ]; then
    AGENTS=("$SPECIFIC_AGENT")
else
    AGENTS=("${ALL_AGENTS[@]}")
fi

echo "🔐 Linear OAuth Setup for ${#AGENTS[@]} agent(s)"
echo "=============================================="
echo ""

# Check 1Password
if ! op whoami &> /dev/null; then
    echo "❌ Not signed in to 1Password"
    exit 1
fi

# Item name for client secrets
SECRETS_ITEM="Linear Agent Client Secrets (Rotated 2026-01-02)"

# Function to get client_secret from central item
get_client_secret() {
    local agent="$1"
    # Capitalize first letter
    local section
    case "$agent" in
        morgan) section="Morgan" ;;
        rex)    section="Rex" ;;
        blaze)  section="Blaze" ;;
        grizz)  section="Grizz" ;;
        nova)   section="Nova" ;;
        tap)    section="Tap" ;;
        spark)  section="Spark" ;;
        cleo)   section="Cleo" ;;
        cipher) section="Cipher" ;;
        tess)   section="Tess" ;;
        atlas)  section="Atlas" ;;
        bolt)   section="Bolt" ;;
        stitch) section="Stitch" ;;
        vex)    section="Vex" ;;
        *)      section="$agent" ;;
    esac
    op item get "$SECRETS_ITEM" --vault "Automation" --fields "section=${section}.client_secret" --reveal 2>/dev/null || echo ""
}

# Function to get agent's OAuth item name
get_oauth_item() {
    local agent="$1"
    case "$agent" in
        morgan) echo "Linear Morgan OAuth" ;;
        rex)    echo "Linear Rex OAuth" ;;
        blaze)  echo "Linear Blaze OAuth" ;;
        grizz)  echo "Linear Grizz OAuth" ;;
        nova)   echo "Linear Nova OAuth" ;;
        tap)    echo "Linear Tap OAuth" ;;
        spark)  echo "Linear Spark OAuth" ;;
        cleo)   echo "Linear Cleo OAuth" ;;
        cipher) echo "Linear Cipher OAuth" ;;
        tess)   echo "Linear Tess OAuth" ;;
        atlas)  echo "Linear Atlas OAuth" ;;
        bolt)   echo "Linear Bolt OAuth" ;;
        stitch) echo "Linear Stitch OAuth" ;;
        vex)    echo "Linear Vex OAuth" ;;
        *)      echo "Linear $agent OAuth" ;;
    esac
}

# Stats
VALID=0
EXPIRED=0
MISSING=0
REFRESHED=0
NEEDS_AUTH=0
NEED_ATTENTION=""

echo "📋 Agent OAuth Status"
echo "--------------------------------------------"
printf "%-10s %-12s %-40s\n" "Agent" "Status" "Details"
echo "--------------------------------------------"

for agent in "${AGENTS[@]}"; do
    ITEM_NAME=$(get_oauth_item "$agent")
    STATUS="?"
    DETAILS=""
    
    # Check if per-agent OAuth item exists
    if op item get "$ITEM_NAME" --vault "Automation" &>/dev/null 2>&1; then
        # Get credentials
        ACCESS_TOKEN=$(op item get "$ITEM_NAME" --vault "Automation" --fields label=developer_token --reveal 2>/dev/null || \
                       op item get "$ITEM_NAME" --vault "Automation" --fields label=password --reveal 2>/dev/null || echo "")
        CLIENT_ID=$(op item get "$ITEM_NAME" --vault "Automation" --fields label=client_id --reveal 2>/dev/null || echo "")
        CLIENT_SECRET=$(op item get "$ITEM_NAME" --vault "Automation" --fields label=client_secret --reveal 2>/dev/null || echo "")
        REFRESH_TOKEN=$(op item get "$ITEM_NAME" --vault "Automation" --fields label=refresh_token --reveal 2>/dev/null || echo "")
        
        if [ -n "$ACCESS_TOKEN" ]; then
            # Test the token
            RESPONSE=$(curl -s -w "\n%{http_code}" \
                -H "Authorization: Bearer $ACCESS_TOKEN" \
                -H "Content-Type: application/json" \
                -d '{"query":"{ viewer { id name } }"}' \
                "https://api.linear.app/graphql" 2>/dev/null || echo -e "\n000")
            
            HTTP_CODE=$(echo "$RESPONSE" | tail -1)
            BODY=$(echo "$RESPONSE" | sed '$d')
            
            if [ "$HTTP_CODE" = "200" ] && ! echo "$BODY" | jq -e '.errors' &>/dev/null; then
                USER_NAME=$(echo "$BODY" | jq -r '.data.viewer.name // "unknown"')
                STATUS="✅ valid"
                DETAILS="user: $USER_NAME"
                ((VALID++))
            else
                STATUS="❌ expired"
                DETAILS="needs refresh"
                ((EXPIRED++))
                NEED_ATTENTION="$NEED_ATTENTION $agent"
                
                # Attempt refresh if requested
                if [ "$DO_REFRESH" = true ] && [ -n "$REFRESH_TOKEN" ] && [ -n "$CLIENT_ID" ] && [ -n "$CLIENT_SECRET" ]; then
                    echo ""
                    echo "   🔄 Attempting refresh for $agent..."
                    
                    REFRESH_RESPONSE=$(curl -s -X POST \
                        -d "grant_type=refresh_token" \
                        -d "refresh_token=$REFRESH_TOKEN" \
                        -d "client_id=$CLIENT_ID" \
                        -d "client_secret=$CLIENT_SECRET" \
                        "https://api.linear.app/oauth/token" 2>/dev/null)
                    
                    NEW_TOKEN=$(echo "$REFRESH_RESPONSE" | jq -r '.access_token // empty')
                    NEW_REFRESH=$(echo "$REFRESH_RESPONSE" | jq -r '.refresh_token // empty')
                    
                    if [ -n "$NEW_TOKEN" ]; then
                        # Update 1Password
                        op item edit "$ITEM_NAME" --vault "Automation" \
                            "developer_token[concealed]=$NEW_TOKEN" \
                            ${NEW_REFRESH:+"refresh_token[concealed]=$NEW_REFRESH"} &>/dev/null
                        
                        STATUS="🔄 refreshed"
                        DETAILS="new token saved"
                        ((REFRESHED++))
                        
                    else
                        ERROR=$(echo "$REFRESH_RESPONSE" | jq -r '.error // "unknown"')
                        DETAILS="refresh failed: $ERROR"
                    fi
                elif [ "$DO_REFRESH" = true ]; then
                    DETAILS="missing refresh credentials"
                fi
            fi
        else
            STATUS="⚠️  no token"
            DETAILS="has client creds, needs OAuth"
            ((NEEDS_AUTH++))
            NEED_ATTENTION="$NEED_ATTENTION $agent"
        fi
    else
        # No per-agent item - check if we have client_secret in central item
        CENTRAL_SECRET=$(get_client_secret "$agent")
        
        if [ -n "$CENTRAL_SECRET" ]; then
            STATUS="📦 partial"
            DETAILS="has secret, needs OAuth item"
            ((MISSING++))
            NEED_ATTENTION="$NEED_ATTENTION $agent"
        else
            STATUS="❌ missing"
            DETAILS="no OAuth app configured"
            ((MISSING++))
            NEED_ATTENTION="$NEED_ATTENTION $agent"
        fi
    fi
    
    printf "%-10s %-12s %-40s\n" "$agent" "$STATUS" "$DETAILS"
done

echo "--------------------------------------------"
echo ""
echo "📊 Summary"
echo "   ✅ Valid:     $VALID"
echo "   ❌ Expired:   $EXPIRED"
echo "   🔄 Refreshed: $REFRESHED"
echo "   ⚠️  Needs Auth: $NEEDS_AUTH"
echo "   📦 Partial:   $MISSING"
echo ""

# Show agents needing attention
if [ -n "$NEED_ATTENTION" ]; then
    echo "🔧 Agents needing attention:$NEED_ATTENTION"
    echo ""
    echo "To authorize an agent:"
    echo "  1. Ensure OAuth app exists in Linear (Settings → API → OAuth Applications)"
    echo "  2. Run: open 'https://pm.5dlabs.ai/oauth/{agent}/authorize'"
    echo "  3. Or manually create 1Password item with client_id, client_secret, developer_token"
    echo ""
    exit 1
fi
