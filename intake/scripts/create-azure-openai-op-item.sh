#!/usr/bin/env bash
# Create 1Password item "Azure OpenAI Foundry (jonathon-0121)" in the Automation vault.
# Requires: op CLI signed in with permission to create items in that vault.
# API key is read from stdin (hidden) so it is not stored in shell history.
set -euo pipefail

VAULT="${1:-Automation}"
TITLE="Azure OpenAI Foundry (jonathon-0121)"
BASE_URL="${AZURE_OPENAI_BASE_URL:-https://jonathon-0121-resource.openai.azure.com/openai/v1}"

echo "Paste Azure OpenAI API key, then Enter (input hidden):" >&2
read -rs KEY
echo >&2
if [ -z "$KEY" ]; then
  echo "Empty key; aborting." >&2
  exit 1
fi

NOTES="$(cat <<'EOF'
OpenAI-compatible v1 base URL is in hostname.

Responses API (gpt-5.4-mini):
https://jonathon-0121-resource.cognitiveservices.azure.com/openai/responses?api-version=2025-04-01-preview

Foundry project:
https://jonathon-0121-resource.services.ai.azure.com/api/projects/jonathon-0121

Rotate: Azure Portal → resource → Keys and Endpoint.
EOF
)"

TMP="$(mktemp)"
trap 'rm -f "$TMP"' EXIT

op item template get "API Credential" | jq \
  --arg title "$TITLE" \
  --arg c "$KEY" \
  --arg h "$BASE_URL" \
  --arg notes "$NOTES" \
  '
  .title = $title
  | .fields |= map(
      if .id == "credential" then .value = $c
      elif .id == "hostname" then .value = $h
      elif .id == "notesPlain" then .value = $notes
      elif .id == "username" then .value = "jonathon-0121-resource"
      else . end
    )
  ' >"$TMP"

op item create --vault "$VAULT" --template "$TMP"
echo "Created 1Password item: $TITLE (vault: $VAULT)" >&2
echo "Uncomment the Azure lines in intake/local.env.op.defaults → intake/local.env.op, then re-run ensure-local-env-op.sh if you use it." >&2
