#!/usr/bin/env bash
set -euo pipefail

# Upload a local HTML mockup or a live URL to Linear as a screenshot image.
# Usage: upload-mockup-to-linear.sh <html_file_or_url> <device_type> [session_id]
# Requires: playwright (python3), plus a runtime Linear token via env or PM/Kubernetes.
# Returns: JSON { "assetUrl": "...", "success": true }

INPUT_SOURCE="${1:?usage: upload-mockup-to-linear.sh <html_file_or_url> <device_type> [session_id]}"
DEVICE="${2:-DESKTOP}"
SESSION_ID="${3:-}"

ROOT="${WORKSPACE:-.}"
SCREENSHOT_DIR="$ROOT/.intake/design/stitch/screenshots"
mkdir -p "$SCREENSHOT_DIR"

if [[ "$INPUT_SOURCE" =~ ^https?:// ]]; then
  TARGET_URL="$INPUT_SOURCE"
  BASENAME="$(python3 - "$INPUT_SOURCE" <<'PY'
import hashlib
import re
import sys
from urllib.parse import urlparse

source = sys.argv[1]
parsed = urlparse(source)
base = (parsed.netloc + parsed.path).strip('/') or 'remote'
base = re.sub(r'[^A-Za-z0-9._-]+', '-', base).strip('-') or 'remote'
print(f"{base[:80]}-{hashlib.sha1(source.encode()).hexdigest()[:8]}")
PY
)"
else
  TARGET_URL="file://$INPUT_SOURCE"
  BASENAME="$(basename "$INPUT_SOURCE" .html)"
fi
PNG_FILE="$SCREENSHOT_DIR/${BASENAME}.png"

# Viewport dimensions
if [ "$DEVICE" = "MOBILE" ]; then
  WIDTH=390; HEIGHT=844
else
  WIDTH=1440; HEIGHT=900
fi

# Step 1: Screenshot the source with playwright
python3 - "$TARGET_URL" "$PNG_FILE" "$WIDTH" "$HEIGHT" <<'PY'
import sys
from playwright.sync_api import sync_playwright

target_url, png_file = sys.argv[1], sys.argv[2]
width, height = int(sys.argv[3]), int(sys.argv[4])

with sync_playwright() as p:
    browser = p.chromium.launch(headless=True)
    page = browser.new_page(viewport={"width": width, "height": height})
    page.goto(target_url, wait_until="networkidle")
    page.screenshot(path=png_file, full_page=False)
    browser.close()

print(f"Screenshot saved: {png_file}", file=sys.stderr)
PY

if [ ! -f "$PNG_FILE" ]; then
  echo "upload-mockup: screenshot failed" >&2
  jq -nc '{success: false, error: "screenshot failed"}'
  exit 1
fi

FILE_SIZE=$(stat -f%z "$PNG_FILE" 2>/dev/null || stat -c%s "$PNG_FILE" 2>/dev/null)

# Step 2: Get Linear runtime token
LINEAR_OAUTH="${LINEAR_OAUTH_TOKEN:-${LINEAR_API_KEY:-}}"
if [ -z "$LINEAR_OAUTH" ]; then
  PM_BASE_URL="${PM_BASE_URL:-https://pm.5dlabs.ai}"
  NAMESPACE="${NAMESPACE:-cto}"
  curl -fsS -X POST "${PM_BASE_URL}/oauth/mint/morgan" >/dev/null 2>&1 || true
  LINEAR_OAUTH="$(kubectl get secret linear-app-morgan -n "${NAMESPACE}" \
    -o jsonpath='{.data.access_token}' 2>/dev/null | base64 -d 2>/dev/null || true)"
fi
if [ -z "$LINEAR_OAUTH" ]; then
  echo "upload-mockup: no runtime token available" >&2
  jq -nc '{success: false, error: "no oauth token"}'
  exit 1
fi

# Step 3: Request upload URL from Linear
GRAPHQL_BODY="$(jq -nc --arg fn "${BASENAME}.png" --argjson sz "$FILE_SIZE" \
  '{query:"mutation($fn:String!,$ct:String!,$sz:Int!){fileUpload(filename:$fn,contentType:$ct,size:$sz,makePublic:true){success uploadFile{uploadUrl assetUrl headers{key value}}}}", variables:{fn:$fn,ct:"image/png",sz:$sz}}')"
UPLOAD_RESP="$(curl -sS -X POST https://api.linear.app/graphql \
  -H "Authorization: Bearer $LINEAR_OAUTH" \
  -H "Content-Type: application/json" \
  --data-raw "$GRAPHQL_BODY")"

UPLOAD_URL="$(printf '%s' "$UPLOAD_RESP" | jq -r '.data.fileUpload.uploadFile.uploadUrl // empty')"
ASSET_URL="$(printf '%s' "$UPLOAD_RESP" | jq -r '.data.fileUpload.uploadFile.assetUrl // empty')"

if [ -z "$UPLOAD_URL" ]; then
  echo "upload-mockup: fileUpload mutation failed: $(printf '%s' "$UPLOAD_RESP" | head -c 300)" >&2
  jq -nc '{success: false, error: "fileUpload failed"}'
  exit 1
fi

# Step 4: PUT the file to the signed upload URL
HEADERS_JSON="$(printf '%s' "$UPLOAD_RESP" | jq -c '.data.fileUpload.uploadFile.headers // []')"
CURL_HEADERS=(-H "Content-Type: image/png")
for i in $(seq 0 "$(printf '%s' "$HEADERS_JSON" | jq 'length - 1')"); do
  KEY="$(printf '%s' "$HEADERS_JSON" | jq -r ".[$i].key")"
  VAL="$(printf '%s' "$HEADERS_JSON" | jq -r ".[$i].value")"
  CURL_HEADERS+=(-H "$KEY: $VAL")
done

HTTP_CODE="$(curl -sS -o /dev/null -w '%{http_code}' -X PUT \
  "${CURL_HEADERS[@]}" \
  --data-binary "@$PNG_FILE" \
  "$UPLOAD_URL")"

if [ "$HTTP_CODE" = "200" ] || [ "$HTTP_CODE" = "201" ]; then
  echo "upload-mockup: uploaded ${BASENAME}.png (${FILE_SIZE} bytes) → $ASSET_URL" >&2
  jq -nc --arg url "$ASSET_URL" --arg png "$PNG_FILE" '{success: true, assetUrl: $url, localPath: $png}'
else
  echo "upload-mockup: PUT failed HTTP=$HTTP_CODE" >&2
  jq -nc --arg code "$HTTP_CODE" '{success: false, error: ("PUT failed: " + $code)}'
  exit 1
fi
