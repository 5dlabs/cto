#!/usr/bin/env bash
# Print Linear team UUID for the team key in cto-config.json (e.g. CTOPA → real id).
# Same resolution as the Linear UI / API — no manual copy-paste from the browser.
#
# Requires: LINEAR_API_KEY in environment (OAuth access token or lin_api_*).
# Override key: LINEAR_TEAM_KEY=XXX or pass first arg.
#
# Usage:
#   LINEAR_API_KEY=… ./intake/scripts/linear-resolve-team-id.sh
#   op run --env-file=intake/local.env.op -- ./intake/scripts/linear-resolve-team-id.sh

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
CFG="$ROOT/cto-config.json"

[[ -n "${LINEAR_API_KEY:-}" ]] || {
  echo "linear-resolve-team-id.sh: LINEAR_API_KEY required (use OAuth token or lin_api key)" >&2
  exit 1
}

TEAM_KEY="${1:-${LINEAR_TEAM_KEY:-}}"
if [[ -z "$TEAM_KEY" ]]; then
  TEAM_KEY="$(jq -r '.defaults.linear.teamId // empty' "$CFG")"
fi
[[ -n "$TEAM_KEY" ]] || {
  echo "linear-resolve-team-id.sh: no team key (set LINEAR_TEAM_KEY or defaults.linear.teamId in cto-config.json)" >&2
  exit 1
}

# UUID already
if [[ "$TEAM_KEY" == *-* ]] && [[ ${#TEAM_KEY} -gt 20 ]]; then
  echo "$TEAM_KEY"
  exit 0
fi

if [[ "$LINEAR_API_KEY" == lin_api_* ]]; then
  AUTH_HDR="$LINEAR_API_KEY"
else
  AUTH_HDR="Bearer ${LINEAR_API_KEY}"
fi

payload='{"query":"query { teams { nodes { id key name } } }"}'
json=$(curl -sS --max-time 30 https://api.linear.app/graphql \
  -H "Content-Type: application/json" \
  -H "Authorization: ${AUTH_HDR}" \
  -d "$payload")

tid=$(echo "$json" | jq -r --arg k "$TEAM_KEY" '
  ($k|ascii_downcase) as $kl |
  .data.teams.nodes[]? | select((.key|ascii_downcase) == $kl) | .id
' | head -1)

if [[ -z "$tid" || "$tid" == "null" ]]; then
  echo "linear-resolve-team-id.sh: team key '$TEAM_KEY' not found. API response:" >&2
  echo "$json" | jq . 2>/dev/null || echo "$json" >&2
  exit 1
fi

echo "$tid"
