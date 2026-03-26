#!/usr/bin/env bash
# Wrapper around openclaw.invoke with exponential backoff on transient failures.
# Usage: openclaw-invoke-retry.sh [openclaw.invoke args...]
# Env: OPENCLAW_MAX_RETRIES (default 5)

set -euo pipefail
MAX=${OPENCLAW_MAX_RETRIES:-5}
ERR_FILE=$(mktemp)
trap 'rm -f "$ERR_FILE"' EXIT

for ATTEMPT in $(seq 1 "$MAX"); do
  if OUTPUT=$(openclaw.invoke "$@" 2>"$ERR_FILE"); then
    cat "$ERR_FILE" >&2
    printf '%s' "$OUTPUT"
    exit 0
  fi
  ERR_TEXT=$(cat "$ERR_FILE")
  case "$ERR_TEXT$OUTPUT" in
    *"503"*|*"Service Unavailable"*|*"ECONNREFUSED"*|*"timed out"*|*"network"*|*"rate limit"*|*"429"*|*"all LLM providers failed"*)
      if [ "$ATTEMPT" -lt "$MAX" ]; then
        BACKOFF=$((ATTEMPT * ATTEMPT * 5))
        echo "openclaw-invoke-retry: transient failure (attempt $ATTEMPT/$MAX), retrying in ${BACKOFF}s..." >&2
        echo "  stderr: ${ERR_TEXT:0:200}" >&2
        if [ "$ATTEMPT" -ge 3 ]; then
          echo "openclaw-invoke-retry: killing stale gateway on attempt $ATTEMPT..." >&2
          pkill -f openclaw-gateway 2>/dev/null || true
          sleep 3
        else
          sleep "$BACKOFF"
        fi
        continue
      fi
      ;;
  esac
  cat "$ERR_FILE" >&2
  printf '%s' "$OUTPUT"
  exit 1
done
cat "$ERR_FILE" >&2
printf '%s' "$OUTPUT"
exit 1
