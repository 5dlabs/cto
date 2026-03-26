#!/usr/bin/env bash
set -euo pipefail

LINEAR_URL="${LINEAR_BRIDGE_URL:-http://127.0.0.1:3100}"
DISCORD_URL="${DISCORD_BRIDGE_URL:-http://127.0.0.1:3200}"
PR_URL="${PR_URL:-}"
LINEAR_ISSUE_COUNT="${LINEAR_ISSUE_COUNT:-0}"
ELICITATION_ID="${ELICITATION_ID:-}"
SESSION_ID="${SESSION_ID:-}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --linear-url) LINEAR_URL="$2"; shift 2 ;;
    --discord-url) DISCORD_URL="$2"; shift 2 ;;
    --pr-url) PR_URL="$2"; shift 2 ;;
    --linear-issue-count) LINEAR_ISSUE_COUNT="$2"; shift 2 ;;
    --elicitation-id) ELICITATION_ID="$2"; shift 2 ;;
    --session-id) SESSION_ID="$2"; shift 2 ;;
    *)
      echo "unknown arg: $1" >&2
      exit 1
      ;;
  esac
done

PR_OK=false
if [[ "$PR_URL" =~ ^https://github\.com/.+/pull/[0-9]+$ ]]; then
  PR_OK=true
fi

LINEAR_ISSUES_OK=false
if printf '%s' "$LINEAR_ISSUE_COUNT" | jq -e 'tonumber > 0' >/dev/null 2>&1; then
  LINEAR_ISSUES_OK=true
fi

DECISIONS_JSON="$(curl -fsS "${LINEAR_URL%/}/history/decisions?limit=20" 2>/dev/null || printf '%s' '{"decisions":[]}' )"
WAITING_JSON="$(curl -fsS "${LINEAR_URL%/}/history/waiting?limit=20" 2>/dev/null || printf '%s' '{"waiting":[]}' )"
SESSIONS_JSON="$(curl -fsS "${LINEAR_URL%/}/history/sessions?limit=20" 2>/dev/null || printf '%s' '{"sessions":[]}' )"
DISCORD_DECISIONS_JSON="$(curl -fsS "${DISCORD_URL%/}/history/decisions?limit=20" 2>/dev/null || printf '%s' '{"decisions":[]}' )"
LINEAR_DESIGN_JSON="$(curl -fsS "${LINEAR_URL%/}/history/design?limit=20" 2>/dev/null || printf '%s' '{"design":[]}' )"
DISCORD_DESIGN_JSON="$(curl -fsS "${DISCORD_URL%/}/history/design?limit=20" 2>/dev/null || printf '%s' '{"design":[]}' )"

AUDIT_JSON='{}'
if [[ -n "$ELICITATION_ID" ]]; then
  AUDIT_JSON="$(curl -fsS "${LINEAR_URL%/}/history/decision-audit?elicitation_id=$(python3 - <<'PY'
import urllib.parse, os
print(urllib.parse.quote(os.environ["ELICITATION_ID"]))
PY
)" 2>/dev/null || printf '%s' '{"audit":{}}')"
fi

jq -nc \
  --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg linear_url "$LINEAR_URL" \
  --arg discord_url "$DISCORD_URL" \
  --arg pr_url "$PR_URL" \
  --argjson pr_ok "$PR_OK" \
  --arg linear_issue_count "$LINEAR_ISSUE_COUNT" \
  --argjson linear_issues_ok "$LINEAR_ISSUES_OK" \
  --arg session_id "$SESSION_ID" \
  --argjson linear_decisions "$DECISIONS_JSON" \
  --argjson linear_waiting "$WAITING_JSON" \
  --argjson linear_sessions "$SESSIONS_JSON" \
  --argjson discord_decisions "$DISCORD_DECISIONS_JSON" \
  --argjson linear_design "$LINEAR_DESIGN_JSON" \
  --argjson discord_design "$DISCORD_DESIGN_JSON" \
  --argjson audit "$AUDIT_JSON" \
  '{
    generated_at: $generated_at,
    session_id: $session_id,
    gate_evidence: {
      pr_url: $pr_url,
      pr_ok: $pr_ok,
      linear_issue_count: ($linear_issue_count | tonumber? // 0),
      linear_issues_ok: $linear_issues_ok
    },
    persistence_evidence: {
      linear: {
        decisions: ($linear_decisions.decisions // []),
        waiting: ($linear_waiting.waiting // []),
        sessions: ($linear_sessions.sessions // []),
        design: ($linear_design.design // [])
      },
      discord: {
        decisions: ($discord_decisions.decisions // []),
        design: ($discord_design.design // [])
      },
      audit: ($audit.audit // {})
    }
  }'
