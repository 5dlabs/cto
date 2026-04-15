#!/usr/bin/env bash
# Wrapper around openclaw.invoke with exponential backoff on transient failures.
# Usage: openclaw-invoke-retry.sh [openclaw.invoke args...]
# Env: OPENCLAW_MAX_RETRIES (default 5)

set -euo pipefail
MAX=${OPENCLAW_MAX_RETRIES:-5}
ERR_FILE=$(mktemp)
trap 'rm -f "$ERR_FILE"' EXIT

_ROOT="${WORKSPACE:-.}"
_RUN_ID_FILE="$_ROOT/.intake/run-id.txt"

_resolve_run_id() {
  if [ -n "${INTAKE_RUN_ID:-}" ]; then
    printf '%s' "$INTAKE_RUN_ID"
  elif [ -f "$_RUN_ID_FILE" ]; then
    cat "$_RUN_ID_FILE"
  else
    printf 'unknown'
  fi
}

# Observability: log retry events
_RETRY_LOG_DIR="$_ROOT/.intake/logs"
_retry_log() {
  local attempt="$1" status="$2" backoff="${3:-0}" error="${4:-}"
  mkdir -p "$_RETRY_LOG_DIR" 2>/dev/null || true
  local ts run_id
  ts=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  run_id="$(_resolve_run_id)"
  error=$(printf '%s' "$error" | head -c 200 | sed 's/"/\\"/g' | tr '\n' ' ')
  printf '{"ts":"%s","event":"retry","run_id":"%s","attempt":%d,"max_attempts":%d,"status":"%s","backoff_sec":%d,"error":"%s"}\n' \
    "$ts" "$run_id" "$attempt" "$MAX" "$status" "$backoff" "$error" \
    >> "$_RETRY_LOG_DIR/llm-calls.jsonl"
}

_is_local_gateway_failure() {
  local text="$1"
  case "$text" in
    *"ECONNREFUSED"*|*"ECONNRESET"*|*"connection refused"*|*"connect ECONNREFUSED"*) return 0 ;;
  esac
  if ! curl -sf --max-time 3 http://127.0.0.1:18789/health >/dev/null 2>&1; then
    return 0
  fi
  return 1
}

_restart_gateway() {
  echo "openclaw-invoke-retry: restarting gateway..." >&2
  for pid in $(ps -eo pid,comm 2>/dev/null | awk '/openclaw/{print $1}' | head -5); do
    kill -9 "$pid" 2>/dev/null || true
  done
  sleep 2
  nohup openclaw gateway > /tmp/openclaw-gateway-restart.log 2>&1 &
  local gw_pid=$!
  echo "openclaw-invoke-retry: gateway started (pid=$gw_pid), waiting for health..." >&2
  local i=0
  while [ $i -lt 10 ]; do
    if curl -sf --max-time 2 http://127.0.0.1:18789/health >/dev/null 2>&1; then
      echo "openclaw-invoke-retry: gateway healthy after restart" >&2
      return 0
    fi
    sleep 1
    i=$((i + 1))
  done
  echo "openclaw-invoke-retry: gateway may not be healthy after restart" >&2
  return 1
}

for ATTEMPT in $(seq 1 "$MAX"); do
  if OUTPUT=$(openclaw.invoke "$@" 2>"$ERR_FILE"); then
    cat "$ERR_FILE" >&2
    printf '%s' "$OUTPUT"
    exit 0
  fi
  ERR_TEXT=$(cat "$ERR_FILE")
  case "$ERR_TEXT$OUTPUT" in
    *"503"*|*"Service Unavailable"*|*"ECONNREFUSED"*|*"ECONNRESET"*|*"timed out"*|*"network"*|*"rate limit"*|*"429"*|*"all LLM providers failed"*)
      if [ "$ATTEMPT" -lt "$MAX" ]; then
        BACKOFF=$((ATTEMPT * ATTEMPT * 5))
        echo "openclaw-invoke-retry: transient failure (attempt $ATTEMPT/$MAX), retrying in ${BACKOFF}s..." >&2
        echo "  stderr: ${ERR_TEXT:0:200}" >&2
        _retry_log "$ATTEMPT" "transient" "$BACKOFF" "${ERR_TEXT:0:200}"
        if [ "$ATTEMPT" -ge 3 ] && _is_local_gateway_failure "$ERR_TEXT$OUTPUT"; then
          _restart_gateway || true
        else
          sleep "$BACKOFF"
        fi
        continue
      fi
      _retry_log "$ATTEMPT" "exhausted" "0" "${ERR_TEXT:0:200}"
      ;;
    *)
      _retry_log "$ATTEMPT" "fatal" "0" "${ERR_TEXT:0:200}"
      ;;
  esac
  cat "$ERR_FILE" >&2
  printf '%s' "$OUTPUT"
  exit 1
done
cat "$ERR_FILE" >&2
printf '%s' "$OUTPUT"
exit 1
