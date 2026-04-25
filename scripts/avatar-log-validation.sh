#!/usr/bin/env bash
# avatar-log-validation.sh — repeatable Datadog gate for the Morgan avatar
# (EchoMimic / /echo-turn) before remote readiness tests or deploys.
#
# Modes:
#   validate (default)  Run a one-shot Datadog tail over the recent window,
#                       scan output for known blocker patterns, exit non-zero
#                       if any are present.
#   tail                Stream Datadog logs continuously (Ctrl-C to stop).
#
# Configuration (env):
#   AVATAR_DD_TAIL      Path to the session-local dd-avatar-tail.sh helper.
#                       If unset, the script auto-discovers the most recently
#                       modified copy under ~/.copilot/session-state/*/files/.
#   AVATAR_DD_FROM      Datadog lower bound (default: now-15m).
#   AVATAR_DD_LIMIT     Max events per request (default: 200).
#   AVATAR_DD_QUERY     Override the default avatar query (rarely needed).
#
# Notes:
#   * The underlying helper reads DD creds from the OVH cluster Secret via
#     1Password (op) — no secrets are ever printed by this wrapper.
#   * Output is filtered through a narrow redactor that only collapses
#     clearly secret-shaped tokens (sk-*, JWTs, Bearer/Token/Api-Key values,
#     long pure hex, long pure base64). URL paths, hosts, query strings, and
#     HTTP status codes are preserved so blocker patterns like
#     /animate.*5xx, openai.*401, and elevenlabs.*401 still match.
#
# See docs/avatar/validation.md for the full gate procedure.

set -euo pipefail

MODE="${1:-validate}"
shift || true

DD_FROM="${AVATAR_DD_FROM:-now-15m}"
DD_LIMIT="${AVATAR_DD_LIMIT:-200}"

discover_tail() {
  if [[ -n "${AVATAR_DD_TAIL:-}" && -x "${AVATAR_DD_TAIL}" ]]; then
    printf '%s\n' "$AVATAR_DD_TAIL"
    return 0
  fi
  local match
  match=$(ls -t "$HOME"/.copilot/session-state/*/files/dd-avatar-tail.sh 2>/dev/null | head -n1 || true)
  if [[ -n "$match" && -x "$match" ]]; then
    printf '%s\n' "$match"
    return 0
  fi
  return 1
}

redact() {
  # Narrow redaction: only collapse clearly secret-shaped tokens. The
  # previous broad regex (any 32+ run of [A-Za-z0-9_+/.-]) ate URL paths,
  # hostnames, and query strings, which masked blocker patterns such as
  # /animate.*5xx, openai.*401, and elevenlabs.*401. The patterns below
  # target shapes that are unambiguously secrets and avoid path
  # punctuation (`/`, `.`, `-`) in the generic long-token catches.
  sed -E \
    -e 's#sk-[A-Za-z0-9_-]{16,}#[redacted-sk]#g' \
    -e 's#eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+#[redacted-jwt]#g' \
    -e 's#(([Bb]earer|[Tt]oken|[Aa]pi[-_ ]?[Kk]ey)[ =:"]+)[A-Za-z0-9._+/=-]{16,}#\1[redacted]#g' \
    -e 's#[A-Fa-f0-9]{40,}#[redacted-hex]#g' \
    -e 's#[A-Za-z0-9+]{40,}={0,2}#[redacted-b64]#g'
}

if ! TAIL_BIN=$(discover_tail); then
  cat >&2 <<EOF
error: could not locate dd-avatar-tail.sh.
       Set AVATAR_DD_TAIL or place the helper under
       ~/.copilot/session-state/<id>/files/dd-avatar-tail.sh and chmod +x.
EOF
  exit 2
fi

declare -a TAIL_ARGS=(--from "$DD_FROM" --limit "$DD_LIMIT")
if [[ -n "${AVATAR_DD_QUERY:-}" ]]; then
  TAIL_ARGS+=(--query "$AVATAR_DD_QUERY")
fi

case "$MODE" in
  tail)
    exec "$TAIL_BIN" "${TAIL_ARGS[@]}" "$@"
    ;;
  validate)
    : # fall through
    ;;
  -h|--help|help)
    sed -n '2,30p' "$0"
    exit 0
    ;;
  *)
    echo "error: unknown mode '$MODE' (expected 'validate' or 'tail')" >&2
    exit 2
    ;;
esac

# --- one-shot validation ------------------------------------------------------

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
RAW="$WORK/dd.log"

echo "[avatar-validate] running one-shot Datadog tail (from=$DD_FROM limit=$DD_LIMIT)" >&2
if ! "$TAIL_BIN" "${TAIL_ARGS[@]}" --once "$@" 2> >(redact >&2) | redact > "$RAW"; then
  echo "[avatar-validate] datadog tail failed; see stderr above" >&2
  exit 3
fi

LINES=$(wc -l < "$RAW" | tr -d ' ')
echo "[avatar-validate] received $LINES log lines" >&2

# Blocker patterns. Each entry: "<label>|<egrep pattern>".
# Patterns are case-insensitive and matched against the redacted log stream.
PATTERNS=(
  'cloudflare-524|524|cloudflare.*timeout'
  'openai-auth-fallback|openai.*(401|403|invalid_api_key|auth.*fail|falling back)'
  'tts-fallback-header|x-tts-fallback|elevenlabs.*(401|403|fallback|voice_clone_sample)'
  'echomimic-5xx|/animate.*(5[0-9]{2})|echomimic.*(error|5[0-9]{2}|timeout)'
  'nats-stale-narration|nats.*(stale|disconnect|no responders|narration.*stuck)'
  'browser-stuck-working|working.*(stuck|stalled)|avatar.*frozen'
)

FAIL=0
declare -a HITS
for entry in "${PATTERNS[@]}"; do
  label="${entry%%|*}"
  pat="${entry#*|}"
  if matches=$(grep -iEn "$pat" "$RAW" || true); [[ -n "$matches" ]]; then
    FAIL=1
    HITS+=("$label")
    echo "[avatar-validate] BLOCKER: $label" >&2
    echo "$matches" | sed 's/^/    /' >&2
  fi
done

if [[ "$FAIL" -ne 0 ]]; then
  printf '[avatar-validate] FAIL: blockers detected -> %s\n' "${HITS[*]}" >&2
  exit 1
fi

if [[ "$LINES" -eq 0 ]]; then
  echo "[avatar-validate] WARN: no log lines in window — verify the avatar pods are emitting and the DD query is current" >&2
  exit 4
fi

echo "[avatar-validate] PASS: no blocker patterns in last $DD_FROM window" >&2
