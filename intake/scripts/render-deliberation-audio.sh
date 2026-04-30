#!/usr/bin/env bash
# render-deliberation-audio.sh — Render a multi-speaker deliberation transcript to MP3
# Called by deliberation.lobster.yaml and design-deliberation.lobster.yaml
#
# Usage: render-deliberation-audio.sh <name> <transcript_json> <output_mp3> <status_json> <log_file> [validation_json]
#   name            — e.g. "architecture-deliberation" or "design-deliberation"
#   transcript_json — path to the transcript JSON (segments array)
#   output_mp3      — destination MP3 path
#   status_json     — path to write render status
#   log_file        — path to write render log
#   validation_json — optional path to write post-render validation status
set -euo pipefail

NAME="${1:?render-deliberation-audio: missing name}"
TRANSCRIPT="${2:?render-deliberation-audio: missing transcript path}"
OUTPUT_MP3="${3:?render-deliberation-audio: missing output mp3 path}"
STATUS_JSON="${4:?render-deliberation-audio: missing status json path}"
LOG_FILE="${5:?render-deliberation-audio: missing log file path}"
VALIDATION_JSON="${6:-${STATUS_JSON%.status.json}.validation.json}"

ROOT="${WORKSPACE:-.}"
VOICE_BIN="${LOBSTER_VOICE_BIN:-$ROOT/apps/lobster-voice/lobster-voice}"
VALIDATE_AUDIO_BIN="${VALIDATE_DELIBERATION_AUDIO_BIN:-$ROOT/intake/scripts/validate-deliberation-audio.sh}"

mkdir -p "$(dirname "$OUTPUT_MP3")" "$(dirname "$STATUS_JSON")" "$(dirname "$LOG_FILE")" "$(dirname "$VALIDATION_JSON")"

if [ ! -f "$TRANSCRIPT" ]; then
  echo "render-deliberation-audio: transcript not found at $TRANSCRIPT" >&2
  jq -nc --arg name "$NAME" --arg ts "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    '{name:$name,status:"failed",reason:"transcript_missing",startedAt:$ts,completedAt:$ts}' \
    > "$STATUS_JSON"
  exit 1
fi

if ! command -v "$VOICE_BIN" >/dev/null 2>&1; then
  echo "render-deliberation-audio: lobster-voice not found at $VOICE_BIN" >&2
  jq -nc --arg name "$NAME" --arg ts "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    '{name:$name,status:"failed",reason:"lobster_voice_missing",startedAt:$ts,completedAt:$ts}' \
    > "$STATUS_JSON"
  exit 1
fi

START_TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

jq -nc --arg name "$NAME" --arg ts "$START_TS" \
  '{name:$name,status:"rendering",startedAt:$ts}' > "$STATUS_JSON"

echo "render-deliberation-audio: starting render of $NAME" >&2
echo "  transcript: $TRANSCRIPT" >&2
echo "  output:     $OUTPUT_MP3" >&2

if "$VOICE_BIN" render-transcript \
    --input "$TRANSCRIPT" \
    --output "$OUTPUT_MP3" \
    --status "$STATUS_JSON" \
    --log "$LOG_FILE" 2>>"$LOG_FILE"; then
  END_TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "render-deliberation-audio: $NAME rendered successfully → $OUTPUT_MP3" >&2
  # Update status (lobster-voice may have already written it)
  if [ -f "$STATUS_JSON" ]; then
    jq --arg ts "$END_TS" '. + {completedAt: $ts}' "$STATUS_JSON" > "${STATUS_JSON}.tmp" \
      && mv "${STATUS_JSON}.tmp" "$STATUS_JSON" 2>/dev/null || true
  fi
  if [ ! -x "$VALIDATE_AUDIO_BIN" ]; then
    echo "render-deliberation-audio: validator not found at $VALIDATE_AUDIO_BIN" >&2
    jq -nc --arg name "$NAME" --arg path "$VALIDATE_AUDIO_BIN" --arg ts "$END_TS" \
      '{name:$name,status:"failed",reason:"validator_missing",validatorPath:$path,validatedAt:$ts}' \
      > "$VALIDATION_JSON"
    exit 1
  fi
  "$VALIDATE_AUDIO_BIN" "$NAME" "$OUTPUT_MP3" "$STATUS_JSON" "$VALIDATION_JSON"
else
  END_TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "render-deliberation-audio: $NAME render failed — see $LOG_FILE" >&2
  jq -nc --arg name "$NAME" --arg start "$START_TS" --arg end "$END_TS" \
    '{name:$name,status:"failed",reason:"render_error",startedAt:$start,completedAt:$end}' \
    > "$STATUS_JSON"
  exit 1
fi
