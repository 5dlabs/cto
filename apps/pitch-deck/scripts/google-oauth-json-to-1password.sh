#!/usr/bin/env bash
# Create a 1Password "API Credential" item from a Google Cloud OAuth client JSON download.
# Secrets are passed via stdin to `op item create` (not argv) to avoid shell history leaks.
#
# Prerequisites:
#   - 1Password CLI: https://developer.1password.com/docs/cli/get-started
#   - jq: brew install jq
#
# Authentication (pick one):
#   A) Service account with "Create and view items" on the target vault:
#        export OP_SERVICE_ACCOUNT_TOKEN
#      Store the token in 1Password (e.g. secure note "1Password Service Account — CTO") and:
#        export OP_SERVICE_ACCOUNT_TOKEN="$(op read 'op://Vault/Item/credential')"
#      (Run that export in a shell already signed in with your human account.)
#
#   B) Interactive user session:
#        eval "$(op signin)"
#
# Usage:
#   ./scripts/google-oauth-json-to-1password.sh /path/to/client_secret_....json
#   VAULT=Private ./scripts/google-oauth-json-to-1password.sh ./client.json
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT"

die() { echo "error: $*" >&2; exit 1; }

command -v op >/dev/null || die "install 1Password CLI (op)"
command -v jq >/dev/null || die "install jq"

JSON_FILE="${1:-}"
[[ -n "$JSON_FILE" && -f "$JSON_FILE" ]] || die "usage: $0 <client_secret_....apps.googleusercontent.com.json>"

VAULT="${VAULT:-Private}"
ITEM_TITLE="${ITEM_TITLE:-Google OAuth — Pitch Deck (5dlabs)}"

CLIENT_ID="$(jq -r '.web.client_id // empty' "$JSON_FILE")"
CLIENT_SECRET="$(jq -r '.web.client_secret // empty' "$JSON_FILE")"
PROJECT_ID="$(jq -r '.web.project_id // empty' "$JSON_FILE")"
[[ -n "$CLIENT_ID" && -n "$CLIENT_SECRET" ]] || die "missing .web.client_id or .web.client_secret in JSON"

NOTES="Google Cloud OAuth 2.0 Web client (pitch deck).
project_id: ${PROJECT_ID}
Source file: $(basename "$JSON_FILE")
Created: $(date -u +%Y-%m-%dT%H:%MZ)"

op whoami >/dev/null 2>&1 || die "not signed in: set OP_SERVICE_ACCOUNT_TOKEN or run eval \"\$(op signin)\""

op item template get "API Credential" | jq \
  --arg title "$ITEM_TITLE" \
  --arg user "$CLIENT_ID" \
  --arg cred "$CLIENT_SECRET" \
  --arg notes "$NOTES" \
  '
  .title = $title |
  (.fields[] | select(.id=="username") | .value) = $user |
  (.fields[] | select(.id=="credential") | .value) = $cred |
  (.fields[] | select(.id=="notesPlain") | .value) = $notes |
  (.fields[] | select(.id=="type") | .value) = "oauth2"
  ' | op item create --vault "$VAULT" - >/dev/null

echo "Created 1Password item: \"$ITEM_TITLE\" in vault \"$VAULT\""
echo ""
echo "Use these op:// references in apps/pitch-deck/local.env.op (copy from google-oauth.env.op.defaults):"
echo "  NEXT_PUBLIC_GOOGLE_OAUTH_CLIENT_ID=op://${VAULT}/${ITEM_TITLE}/username"
echo ""
echo "Cloudflare Pages: set NEXT_PUBLIC_GOOGLE_OAUTH_CLIENT_ID to the public client id:"
echo "  $CLIENT_ID"
