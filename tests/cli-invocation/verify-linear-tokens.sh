#!/usr/bin/env bash
# Verify runtime Linear tokens from Kubernetes.
#
# Tests that each agent's current runtime access token in linear-app-{agent}
# can access the Linear API. Optionally asks PM to mint a fresh token first.
#
# Usage:
#   ./verify-linear-tokens.sh             # Verify all configured agents
#   ./verify-linear-tokens.sh bolt        # Verify a single agent
#   ./verify-linear-tokens.sh --mint      # Mint before verification when needed
#
# Requirements:
#   - kubectl
#   - curl
#   - jq

set -euo pipefail

NAMESPACE="${NAMESPACE:-cto}"
PM_BASE_URL="${PM_BASE_URL:-https://pm.5dlabs.ai}"
DO_MINT=false
AGENTS_TO_TEST=()

for arg in "$@"; do
    case "$arg" in
        --mint|--refresh)
            DO_MINT=true
            ;;
        *)
            AGENTS_TO_TEST+=("$arg")
            ;;
    esac
done

ALL_AGENTS=(angie atlas blaze bolt cipher cleo grizz morgan nova pixel rex spark stitch tap tess vex)

if [[ ${#AGENTS_TO_TEST[@]} -gt 0 ]]; then
    AGENTS=("${AGENTS_TO_TEST[@]}")
else
    AGENTS=("${ALL_AGENTS[@]}")
fi

read_runtime_token() {
    local agent="$1"
    kubectl get secret "linear-app-${agent}" -n "${NAMESPACE}" \
        -o jsonpath='{.data.access_token}' 2>/dev/null | base64 -d 2>/dev/null || true
}

mint_via_pm() {
    local agent="$1"
    local response
    local http_code
    local body

    response=$(curl -sS -w "\n%{http_code}" -X POST \
        "${PM_BASE_URL}/oauth/mint/${agent}" 2>/dev/null || echo -e "\n000")
    http_code=$(echo "$response" | tail -1)
    body=$(echo "$response" | sed '$d')

    if [[ "$http_code" == "200" ]]; then
        return 0
    fi

    local error
    error=$(echo "$body" | jq -r '.error // "Unknown error"' 2>/dev/null || echo "Unknown error")
    echo "   ❌ PM mint failed: ${error} (HTTP ${http_code})"
    return 1
}

verify_token() {
    local token="$1"
    curl -s -w "\n%{http_code}" \
        -H "Authorization: Bearer ${token}" \
        -H "Content-Type: application/json" \
        -d '{"query":"{ viewer { id name email } }"}' \
        "https://api.linear.app/graphql" 2>/dev/null || echo -e "\n000"
}

echo "🔐 Verifying Linear runtime tokens for ${#AGENTS[@]} agent(s)..."
echo "   Namespace: ${NAMESPACE}"
if [[ "$DO_MINT" == true ]]; then
    echo "   PM mint on failure: enabled (${PM_BASE_URL})"
fi
echo ""

PASSED=0
FAILED=0
SKIPPED=0

for agent in "${AGENTS[@]}"; do
    echo "🔍 Testing: ${agent}"

    if ! kubectl get secret "linear-app-${agent}" -n "${NAMESPACE}" >/dev/null 2>&1; then
        echo "   ⏭️  No runtime secret found: linear-app-${agent}"
        if [[ "$DO_MINT" == true ]]; then
            echo "   🔄 Asking PM to mint a token..."
            if mint_via_pm "$agent"; then
                echo "   ✓ PM reported mint success"
            else
                ((FAILED++))
                echo ""
                continue
            fi
        else
            ((SKIPPED++))
            echo ""
            continue
        fi
    fi

    access_token="$(read_runtime_token "$agent")"
    if [[ -z "$access_token" ]]; then
        if [[ "$DO_MINT" == true ]]; then
            echo "   🔄 No runtime token present, asking PM to mint..."
            if mint_via_pm "$agent"; then
                access_token="$(read_runtime_token "$agent")"
            fi
        fi
    fi

    if [[ -z "$access_token" ]]; then
        echo "   ❌ No runtime access token found in linear-app-${agent}"
        ((FAILED++))
        echo ""
        continue
    fi

    response="$(verify_token "$access_token")"
    http_code="$(echo "$response" | tail -1)"
    body="$(echo "$response" | sed '$d')"

    if [[ "$http_code" == "200" ]] && ! echo "$body" | jq -e '.errors' >/dev/null 2>&1; then
        user_name="$(echo "$body" | jq -r '.data.viewer.name // "unknown"')"
        user_email="$(echo "$body" | jq -r '.data.viewer.email // "unknown"')"
        echo "   ✓ Token valid (user: ${user_name} <${user_email}>)"
        ((PASSED++))
        echo ""
        continue
    fi

    if [[ "$http_code" == "401" && "$DO_MINT" == true ]]; then
        echo "   ⚠️  Runtime token rejected (401), asking PM to mint a fresh token..."
        if mint_via_pm "$agent"; then
            access_token="$(read_runtime_token "$agent")"
            if [[ -n "$access_token" ]]; then
                response="$(verify_token "$access_token")"
                http_code="$(echo "$response" | tail -1)"
                body="$(echo "$response" | sed '$d')"
                if [[ "$http_code" == "200" ]] && ! echo "$body" | jq -e '.errors' >/dev/null 2>&1; then
                    user_name="$(echo "$body" | jq -r '.data.viewer.name // "unknown"')"
                    user_email="$(echo "$body" | jq -r '.data.viewer.email // "unknown"')"
                    echo "   ✓ Fresh token valid (user: ${user_name} <${user_email}>)"
                    ((PASSED++))
                    echo ""
                    continue
                fi
            fi
        fi
    fi

    if [[ "$http_code" == "401" ]]; then
        echo "   ❌ Token invalid or expired (401)"
    else
        echo "   ❌ API call failed (HTTP ${http_code})"
    fi
    ((FAILED++))
    echo ""
done

echo "=============================================="
echo "📊 Summary"
echo "   Passed:  ${PASSED}"
echo "   Failed:  ${FAILED}"
echo "   Skipped: ${SKIPPED}"
echo "=============================================="
echo ""
echo "📝 Notes:"
echo "   Runtime access tokens live in Kubernetes secrets, not 1Password."
echo "   1Password should store client_id/client_secret only."
echo "   Preferred repair path is POST ${PM_BASE_URL}/oauth/mint/{agent}."

if [[ ${FAILED} -gt 0 ]]; then
    exit 1
fi
