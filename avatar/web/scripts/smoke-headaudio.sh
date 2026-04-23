#!/usr/bin/env bash
# Smoke test for HeadAudio lip-sync wiring (WS-A).
#
# Does NOT run a browser or verify visemes — it only catches the static /
# build-time ways this pipeline can silently break:
#
#   1. Worklet + ML model assets are present under public/headaudio/.
#   2. LiveKitAudioBridge is mounted in Room.tsx.
#   3. TalkingHeadView registers the worklet on head.audioCtx and loads
#      the model.
#   4. `pnpm build` succeeds.
#
# See avatar/web/docs/headaudio-verification.md for the full checklist.

set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$here"

fail=0
pass() { printf '  \033[32mok\033[0m   %s\n' "$1"; }
miss() { printf '  \033[31mFAIL\033[0m %s\n' "$1"; fail=1; }

echo "==> Static assets under public/headaudio/"
for f in public/headaudio/headworklet.min.mjs public/headaudio/model-en-mixed.bin; do
  if [[ -f "$f" ]]; then
    size=$(wc -c <"$f" | tr -d ' ')
    pass "$f (${size} bytes)"
  else
    miss "$f missing — run 'pnpm install' or 'pnpm run copy:headaudio'"
  fi
done

echo
echo "==> Room.tsx wires LiveKitAudioBridge with talkingHeadRef"
if grep -q 'import LiveKitAudioBridge from "@/components/LiveKitAudioBridge"' components/Room.tsx \
   && grep -q '<LiveKitAudioBridge talkingHeadRef={talkingHeadRef}' components/Room.tsx; then
  pass "LiveKitAudioBridge imported + mounted"
else
  miss "LiveKitAudioBridge not wired in components/Room.tsx"
fi

echo
echo "==> TalkingHeadView registers worklet + loads model on head.audioCtx"
thv=components/TalkingHeadView.tsx
checks=(
  'head.audioCtx.audioWorklet.addModule'
  'headAudio.loadModel'
  '/headaudio/headworklet.min.mjs'
  '/headaudio/model-en-mixed.bin'
  'head.mtAvatar\[key\]'
  'createMediaStreamSource'
)
for pat in "${checks[@]}"; do
  if grep -Eq "$pat" "$thv"; then
    pass "$thv contains /$pat/"
  else
    miss "$thv missing /$pat/"
  fi
done

echo
echo "==> LiveKitAudioBridge uses useVoiceAssistant + mediaStreamTrack"
bridge=components/LiveKitAudioBridge.tsx
for pat in 'useVoiceAssistant' 'mediaStreamTrack' 'attachAudio' 'detachAudio'; do
  if grep -q "$pat" "$bridge"; then
    pass "$bridge references $pat"
  else
    miss "$bridge missing $pat"
  fi
done

echo
echo "==> Voice-bridge deployment has alignment disabled (HeadAudio path)"
dep=../../infra/manifests/voice-bridge/deployment.yaml
if [[ -f "$dep" ]]; then
  if grep -q 'VOICE_BRIDGE_ENABLE_ALIGNMENT' "$dep" \
     && grep -A1 'VOICE_BRIDGE_ENABLE_ALIGNMENT' "$dep" | grep -q '"0"\|: 0\| 0$'; then
    pass "$dep has VOICE_BRIDGE_ENABLE_ALIGNMENT=0"
  else
    miss "$dep does not pin VOICE_BRIDGE_ENABLE_ALIGNMENT=0 (WS-A expects alignment OFF)"
  fi
else
  printf '  \033[33mskip\033[0m %s not found (ok if manifests live elsewhere)\n' "$dep"
fi

if [[ $fail -ne 0 ]]; then
  echo
  echo "Static checks failed. Fix before running 'pnpm build'." >&2
  exit 1
fi

echo
echo "==> pnpm build"
if command -v pnpm >/dev/null 2>&1; then
  pnpm build
else
  echo "pnpm not found, falling back to npm run build" >&2
  npm run build
fi

echo
echo "All HeadAudio smoke checks passed."
echo "Next: launch 'pnpm dev', port-forward LiveKit, and follow"
echo "  avatar/web/docs/headaudio-verification.md §2–§3"
echo "to eyeball visemes."
