#!/usr/bin/env bash
# Create a 1Password "API Credential" item from a Google Cloud OAuth client JSON download.
# Secrets are passed via stdin to `op item create` (not argv) to avoid shell history leaks.
#
# Prerequisites:
#   - 1Password CLI: https://developer.1password.com/docs/cli/get-started
#   - jq: brew install jq
#
# Authentication (pick one):
#
#   A) You + biometrics / password (most common). Do NOT set OP_SERVICE_ACCOUNT_TOKEN.
#        unset OP_SERVICE_ACCOUNT_TOKEN   # if a bad export is still in the shell
#        eval "$(op signin)"              # may prompt Touch ID / password
#      Then run this script. The AI assistant cannot run this for you: `op` must run on your Mac.
#
#   B) Service account token (CI / automation only). Needs vault grants in 1Password admin.
#        export OP_SERVICE_ACCOUNT_TOKEN="..."   # real token, not op://YOUR_VAULT placeholder
#      Or: export OP_SERVICE_ACCOUNT_TOKEN="$(op read 'op://RealVault/Real Item/credential')"
#      after (A) once, to copy the token into an item — use real vault and item names from `op vault list` / `op item list`.
#
# Usage:
#   ./scripts/google-oauth-json-to-1password.sh /path/to/client_secret_....json
#   VAULT=Private ./scripts/google-oauth-json-to-1password.sh ./client.json
#
# If you omit the path, the script tries (in order):
#   1) $GOOGLE_OAUTH_JSON (if set and the file exists)
#   2) $HOME/Downloads/client_secret_932249086524-gos1d3eidio1ktdq8q9jp0jjkrt54a67.apps.googleusercontent.com.json
#      (Pitch Deck OAuth client — update DEFAULT_JSON in this script if Google issues a new download name.)
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT"

die() { echo "error: $*" >&2; exit 1; }

command -v op >/dev/null || die "install 1Password CLI (op)"
command -v jq >/dev/null || die "install jq"

# Default matches Google Cloud “Download JSON” for the Pitch Deck web client (see header).
DEFAULT_JSON="$HOME/Downloads/client_secret_932249086524-gos1d3eidio1ktdq8q9jp0jjkrt54a67.apps.googleusercontent.com.json"

JSON_FILE="${1:-}"
if [[ -z "$JSON_FILE" && -n "${GOOGLE_OAUTH_JSON:-}" && -f "${GOOGLE_OAUTH_JSON}" ]]; then
  JSON_FILE="${GOOGLE_OAUTH_JSON}"
elif [[ -z "$JSON_FILE" && -f "$DEFAULT_JSON" ]]; then
  JSON_FILE="$DEFAULT_JSON"
fi

if [[ -z "$JSON_FILE" || ! -f "$JSON_FILE" ]]; then
  echo "error: pass the path to Google’s client_secret JSON, or put it at:" >&2
  echo "  $DEFAULT_JSON" >&2
  echo "  (or set GOOGLE_OAUTH_JSON to the file path)" >&2
  die "usage: $0 [<client_secret_....apps.googleusercontent.com.json>]"
fi

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

if ! op whoami >/dev/null 2>&1; then
  echo "error: 1Password CLI is not authenticated." >&2
  echo "  If you use Touch ID: run  unset OP_SERVICE_ACCOUNT_TOKEN  then  eval \"\$(op signin)\"" >&2
  echo "  If OP_SERVICE_ACCOUNT_TOKEN is set to a placeholder, unset it and sign in again." >&2
  exit 1
fi

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
