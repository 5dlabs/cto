#!/usr/bin/env bash
# validate-deliberation-audio.sh — Validate durable deliberation MP3 output.
#
# Usage: validate-deliberation-audio.sh <name> <mp3_path> <status_json> <validation_json>
set -euo pipefail

NAME="${1:?validate-deliberation-audio: missing name}"
MP3_PATH="${2:?validate-deliberation-audio: missing mp3 path}"
STATUS_JSON="${3:?validate-deliberation-audio: missing status json path}"
VALIDATION_JSON="${4:?validate-deliberation-audio: missing validation json path}"

mkdir -p "$(dirname "$VALIDATION_JSON")"
TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

write_failed() {
  local reason="$1"
  local detail="${2:-}"
  jq -nc \
    --arg name "$NAME" \
    --arg mp3 "$MP3_PATH" \
    --arg statusPath "$STATUS_JSON" \
    --arg status "failed" \
    --arg reason "$reason" \
    --arg detail "$detail" \
    --arg ts "$TS" \
    '{name:$name,status:$status,reason:$reason,detail:$detail,mp3Path:$mp3,statusPath:$statusPath,validatedAt:$ts}' \
    > "$VALIDATION_JSON"
}

if [ ! -f "$STATUS_JSON" ]; then
  echo "validate-deliberation-audio: status missing at $STATUS_JSON" >&2
  write_failed "status_missing"
  exit 1
fi

if ! jq -e '.status == "complete"' "$STATUS_JSON" >/dev/null 2>&1; then
  echo "validate-deliberation-audio: render status is not complete in $STATUS_JSON" >&2
  write_failed "status_not_complete" "$(jq -r '.status // "unknown"' "$STATUS_JSON" 2>/dev/null || printf unknown)"
  exit 1
fi

if [ ! -s "$MP3_PATH" ]; then
  echo "validate-deliberation-audio: mp3 missing or empty at $MP3_PATH" >&2
  write_failed "mp3_missing"
  exit 1
fi

if ! command -v ffprobe >/dev/null 2>&1; then
  echo "validate-deliberation-audio: ffprobe missing" >&2
  write_failed "ffprobe_missing"
  exit 1
fi

PROBE_JSON="$(ffprobe -v error -select_streams a:0 -show_entries stream=codec_name -show_entries format=duration,format_name -of json "$MP3_PATH" 2>/tmp/validate-deliberation-audio.ffprobe.err)" || {
  detail="$(cat /tmp/validate-deliberation-audio.ffprobe.err 2>/dev/null || true)"
  echo "validate-deliberation-audio: ffprobe failed for $MP3_PATH" >&2
  write_failed "invalid_audio" "$detail"
  exit 1
}

if ! printf '%s' "$PROBE_JSON" | jq -e '(.streams // []) | length > 0' >/dev/null 2>&1; then
  echo "validate-deliberation-audio: no audio stream in $MP3_PATH" >&2
  write_failed "invalid_audio" "no audio stream"
  exit 1
fi

DURATION="$(printf '%s' "$PROBE_JSON" | jq -r '.format.duration // empty')"
if ! awk -v d="$DURATION" 'BEGIN { exit !(d+0 > 0) }'; then
  echo "validate-deliberation-audio: invalid duration '$DURATION' for $MP3_PATH" >&2
  write_failed "invalid_audio" "non-positive duration"
  exit 1
fi

CODEC="$(printf '%s' "$PROBE_JSON" | jq -r '.streams[0].codec_name // "unknown"')"
FORMAT="$(printf '%s' "$PROBE_JSON" | jq -r '.format.format_name // "unknown"')"
TRANSCRIPT_HASH="$(jq -r '.transcriptHash // empty' "$STATUS_JSON" 2>/dev/null || true)"
TRANSCRIPT_PATH="$(jq -r '.inputPath // empty' "$STATUS_JSON" 2>/dev/null || true)"
if [ -z "$TRANSCRIPT_HASH" ]; then
  echo "validate-deliberation-audio: render status missing transcript hash" >&2
  write_failed "transcript_hash_missing"
  exit 1
fi
MP3_SHA256="$(sha256sum "$MP3_PATH" | awk '{print $1}')"

jq -nc \
  --arg name "$NAME" \
  --arg mp3 "$MP3_PATH" \
  --arg statusPath "$STATUS_JSON" \
  --arg transcriptPath "$TRANSCRIPT_PATH" \
  --arg transcriptHash "$TRANSCRIPT_HASH" \
  --arg mp3Sha256 "$MP3_SHA256" \
  --arg ts "$TS" \
  --arg codec "$CODEC" \
  --arg format "$FORMAT" \
  --argjson duration "$DURATION" \
  '{name:$name,status:"valid",mp3Path:$mp3,statusPath:$statusPath,transcriptPath:$transcriptPath,transcriptHash:$transcriptHash,mp3Sha256:$mp3Sha256,validatedAt:$ts,durationSeconds:$duration,codec:$codec,format:$format}' \
  > "$VALIDATION_JSON"

echo "validate-deliberation-audio: $NAME valid (${DURATION}s, codec=$CODEC)" >&2
