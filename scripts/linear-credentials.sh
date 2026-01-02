#!/usr/bin/env bash
set -euo pipefail

ENV_FILE="${ENV_FILE:-$(dirname "$0")/../linear-agents.env.template}"
AGENTS="morgan rex blaze grizz nova tap spark cleo cipher tess atlas bolt"

store_all() {
  echo "=== Storing all Linear agent credentials in OpenBao ==="
  # shellcheck disable=SC1090
  source "$ENV_FILE"
  
  for agent in $AGENTS; do
    upper=$(echo "$agent" | tr '[:lower:]' '[:upper:]')
    eval "client_id=\$${upper}_CLIENT_ID"
    eval "client_secret=\$${upper}_CLIENT_SECRET"
    eval "webhook_secret=\$${upper}_WEBHOOK_SECRET"
    
    # Validate credentials exist
    if [[ -z "$client_id" || -z "$client_secret" || -z "$webhook_secret" ]]; then
      echo "ERROR: Missing credentials for $agent"
      exit 1
    fi
    
    echo "Storing linear-app-$agent..."
    bao kv put "linear-app-$agent" \
      client_id="$client_id" \
      client_secret="$client_secret" \
      webhook_secret="$webhook_secret"
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
  # shellcheck disable=SC1090
  source "$ENV_FILE"
  
  local base_url="https://linear.app/oauth/authorize"
  local redirect_uri="https://cto.5dlabs.ai/oauth/callback"
  local scope="read,write,app:assignable,app:mentionable"
  
  for agent in $AGENTS; do
    upper=$(echo "$agent" | tr '[:lower:]' '[:upper:]')
    eval "client_id=\$${upper}_CLIENT_ID"
    echo ""
    echo "=== ${agent^} ==="
    echo "${base_url}?client_id=${client_id}&redirect_uri=${redirect_uri}&response_type=code&scope=${scope}&actor=app"
  done
}

validate() {
  echo "=== Validating env file format ==="
  # shellcheck disable=SC1090
  source "$ENV_FILE"
  
  local errors=0
  for agent in $AGENTS; do
    upper=$(echo "$agent" | tr '[:lower:]' '[:upper:]')
    eval "client_id=\$${upper}_CLIENT_ID"
    eval "client_secret=\$${upper}_CLIENT_SECRET"
    eval "webhook_secret=\$${upper}_WEBHOOK_SECRET"
    
    echo -n "$agent: "
    
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
  store)    store_all ;;
  verify)   verify_all ;;
  export)   export_env ;;
  urls)     oauth_urls ;;
  validate) validate ;;
  *)
    echo "Usage: $0 {store|verify|export|urls|validate}"
    echo "  store    - Store all credentials from env file to OpenBao"
    echo "  verify   - Check all credentials exist in OpenBao"
    echo "  export   - Export credentials from OpenBao to env file"
    echo "  urls     - Generate OAuth installation URLs"
    echo "  validate - Validate env file format without storing"
    exit 1
    ;;
esac
