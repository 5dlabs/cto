#!/usr/bin/env bash
# Gate 1 validation: photoreal photo + sample driving video → MP4.
#
# LivePortrait is VIDEO-driven (not audio-driven), so Gate 1 uses a driving
# clip from the LivePortrait repo assets. Audio-driven lip-sync moves to
# Gate 2 (EchoMimic V3).
#
# Usage:
#   APP_URL=https://<id>.app.gra.ai.cloud.ovh.net ./gate1-validate.sh
#
# Env overrides:
#   SOURCE_IMG=avatar/morgan.jpg
#   DRIVING_URL=https://raw.githubusercontent.com/KwaiVGI/LivePortrait/main/assets/examples/driving/d0.mp4
#   OUT=_runs/gate1-liveportrait.mp4

set -euo pipefail

[[ -n "${APP_URL:-}" ]] || { echo "APP_URL is required" >&2; exit 2; }

REPO_ROOT="$(git rev-parse --show-toplevel)"
SOURCE_IMG="${SOURCE_IMG:-${REPO_ROOT}/avatar/morgan.jpg}"
DRIVING_URL="${DRIVING_URL:-https://raw.githubusercontent.com/KwaiVGI/LivePortrait/main/assets/examples/driving/d0.mp4}"
OUT="${OUT:-${REPO_ROOT}/_runs/gate1-liveportrait.mp4}"

mkdir -p "$(dirname "$OUT")"
[[ -f "$SOURCE_IMG" ]] || { echo "source image not found: $SOURCE_IMG" >&2; exit 2; }

driving="$(mktemp -t liveportrait-driving.XXXXXX.mp4)"
trap 'rm -f "$driving"' EXIT
echo "[info] fetching driving clip: $DRIVING_URL"
curl -fsSL -o "$driving" "$DRIVING_URL"

echo "[info] reachability check (Kong intercepts /health with 204, so hit /docs instead)"
curl -fsS --max-time 15 -o /dev/null "${APP_URL%/}/docs" || {
  echo "[fatal] app not reachable at ${APP_URL}"; exit 3;
}

echo "[info] POST /animate  source=$SOURCE_IMG  driving=$(basename "$driving")"
http_code=$(curl -sS -o "$OUT" -w '%{http_code}' \
  --max-time 600 \
  -F "source=@${SOURCE_IMG}" \
  -F "driving=@${driving}" \
  "${APP_URL%/}/animate")

if [[ "$http_code" != "200" ]]; then
  echo "[fatal] /animate returned HTTP ${http_code}"
  head -c 2000 "$OUT" || true
  exit 4
fi

size="$(wc -c < "$OUT" | tr -d ' ')"
[[ "$size" -gt 50000 ]] || { echo "[fatal] output too small ($size bytes) — likely an error payload"; exit 5; }

echo "[ok] wrote $OUT ($size bytes)"
command -v ffprobe >/dev/null 2>&1 && ffprobe -hide_banner -v error \
  -show_entries stream=codec_name,width,height,duration -of default=nk=1:nw=1 "$OUT" | paste -sd' ' -
echo "[next] eyeball $OUT; if lip-sync / motion natural, Gate 1 PASS."
