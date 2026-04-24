#!/usr/bin/env bash
# Gate 2 validation: portrait photo + audio clip → MP4 (audio-driven lip sync).
#
# EchoMimic V3 is AUDIO-driven, so Gate 2 uses Morgan's voice sample that
# already lives in the repo root (voice_clone_sample.mp3).
#
# Usage:
#   APP_URL=https://<id>.app.gra.ai.cloud.ovh.net ./gate2-validate.sh
#
# Env overrides:
#   SOURCE_IMG=avatar/morgan.jpg
#   AUDIO=voice_clone_sample.mp3
#   OUT=_runs/gate2-echomimic.mp4
#   VIDEO_LENGTH=147                 # optional /animate tuning override
#   SAMPLE_HEIGHT=768                # optional /animate tuning override
#   SAMPLE_WIDTH=768                 # optional /animate tuning override
#   WEIGHT_DTYPE=bfloat16            # optional /animate tuning override
#   PROMPT='A golden retriever...'    # optional identity-preserving prompt

set -euo pipefail

[[ -n "${APP_URL:-}" ]] || { echo "APP_URL is required" >&2; exit 2; }

REPO_ROOT="$(git rev-parse --show-toplevel)"
SOURCE_IMG="${SOURCE_IMG:-${REPO_ROOT}/avatar/morgan.jpg}"
AUDIO="${AUDIO:-${REPO_ROOT}/voice_clone_sample.mp3}"
OUT="${OUT:-${REPO_ROOT}/_runs/gate2-echomimic.mp4}"

mkdir -p "$(dirname "$OUT")"
[[ -f "$SOURCE_IMG" ]] || { echo "source image not found: $SOURCE_IMG" >&2; exit 2; }
[[ -f "$AUDIO"      ]] || { echo "audio not found: $AUDIO"            >&2; exit 2; }

echo "[info] reachability check (Kong intercepts /health with 204, so hit /docs instead)"
curl -fsS --max-time 15 -o /dev/null "${APP_URL%/}/docs" || {
  echo "[fatal] app not reachable at ${APP_URL}"; exit 3;
}

form=(-F "source=@${SOURCE_IMG}" -F "audio=@${AUDIO}")
[[ -z "${VIDEO_LENGTH:-}" ]] || form+=(-F "video_length=${VIDEO_LENGTH}")
[[ -z "${SAMPLE_HEIGHT:-}" ]] || form+=(-F "sample_height=${SAMPLE_HEIGHT}")
[[ -z "${SAMPLE_WIDTH:-}" ]] || form+=(-F "sample_width=${SAMPLE_WIDTH}")
[[ -z "${WEIGHT_DTYPE:-}" ]] || form+=(-F "weight_dtype=${WEIGHT_DTYPE}")
[[ -z "${PROMPT:-}" ]] || form+=(-F "prompt=${PROMPT}")

echo "[info] POST /animate  source=$SOURCE_IMG  audio=$(basename "$AUDIO")"
http_code=$(curl -sS -o "$OUT" -w '%{http_code}' \
  --max-time 900 \
  "${form[@]}" \
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
echo "[next] eyeball $OUT; if lip-sync locked to audio, Gate 2 PASS."
