#!/bin/bash
# Refresh Linear OAuth tokens for all CTO agents using client_credentials flow
# 
# Usage: ./refresh-linear-tokens.sh [agent]
#   No args: refresh all agents
#   With arg: refresh specific agent (e.g., ./refresh-linear-tokens.sh Rex)
#
# Prerequisites:
# - 1Password CLI (op) authenticated
# - Client credentials enabled on each Linear OAuth app

set -e

AGENTS=(Atlas Blaze Bolt Cipher Cleo Grizz Morgan Nova Rex Spark Stitch Tap Tess Vex)
SCOPES="read,write,issues:create,comments:create"

refresh_token() {
  local agent=$1
  local item_name="Linear $agent OAuth"
  
  echo -n "🔄 $agent: "
  
  # Get credentials
  CLIENT_ID=$(op read "op://Automation/$item_name/client_id" 2>/dev/null)
  CLIENT_SECRET=$(op read "op://Automation/$item_name/client_secret" 2>/dev/null)
  
  if [[ -z "$CLIENT_ID" || -z "$CLIENT_SECRET" ]]; then
    echo "❌ Missing credentials in 1Password"
    return 1
  fi
  
  # Request new token
  RESPONSE=$(curl -s -X POST https://api.linear.app/oauth/token \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "grant_type=client_credentials" \
    -d "scope=$SCOPES" \
    -d "client_id=$CLIENT_ID" \
    -d "client_secret=$CLIENT_SECRET")
  
  TOKEN=$(echo "$RESPONSE" | jq -r '.access_token // empty')
  ERROR=$(echo "$RESPONSE" | jq -r '.error_description // .error // empty')
  
  if [[ -n "$TOKEN" ]]; then
    # Update 1Password
    op item edit "$item_name" "developer_token=$TOKEN" --vault Automation >/dev/null 2>&1
    EXPIRES_IN=$(echo "$RESPONSE" | jq -r '.expires_in')
    DAYS=$((EXPIRES_IN / 86400))
    echo "✅ Token refreshed (expires in ${DAYS} days)"
    return 0
  else
    echo "❌ Failed: $ERROR"
    return 1
  fi
}

# Main
if [[ -n "$1" ]]; then
  # Single agent
  refresh_token "$1"
else
  # All agents
  echo "Refreshing Linear OAuth tokens for all CTO agents..."
  echo "Using client_credentials flow (30-day tokens, no user login required)"
  echo ""
  
  SUCCESS=0
  FAILED=0
  
  for agent in "${AGENTS[@]}"; do
    if refresh_token "$agent"; then
      ((SUCCESS++))
    else
      ((FAILED++))
    fi
  done
  
  echo ""
  echo "Done: $SUCCESS succeeded, $FAILED failed"
fi
