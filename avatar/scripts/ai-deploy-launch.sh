#!/usr/bin/env bash
# Launch (or refresh) a LivePortrait AI Deploy app on OVH.
#
# Reads OVH creds from 1Password (op://Automation/OVH CA API).
# Idempotent-ish: if an app with the same name is already running, prints its
# URL and exits. Otherwise POSTs a new one and polls until RUNNING.
#
# Env overrides:
#   IMAGE=ghcr.io/5dlabs/liveportrait:latest
#   FLAVOR=ai1-1-gpu-v100s           # single V100S, covered by credits
#   REGION=GRA                       # Gravelines
#   APP_NAME=liveportrait-gate1
#   PORT=8000
#   PROBE_PATH=/health
#   UNSAFE_OBJECT_STORAGE=0          # we bake weights; no volume needed

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/ovh-api.sh"

IMAGE="${IMAGE:-ghcr.io/5dlabs/liveportrait:latest}"
FLAVOR="${FLAVOR:-ai1-1-gpu}"
REGION="${REGION:-GRA}"
APP_NAME="${APP_NAME:-liveportrait-gate1}"
PORT="${PORT:-8000}"
PROBE_PATH="${PROBE_PATH:-/health}"

need() { command -v "$1" >/dev/null 2>&1 || { echo "missing $1" >&2; exit 1; }; }
need jq
need curl

PROJECT_ID="$(op read 'op://Automation/OVH CA API/project_id')"
[[ -n "$PROJECT_ID" ]] || { echo "project_id missing from op://Automation/OVH CA API" >&2; exit 1; }

existing="$(ovh_api GET "/cloud/project/${PROJECT_ID}/ai/app" \
  | jq -r --arg n "$APP_NAME" '.[] | select(.spec.name == $n) | .id' | head -1 || true)"

if [[ -n "$existing" ]]; then
  echo "[info] app '${APP_NAME}' already exists (id=${existing}); refreshing status"
  APP_ID="$existing"
else
  body=$(jq -n \
    --arg name "$APP_NAME" \
    --arg region "$REGION" \
    --arg flavor "$FLAVOR" \
    --arg image "$IMAGE" \
    --argjson port "$PORT" \
    --arg probe "$PROBE_PATH" \
    '{
      name: $name,
      region: $region,
      image: $image,
      resources: { flavor: $flavor, flavorCount: 1 },
      command: [],
      defaultHttpPort: $port,
      probe: { path: $probe, port: $port },
      unsecureHttp: true,
      scalingStrategy: { fixed: { replicas: 1 } },
      labels: { owner: "5dlabs", purpose: "morgan-avatar-gate1", model: "liveportrait" }
    }')

  echo "[info] creating AI Deploy app ${APP_NAME} (image=${IMAGE}, flavor=${FLAVOR}, region=${REGION})"
  resp="$(ovh_api POST "/cloud/project/${PROJECT_ID}/ai/app" "$body")"
  APP_ID="$(echo "$resp" | jq -r '.id')"
  [[ -n "$APP_ID" && "$APP_ID" != "null" ]] || { echo "failed to create app:"; echo "$resp" | jq .; exit 1; }
  echo "[info] created app id=${APP_ID}"
fi

echo "[info] polling for RUNNING status..."
url=""
for i in $(seq 1 60); do
  info="$(ovh_api GET "/cloud/project/${PROJECT_ID}/ai/app/${APP_ID}")"
  state="$(echo "$info" | jq -r '.status.state // .state // "UNKNOWN"')"
  url="$(echo "$info" | jq -r '.status.url // .url // empty')"
  printf "  t=%3ds  state=%s%s\n" "$((i*10))" "$state" "${url:+  url=$url}"
  case "$state" in
    RUNNING|SCALED_UP) break ;;
    FAILED|ERROR)
      echo "[fatal] app entered $state"
      echo "$info" | jq '.status.history[-5:]' 2>/dev/null || true
      exit 2
      ;;
  esac
  sleep 10
done

[[ -n "$url" ]] || { echo "[warn] RUNNING but no URL yet; check OVH Manager"; exit 3; }

echo "[ok] app=${APP_ID} url=${url}"
echo "[next] curl -fsSL ${url}${PROBE_PATH}"
