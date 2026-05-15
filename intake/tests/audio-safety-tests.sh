#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TEST_DIR="$(mktemp -d)"
trap 'rm -rf "$TEST_DIR"' EXIT

pass_count=0
pass() {
  pass_count=$((pass_count + 1))
  echo "ok - $*"
}

fail() {
  echo "not ok - $*" >&2
  exit 1
}

require_grep() {
  local pattern="$1" file="$2" msg="$3"
  grep -Eq "$pattern" "$file" || fail "$msg"
}

# Test 1: audio validation script should exist and validate a status/mp3 pair.
if [ -x "$ROOT/intake/scripts/validate-deliberation-audio.sh" ]; then
  pass "validate-deliberation-audio.sh exists"
else
  fail "validate-deliberation-audio.sh missing or not executable"
fi

# Test 2: render wrapper should invoke validation after lobster-voice succeeds.
require_grep 'validate-deliberation-audio\.sh' \
  "$ROOT/intake/scripts/render-deliberation-audio.sh" \
  "render-deliberation-audio.sh does not call validate-deliberation-audio.sh"
pass "render wrapper invokes MP3 validation"

# Test 3: design deliberation should emit validation artifact after render.
require_grep 'design-deliberation\.validation\.json' \
  "$ROOT/intake/workflows/design-deliberation.lobster.yaml" \
  "design deliberation workflow does not produce validation artifact"
pass "design deliberation validates MP3 artifact"

# Test 4: architecture deliberation should emit validation artifact after render.
require_grep 'architecture-deliberation\.validation\.json' \
  "$ROOT/intake/workflows/deliberation.lobster.yaml" \
  "architecture deliberation workflow does not produce validation artifact"
pass "architecture deliberation validates MP3 artifact"

# Test 5: video generation must require validated design MP3 before resolving/calling intake-agent.
python3 - "$ROOT/intake/workflows/pipeline.lobster.yaml" <<'PY'
import sys
text=open(sys.argv[1]).read()
step=text.split('- id: generate-deliberation-video',1)[1].split('\n  - id:',1)[0]
required=[
  'design-deliberation.validation.json',
  'design-deliberation.mp3',
  'audio_not_validated',
  '.status == "valid"',
  'TRANSCRIPT_SHA256',
  'transcriptHash',
]
missing=[s for s in required if s not in step]
if missing:
    print('missing from generate-deliberation-video step:', ', '.join(missing), file=sys.stderr)
    sys.exit(1)
if step.find('audio_not_validated') > step.find('INTAKE_AGENT_BIN'):
    print('audio validation gate appears after intake-agent resolution', file=sys.stderr)
    sys.exit(1)
PY
pass "video generation is gated on validated MP3 before intake-agent"

# Test 6: fixture-based validation rejects invalid audio without video files.
mkdir -p "$TEST_DIR/.intake/audio" "$TEST_DIR/.tasks/audio"
printf 'not audio' > "$TEST_DIR/.tasks/audio/design-deliberation.mp3"
jq -nc '{status:"complete", outputPath:".tasks/audio/design-deliberation.mp3", transcriptHash:"fixture-hash"}' > "$TEST_DIR/.intake/audio/design-deliberation.status.json"
if WORKSPACE="$TEST_DIR" "$ROOT/intake/scripts/validate-deliberation-audio.sh" \
    "design-deliberation" \
    "$TEST_DIR/.tasks/audio/design-deliberation.mp3" \
    "$TEST_DIR/.intake/audio/design-deliberation.status.json" \
    "$TEST_DIR/.intake/audio/design-deliberation.validation.json" >/tmp/audio-safety-invalid.out 2>/tmp/audio-safety-invalid.err; then
  fail "invalid mp3 unexpectedly passed validation"
fi
jq -e '.status == "failed" and .reason == "invalid_audio"' "$TEST_DIR/.intake/audio/design-deliberation.validation.json" >/dev/null \
  || fail "invalid audio validation did not write failed invalid_audio status"
pass "invalid MP3 fixture is rejected"

# Test 7: valid audio records transcript and MP3 hashes in the validation artifact.
python3 - "$TEST_DIR/.tasks/audio/design-deliberation.mp3" <<'PY'
import math
import struct
import sys
import wave

path = sys.argv[1]
rate = 8000
with wave.open(path, "wb") as wav:
    wav.setnchannels(1)
    wav.setsampwidth(2)
    wav.setframerate(rate)
    frames = bytearray()
    for n in range(rate // 10):
        sample = int(8000 * math.sin(2 * math.pi * 440 * n / rate))
        frames.extend(struct.pack("<h", sample))
    wav.writeframes(frames)
PY
printf '{"sessionId":"fixture","segments":[]}\n' > "$TEST_DIR/.tasks/audio/design-deliberation.transcript.json"
TRANSCRIPT_HASH="$(sha256sum "$TEST_DIR/.tasks/audio/design-deliberation.transcript.json" | awk '{print $1}')"
MP3_HASH="$(sha256sum "$TEST_DIR/.tasks/audio/design-deliberation.mp3" | awk '{print $1}')"
jq -nc \
  --arg output "$TEST_DIR/.tasks/audio/design-deliberation.mp3" \
  --arg input "$TEST_DIR/.tasks/audio/design-deliberation.transcript.json" \
  --arg hash "$TRANSCRIPT_HASH" \
  '{status:"complete", outputPath:$output, inputPath:$input, transcriptHash:$hash}' \
  > "$TEST_DIR/.intake/audio/design-deliberation.status.json"
WORKSPACE="$TEST_DIR" "$ROOT/intake/scripts/validate-deliberation-audio.sh" \
  "design-deliberation" \
  "$TEST_DIR/.tasks/audio/design-deliberation.mp3" \
  "$TEST_DIR/.intake/audio/design-deliberation.status.json" \
  "$TEST_DIR/.intake/audio/design-deliberation.validation.json" >/tmp/audio-safety-valid.out 2>/tmp/audio-safety-valid.err
jq -e \
  --arg transcript_hash "$TRANSCRIPT_HASH" \
  --arg mp3_hash "$MP3_HASH" \
  '.status == "valid" and .transcriptHash == $transcript_hash and .mp3Sha256 == $mp3_hash' \
  "$TEST_DIR/.intake/audio/design-deliberation.validation.json" >/dev/null \
  || fail "valid audio validation did not record expected hashes"
pass "valid MP3 fixture records transcript and MP3 hashes"

# Test 8: valid audio without transcript hash is rejected.
jq -nc '{status:"complete", outputPath:".tasks/audio/design-deliberation.mp3"}' > "$TEST_DIR/.intake/audio/design-deliberation.status.json"
if WORKSPACE="$TEST_DIR" "$ROOT/intake/scripts/validate-deliberation-audio.sh" \
    "design-deliberation" \
    "$TEST_DIR/.tasks/audio/design-deliberation.mp3" \
    "$TEST_DIR/.intake/audio/design-deliberation.status.json" \
    "$TEST_DIR/.intake/audio/design-deliberation.validation.json" >/tmp/audio-safety-missing-hash.out 2>/tmp/audio-safety-missing-hash.err; then
  fail "valid audio without transcript hash unexpectedly passed validation"
fi
jq -e '.status == "failed" and .reason == "transcript_hash_missing"' "$TEST_DIR/.intake/audio/design-deliberation.validation.json" >/dev/null \
  || fail "missing transcript hash validation did not write failed transcript_hash_missing status"
pass "validation rejects audio without transcript hash"

echo "Audio safety tests passed: $pass_count"
