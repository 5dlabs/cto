#!/usr/bin/env bash
set -euo pipefail

# Upload a local HTML mockup to Linear as a screenshot image.
# Usage: upload-mockup-to-linear.sh <html_file> <device_type> [session_id]
# Requires: playwright (python3), LINEAR_OAUTH_TOKEN or op://...
# Returns: JSON { "assetUrl": "...", "success": true }

HTML_FILE="${1:?usage: upload-mockup-to-linear.sh <html_file> <device_type> [session_id]}"
DEVICE="${2:-DESKTOP}"
SESSION_ID="${3:-}"

ROOT="${WORKSPACE:-.}"
SCREENSHOT_DIR="$ROOT/.intake/design/stitch/screenshots"
mkdir -p "$SCREENSHOT_DIR"

BASENAME="$(basename "$HTML_FILE" .html)"
PNG_FILE="$SCREENSHOT_DIR/${BASENAME}.png"

# Viewport dimensions
if [ "$DEVICE" = "MOBILE" ]; then
  WIDTH=390; HEIGHT=844
else
  WIDTH=1440; HEIGHT=900
fi

# Step 1: Screenshot the HTML with playwright
python3 - "$HTML_FILE" "$PNG_FILE" "$WIDTH" "$HEIGHT" <<'PY'
import sys
from playwright.sync_api import sync_playwright

html_file, png_file = sys.argv[1], sys.argv[2]
width, height = int(sys.argv[3]), int(sys.argv[4])

with sync_playwright() as p:
    browser = p.chromium.launch(headless=True)
    page = browser.new_page(viewport={"width": width, "height": height})
    page.goto(f"file://{html_file}", wait_until="networkidle")
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

# Step 2: Get Linear OAuth token
LINEAR_OAUTH="${LINEAR_OAUTH_TOKEN:-}"
if [ -z "$LINEAR_OAUTH" ]; then
  LINEAR_OAUTH="$(op read 'op://Automation/Linear Morgan OAuth/developer_token' 2>/dev/null)" || true
fi
if [ -z "$LINEAR_OAUTH" ]; then
  echo "upload-mockup: no OAuth token available" >&2
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
