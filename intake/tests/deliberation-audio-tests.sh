#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
pass_count=0
pass() { pass_count=$((pass_count + 1)); echo "ok - $*"; }
fail() { echo "not ok - $*" >&2; exit 1; }

WORKFLOW="$ROOT/intake/workflows/deliberation.lobster.yaml"
SPEAKERS="$ROOT/apps/lobster-voice/src/speakers.ts"

python3 - "$WORKFLOW" <<'PY'
import sys
text=open(sys.argv[1]).read()
step=text.split('- id: write-architecture-audio-transcript',1)[1].split('\n  - id:',1)[0]
required=[
  "'speaker': 'morgan'",
  "'voiceProfile': 'morgan-committee-v1'",
  "def role(raw: object)",
  "return 'optimus'",
  "return 'pessimus'",
  "Back to you Morgan.",
  "I am Morgan.",
  "I am {label_for(speaker)}.",
  "no Praxis or sample transcript content was substituted",
]
missing=[r for r in required if r not in step]
if missing:
    print('missing Morgan deliberation transcript markers: '+', '.join(missing), file=sys.stderr)
    sys.exit(1)
PY
pass "architecture transcript writer is Morgan-led and handoff-aware"

python3 - "$SPEAKERS" <<'PY'
import sys
text=open(sys.argv[1]).read()
required={
  'morgan':'iP95p4xoKVk53GoZ742B',
  'optimus':'TX3LPaxmHKxFdv7VOQHJ',
  'pessimus':'cjVigY5qzO86Huf0OWal',
  'praxis':'pNInz6obpgDQGcFmaJgB',
  'rook':'onwK4e9ZLuTAKqWW03F9',
  'veritas':'XrExE9yKIg1WjnnlVkGX',
}
missing=[]
for speaker, voice in required.items():
    if f'| "{speaker}"' not in text and f'  {speaker}: {{' not in text:
        missing.append(speaker)
    if voice not in text:
        missing.append(f'{speaker}:{voice}')
if missing:
    print('missing selected committee voice markers: '+', '.join(missing), file=sys.stderr)
    sys.exit(1)
PY
pass "lobster-voice includes selected Morgan committee voices"

echo "Deliberation audio tests passed: $pass_count"
