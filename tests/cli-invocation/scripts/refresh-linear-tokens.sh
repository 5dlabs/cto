#!/usr/bin/env bash
# Mint Linear runtime access tokens for CTO agents via PM.
#
# Usage: ./refresh-linear-tokens.sh [agent]
#   No args: mint for every configured PM agent via /oauth/mint-all
#   With arg: mint for a specific agent (e.g. ./refresh-linear-tokens.sh bolt)
#
# Notes:
# - 1Password is the source of truth for client_id/client_secret only.
# - PM is the token broker.
# - Runtime access tokens live in Kubernetes secrets: linear-app-{agent}.

set -euo pipefail

PM_BASE_URL="${PM_BASE_URL:-https://pm.5dlabs.ai}"

mint_one() {
  local agent="$1"
  local response
  local http_code
  local body

  response=$(curl -sS -w "\n%{http_code}" -X POST \
    "${PM_BASE_URL}/oauth/mint/${agent}" 2>/dev/null || echo -e "\n000")

  http_code=$(echo "$response" | tail -1)
  body=$(echo "$response" | sed '$d')

  if [[ "$http_code" == "200" ]]; then
    local expires_in
    local days
    expires_in=$(echo "$body" | jq -r '.expires_in // 0')
    days=$((expires_in / 86400))
    echo "✅ ${agent}: minted via PM (expires in ${days} days)"
    return 0
  fi

  local error
  error=$(echo "$body" | jq -r '.error // "Unknown error"' 2>/dev/null || echo "Unknown error")
  echo "❌ ${agent}: ${error} (HTTP ${http_code})"
  return 1
}

if [[ $# -gt 0 ]]; then
  mint_one "$1"
  exit $?
fi

echo "Minting Linear runtime tokens via PM..."
echo "PM base URL: ${PM_BASE_URL}"
echo ""

response=$(curl -sS -w "\n%{http_code}" -X POST \
  "${PM_BASE_URL}/oauth/mint-all" 2>/dev/null || echo -e "\n000")

http_code=$(echo "$response" | tail -1)
body=$(echo "$response" | sed '$d')

if [[ "$http_code" != "200" ]]; then
  error=$(echo "$body" | jq -r '.error // "Unknown error"' 2>/dev/null || echo "Unknown error")
  echo "❌ Bulk mint failed: ${error} (HTTP ${http_code})"
  exit 1
fi

echo "$body" | jq -r '
  .results[]
  | if .status == "minted" then
      "✅ \(.agent): minted"
    elif .status == "skipped" then
      "⏭️  \(.agent): \(.reason)"
    else
      "❌ \(.agent): \(.error // "unknown error")"
    end
'

echo ""
echo "$body" | jq -r '"Summary: minted=\(.counts.minted) skipped=\(.counts.skipped) failed=\(.counts.failed)"'
