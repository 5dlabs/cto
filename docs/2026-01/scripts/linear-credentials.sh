#!/usr/bin/env bash
# Linear agent credential management script
# 
# Credentials are stored in OpenBao at path: linear-app-{agent}
# Each agent has: client_id, client_secret, webhook_secret
#
# To add new credentials, use: bao kv put linear-app-{agent} client_id=... client_secret=... webhook_secret=...
set -euo pipefail

AGENTS="morgan rex blaze grizz nova tap spark cleo cipher tess atlas bolt"

store_interactive() {
  echo "=== Store Linear agent credentials interactively ==="
  echo ""
  echo "For each agent, you'll be prompted for credentials."
  echo "Get these from: https://linear.app/settings/api/applications"
  echo ""
  
  for agent in $AGENTS; do
    echo "--- ${agent^} ---"
    read -rp "Client ID (32 hex chars): " client_id
    read -rp "Client Secret (32 hex chars): " client_secret
    read -rp "Webhook Secret (lin_wh_...): " webhook_secret
    
    # Validate
    if [[ ! "$client_id" =~ ^[a-f0-9]{32}$ ]]; then
      echo "ERROR: Invalid client_id format"
      exit 1
    fi
    
    echo "Storing linear-app-$agent..."
    bao kv put "linear-app-$agent" \
      client_id="$client_id" \
      client_secret="$client_secret" \
      webhook_secret="$webhook_secret"
    echo ""
  done
  
  echo "=== All 12 agents stored ==="
}

verify_all() {
  echo "=== Verifying all Linear agent credentials in OpenBao ==="
  local failed=0
  for agent in $AGENTS; do
    echo -n "$agent: "
    if bao kv get "linear-app-$agent" &>/dev/null; then
      echo "OK"
    else
      echo "MISSING"
      failed=1
    fi
  done
  
  if [[ $failed -eq 1 ]]; then
    echo "=== Some credentials are missing! ==="
    exit 1
  fi
  echo "=== All 12 agents verified ==="
}

export_env() {
  echo "=== Exporting credentials from OpenBao to env file ==="
  local output_file="${ENV_FILE}.restored"
  
  {
    echo "# Exported from OpenBao $(date)"
    echo "# Linear Agent OAuth Credentials"
    echo ""
  } > "$output_file"
  
  for agent in $AGENTS; do
    upper=$(echo "$agent" | tr '[:lower:]' '[:upper:]')
    data=$(bao kv get -format=json "linear-app-$agent" 2>/dev/null | jq -r '.data.data')
    
    {
      echo "# ----- ${agent^} -----"
      echo "${upper}_CLIENT_ID=$(echo "$data" | jq -r '.client_id')"
      echo "${upper}_CLIENT_SECRET=$(echo "$data" | jq -r '.client_secret')"
      echo "${upper}_WEBHOOK_SECRET=$(echo "$data" | jq -r '.webhook_secret')"
      echo ""
    } >> "$output_file"
  done
  
  echo "=== Exported to $output_file ==="
}

oauth_urls() {
  echo "=== OAuth Installation URLs ==="
  echo ""
  echo "Fetching client IDs from OpenBao..."
  echo ""
  
  local base_url="https://linear.app/oauth/authorize"
  local redirect_uri="https://cto.5dlabs.ai/oauth/callback"
  local scope="read,write,app:assignable,app:mentionable"
  
  for agent in $AGENTS; do
    client_id=$(bao kv get -format=json "linear-app-$agent" 2>/dev/null | jq -r '.data.data.client_id' 2>/dev/null || echo "")
    
    if [[ -n "$client_id" && "$client_id" != "null" ]]; then
      echo "=== ${agent^} ==="
      echo "${base_url}?client_id=${client_id}&redirect_uri=${redirect_uri}&response_type=code&scope=${scope}&actor=app"
      echo ""
    else
      echo "=== ${agent^} === (not configured)"
      echo ""
    fi
  done
}

validate() {
  echo "=== Validating credentials in OpenBao ==="
  
  local errors=0
  for agent in $AGENTS; do
    echo -n "$agent: "
    
    data=$(bao kv get -format=json "linear-app-$agent" 2>/dev/null | jq -r '.data.data' 2>/dev/null || echo "{}")
    client_id=$(echo "$data" | jq -r '.client_id // ""')
    client_secret=$(echo "$data" | jq -r '.client_secret // ""')
    webhook_secret=$(echo "$data" | jq -r '.webhook_secret // ""')
    
    # Check client_id format (32 hex chars)
    if [[ ! "$client_id" =~ ^[a-f0-9]{32}$ ]]; then
      echo "ERROR: Invalid client_id format"
      errors=1
      continue
    fi
    
    # Check client_secret format (32 hex chars)
    if [[ ! "$client_secret" =~ ^[a-f0-9]{32}$ ]]; then
      echo "ERROR: Invalid client_secret format"
      errors=1
      continue
    fi
    
    # Check webhook_secret format (starts with lin_wh_)
    if [[ ! "$webhook_secret" =~ ^lin_wh_ ]]; then
      echo "ERROR: Invalid webhook_secret format (should start with lin_wh_)"
      errors=1
      continue
    fi
    
    echo "OK"
  done
  
  if [[ $errors -eq 1 ]]; then
    echo "=== Validation failed! ==="
    exit 1
  fi
  echo "=== All 36 credentials validated ==="
}

case "${1:-}" in
  store)    store_interactive ;;
  verify)   verify_all ;;
  export)   export_env ;;
  urls)     oauth_urls ;;
  validate) validate ;;
  *)
    echo "Usage: $0 {store|verify|export|urls|validate}"
    echo "  store    - Store credentials interactively to OpenBao"
    echo "  verify   - Check all credentials exist in OpenBao"
    echo "  export   - Export credentials from OpenBao to env file"
    echo "  urls     - Generate OAuth installation URLs (from OpenBao)"
    echo "  validate - Validate credential format in OpenBao"
    exit 1
    ;;
esac
