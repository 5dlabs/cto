#!/usr/bin/env bash
# Audible alert for the intake coordinator.
# Usage:
#   coordinator-speak.sh "short reason without secrets" [normal|error|wait]
#
# Uses lobster-voice (multi-provider TTS: ElevenLabs / OpenAI / xAI).
# If lobster-voice is unavailable, logs to stderr only — never calls macOS say.
#
# Required env:
#   At least one of ELEVEN_API_KEY, OPENAI_API_KEY, XAI_API_KEY

set -euo pipefail
MSG="${1:-Intake coordinator: human intervention required.}"
LEVEL="${2:-normal}"
ROOT="${WORKSPACE:-$(cd "$(dirname "$0")/../.." && pwd)}"
LV_BIN=""

if command -v lobster-voice >/dev/null 2>&1; then
  LV_BIN="lobster-voice"
elif [[ -x "$ROOT/apps/lobster-voice/lobster-voice" ]]; then
  LV_BIN="$ROOT/apps/lobster-voice/lobster-voice"
fi

if [[ -n "$LV_BIN" ]]; then
  WORKSPACE="$ROOT" LOBSTER_VOICE_AUDIO_DEVICE="${INTAKE_AUDIO_DEVICE:-82}" "$LV_BIN" speak "$MSG" --level "$LEVEL" && exit 0
  echo "coordinator-speak: lobster-voice failed — silent skip" >&2
  exit 0
fi

echo "coordinator-speak ($LEVEL): $MSG" >&2
