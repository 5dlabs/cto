#!/usr/bin/env bash
# One-off MP3 from ElevenLabs using YOUR cloned voice ID (or any voice_id).
#
# Where this fits in the repo:
#   - Morgan avatar: avatar/agent/morgan_avatar_agent/config.py  → MORGAN_ELEVEN_VOICE_ID
#   - Docs (custom / cloned voices): avatar/docs/elevenlabs-playbook.md
#   - TTS implementation: apps/lobster-voice/src/elevenlabs.ts (same API shape)
#
# Setup:
#   export ELEVEN_API_KEY="..."           # xi-api-key from ElevenLabs
#   export MORGAN_ELEVEN_VOICE_ID="..."   # your Instant Voice Clone / Professional ID
#
# Usage:
#   ./scripts/elevenlabs-tts-sample.sh "Hello, this is a test of my cloned voice."
#   ./scripts/elevenlabs-tts-sample.sh "Some text" ./my-voice-sample.mp3
#
set -euo pipefail

TEXT="${1:-This is a quick test of the ElevenLabs voice configured for 5D Labs.}"
OUT="${2:-./5dlabs-voice-sample.mp3}"

KEY="${ELEVEN_API_KEY:-}"
VID="${MORGAN_ELEVEN_VOICE_ID:-${ELEVEN_VOICE_ID:-}}"

if [[ -z "$KEY" ]]; then
  echo "Set ELEVEN_API_KEY (ElevenLabs xi-api-key)." >&2
  exit 1
fi
if [[ -z "$VID" ]]; then
  echo "Set MORGAN_ELEVEN_VOICE_ID or ELEVEN_VOICE_ID to your voice_id from ElevenLabs Voices." >&2
  exit 1
fi

MODEL="${ELEVEN_MODEL_ID:-eleven_flash_v2_5}"

JSON=$(python3 -c "
import json, os, sys
text = sys.argv[1]
model = sys.argv[2]
print(json.dumps({
  'text': text,
  'model_id': model,
  'voice_settings': {
    'stability': 0.5,
    'similarity_boost': 0.75,
    'style': 0.3,
    'use_speaker_boost': True,
  },
}))
" "$TEXT" "$MODEL")

curl -sS -X POST "https://api.elevenlabs.io/v1/text-to-speech/${VID}" \
  -H "xi-api-key: ${KEY}" \
  -H "Content-Type: application/json" \
  -d "$JSON" \
  -o "$OUT"

echo "Wrote: $OUT"
echo "Play: afplay \"$OUT\"   # macOS"
