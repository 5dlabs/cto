#!/bin/sh
set -eu

HERMES_BIN="${HERMES_BIN:-/opt/hermes/.venv/bin/hermes}"
HERMES_HOME_DIR="${HERMES_HOME:-/opt/data}"
STATE_FILE="${HERMES_HOME_DIR}/gateway_state.json"

if ! pgrep -f 'hermes.*gateway' >/dev/null 2>&1; then
  echo "hermes gateway process not found" >&2
  exit 1
fi

if [ ! -s "$STATE_FILE" ]; then
  echo "gateway state file missing: $STATE_FILE" >&2
  exit 1
fi

"$HERMES_BIN" gateway status 2>/dev/null | grep -Eq 'Gateway is running|running manually|running with|✓|connected' || {
  echo "hermes gateway status is not healthy" >&2
  "$HERMES_BIN" gateway status >&2 || true
  exit 1
}

"/opt/hermes/.venv/bin/python" - "$STATE_FILE" <<'PY'
import json, sys
from pathlib import Path
state_path = Path(sys.argv[1])
try:
    state = json.loads(state_path.read_text())
except Exception as exc:
    print(f"cannot parse gateway state: {exc}", file=sys.stderr)
    raise SystemExit(1)
if state.get("gateway_state") != "running":
    print(f"gateway_state is {state.get('gateway_state')!r}", file=sys.stderr)
    raise SystemExit(1)
platforms = state.get("platforms") or {}
discord = platforms.get("discord") or {}
if discord.get("state") != "connected":
    print(f"discord platform state is {discord.get('state')!r}", file=sys.stderr)
    raise SystemExit(1)
PY
