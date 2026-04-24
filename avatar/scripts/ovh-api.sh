#!/usr/bin/env bash
# Thin OVH API client using the Automation:OVH CA API credential.
# Usage: ovh-api.sh METHOD PATH [BODY_JSON]
set -euo pipefail

OP_ACCT="${OP_ACCOUNT_OVH:-my.1password.com}"
AK="${OVH_APPLICATION_KEY:-$(op --account "$OP_ACCT" read 'op://Automation/OVH CA API/application_key')}"
AS="${OVH_APPLICATION_SECRET:-$(op --account "$OP_ACCT" read 'op://Automation/OVH CA API/application_secret')}"
CK="${OVH_CONSUMER_KEY:-$(op --account "$OP_ACCT" read 'op://Automation/OVH CA API/consumer_key')}"
EP="${OVH_ENDPOINT:-$(op --account "$OP_ACCT" read 'op://Automation/OVH CA API/endpoint')}"

case "$EP" in
  http*)  BASE="${EP%/}" ;;
  ovh-ca) BASE="https://ca.api.ovh.com/1.0" ;;
  ovh-eu) BASE="https://eu.api.ovh.com/1.0" ;;
  ovh-us) BASE="https://api.us.ovhcloud.com/1.0" ;;
  *)      BASE="https://ca.api.ovh.com/1.0" ;;
esac

METHOD="${1:-GET}"
URLPATH="${2:-/me}"
BODY="${3:-}"
URL="${BASE}${URLPATH}"

# Server time to avoid clock drift
TS=$(curl -s "${BASE}/auth/time")
SIG_IN="${AS}+${CK}+${METHOD}+${URL}+${BODY}+${TS}"
SIG="\$1\$$(printf '%s' "$SIG_IN" | shasum -a 1 | awk '{print $1}')"

if [ -n "$BODY" ]; then
  curl -sS -X "$METHOD" "$URL" \
    -H "X-Ovh-Application: $AK" \
    -H "X-Ovh-Consumer: $CK" \
    -H "X-Ovh-Timestamp: $TS" \
    -H "X-Ovh-Signature: $SIG" \
    -H "Content-Type: application/json" \
    --data "$BODY"
else
  curl -sS -X "$METHOD" "$URL" \
    -H "X-Ovh-Application: $AK" \
    -H "X-Ovh-Consumer: $CK" \
    -H "X-Ovh-Timestamp: $TS" \
    -H "X-Ovh-Signature: $SIG"
fi
echo
