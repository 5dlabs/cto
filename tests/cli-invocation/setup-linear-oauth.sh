#!/usr/bin/env bash
# Audit Linear client credentials and runtime tokens for CTO agents.
#
# This script treats 1Password as the source of truth for per-agent
# client_id/client_secret, and Kubernetes as the source of truth for runtime
# access tokens. Optionally, it asks PM to mint missing or stale runtime tokens.
#
# Usage:
#   ./setup-linear-oauth.sh            # Audit all agents
#   ./setup-linear-oauth.sh --mint     # Audit and ask PM to mint when needed
#   ./setup-linear-oauth.sh bolt       # Audit a single agent

set -euo pipefail

PM_BASE_URL="${PM_BASE_URL:-https://pm.5dlabs.ai}"
NAMESPACE="${NAMESPACE:-cto}"
DO_MINT=false
SPECIFIC_AGENT=""

for arg in "$@"; do
    case "$arg" in
        --mint|--refresh) DO_MINT=true ;;
        *)                SPECIFIC_AGENT="$arg" ;;
    esac
done

ALL_AGENTS=(angie atlas blaze bolt cipher cleo grizz morgan nova pixel rex spark stitch tap tess vex)

if [[ -n "$SPECIFIC_AGENT" ]]; then
    AGENTS=("$SPECIFIC_AGENT")
else
    AGENTS=("${ALL_AGENTS[@]}")
fi

if ! op whoami >/dev/null 2>&1; then
    echo "❌ Not signed in to 1Password"
    exit 1
fi

get_oauth_item() {
    local agent="$1"
    echo "Linear ${agent^} OAuth"
}

read_runtime_token() {
    local agent="$1"
    kubectl get secret "linear-app-${agent}" -n "${NAMESPACE}" \
        -o jsonpath='{.data.access_token}' 2>/dev/null | base64 -d 2>/dev/null || true
}

verify_token() {
    local token="$1"
    curl -s -w "\n%{http_code}" \
        -H "Authorization: Bearer ${token}" \
        -H "Content-Type: application/json" \
        -d '{"query":"{ viewer { id name email } }"}' \
        "https://api.linear.app/graphql" 2>/dev/null || echo -e "\n000"
}

mint_via_pm() {
    local agent="$1"
    local response
    local http_code

    response=$(curl -sS -w "\n%{http_code}" -X POST \
        "${PM_BASE_URL}/oauth/mint/${agent}" 2>/dev/null || echo -e "\n000")
    http_code=$(echo "$response" | tail -1)

    [[ "$http_code" == "200" ]]
}

echo "🔐 Linear Client Credentials + Runtime Token Audit"
echo "=================================================="
echo "Agents: ${#AGENTS[@]}"
echo "Namespace: ${NAMESPACE}"
echo "PM base URL: ${PM_BASE_URL}"
if [[ "$DO_MINT" == true ]]; then
    echo "PM mint on missing/invalid runtime tokens: enabled"
fi
echo ""

VALID=0
MINTED=0
MISSING_CREDS=0
MISSING_RUNTIME=0
INVALID_RUNTIME=0
NEEDS_ATTENTION=()

printf "%-10s %-12s %-14s %-40s\n" "Agent" "Creds" "Runtime" "Details"
printf "%-10s %-12s %-14s %-40s\n" "-----" "-----" "-------" "-------"

for agent in "${AGENTS[@]}"; do
    item_name="$(get_oauth_item "$agent")"
    creds_status="missing"
    runtime_status="missing"
    details=""

    client_id=""
    client_secret=""

    if op item get "$item_name" --vault "Automation" >/dev/null 2>&1; then
        client_id="$(op item get "$item_name" --vault "Automation" --fields label=client_id --reveal 2>/dev/null || true)"
        client_secret="$(op item get "$item_name" --vault "Automation" --fields label=client_secret --reveal 2>/dev/null || true)"
    fi

    if [[ -n "$client_id" && -n "$client_secret" ]]; then
        creds_status="ready"
    else
        creds_status="missing"
        ((MISSING_CREDS++))
    fi

    runtime_token="$(read_runtime_token "$agent")"
    if [[ -z "$runtime_token" && "$DO_MINT" == true && "$creds_status" == "ready" ]]; then
        if mint_via_pm "$agent"; then
            runtime_token="$(read_runtime_token "$agent")"
            [[ -n "$runtime_token" ]] && runtime_status="minted"
        fi
    fi

    if [[ -n "$runtime_token" ]]; then
        response="$(verify_token "$runtime_token")"
        http_code="$(echo "$response" | tail -1)"
        body="$(echo "$response" | sed '$d')"
        if [[ "$http_code" == "200" ]] && ! echo "$body" | jq -e '.errors' >/dev/null 2>&1; then
            user_name="$(echo "$body" | jq -r '.data.viewer.name // "unknown"')"
            if [[ "$runtime_status" == "minted" ]]; then
                details="fresh runtime token for ${user_name}"
                ((MINTED++))
            else
                runtime_status="valid"
                details="runtime token for ${user_name}"
                ((VALID++))
            fi
        else
            if [[ "$DO_MINT" == true && "$creds_status" == "ready" ]]; then
                if mint_via_pm "$agent"; then
                    runtime_token="$(read_runtime_token "$agent")"
                    response="$(verify_token "$runtime_token")"
                    http_code="$(echo "$response" | tail -1)"
                    body="$(echo "$response" | sed '$d')"
                    if [[ "$http_code" == "200" ]] && ! echo "$body" | jq -e '.errors' >/dev/null 2>&1; then
                        user_name="$(echo "$body" | jq -r '.data.viewer.name // "unknown"')"
                        runtime_status="minted"
                        details="fresh runtime token for ${user_name}"
                        ((MINTED++))
                    else
                        runtime_status="invalid"
                        details="runtime token still rejected"
                        ((INVALID_RUNTIME++))
                    fi
                else
                    runtime_status="invalid"
                    details="PM mint failed"
                    ((INVALID_RUNTIME++))
                fi
            else
                runtime_status="invalid"
                details="runtime token rejected"
                ((INVALID_RUNTIME++))
            fi
        fi
    else
        runtime_status="missing"
        details="no runtime token in linear-app-${agent}"
        ((MISSING_RUNTIME++))
    fi

    if [[ "$creds_status" != "ready" || ( "$runtime_status" != "valid" && "$runtime_status" != "minted" ) ]]; then
        NEEDS_ATTENTION+=("$agent")
    fi

    printf "%-10s %-12s %-14s %-40s\n" "$agent" "$creds_status" "$runtime_status" "$details"
done

echo ""
echo "Summary"
echo "  Valid runtime tokens: ${VALID}"
echo "  Minted during audit:  ${MINTED}"
echo "  Missing credentials:  ${MISSING_CREDS}"
echo "  Missing runtime:      ${MISSING_RUNTIME}"
echo "  Invalid runtime:      ${INVALID_RUNTIME}"
echo ""

if [[ ${#NEEDS_ATTENTION[@]} -gt 0 ]]; then
    echo "Needs attention: ${NEEDS_ATTENTION[*]}"
    echo ""
    echo "Preferred repair order:"
    echo "  1. Ensure 1Password item 'Linear {Agent} OAuth' has client_id + client_secret"
    echo "  2. Ask PM to mint via POST ${PM_BASE_URL}/oauth/mint/{agent}"
    echo "  3. Re-run ./setup-linear-oauth.sh --mint"
    echo "  4. Use browser auth only for true exception apps"
    exit 1
fi
