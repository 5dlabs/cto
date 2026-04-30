#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PIPELINE="$ROOT/intake/workflows/pipeline.lobster.yaml"
python3 - "$PIPELINE" <<'PY'
import sys
text=open(sys.argv[1]).read()
step=text.split('- id: materialize-design-inputs',1)[1].split('\n  - id:',1)[0]
required=[
  'stitch) REQUIRES_STITCH=true',
  'auto) REQUIRES_STITCH=false',
  'provider auto will use local Stitch-style bundle generation',
]
missing=[x for x in required if x not in step]
if missing:
    print('missing auto design fallback markers: '+', '.join(missing), file=sys.stderr)
    sys.exit(1)
PY
echo 'ok - design provider auto does not require Stitch credentials'
