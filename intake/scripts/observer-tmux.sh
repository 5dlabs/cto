#!/usr/bin/env bash
# Launch a tmux observer grid for intake runs.
# Monitors local bridge logs + latest pipeline log + optional cluster logs.
#
# Usage:
#   ./intake/scripts/observer-tmux.sh
#   ./intake/scripts/observer-tmux.sh --session intake-observer
#   ./intake/scripts/observer-tmux.sh --attach
#   ./intake/scripts/observer-tmux.sh --no-cluster
#
# Notes:
# - Reads from local files only; does not modify state.
# - Safe to run before/after go-green.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
SESSION="intake-observer"
ATTACH=0
CLUSTER=1

while [[ $# -gt 0 ]]; do
  case "$1" in
    --session)
      SESSION="${2:-}"
      shift 2
      ;;
    --attach)
      ATTACH=1
      shift
      ;;
    --no-cluster)
      CLUSTER=0
      shift
      ;;
    -h|--help)
      sed -n '2,24p' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *)
      echo "observer-tmux.sh: unknown arg: $1" >&2
      exit 1
      ;;
  esac
done

command -v tmux >/dev/null 2>&1 || {
  echo "observer-tmux.sh: tmux not found on PATH" >&2
  exit 1
}

if tmux has-session -t "$SESSION" 2>/dev/null; then
  echo "observer-tmux.sh: session already exists: $SESSION" >&2
  [[ "$ATTACH" -eq 1 ]] && exec tmux attach -t "$SESSION"
  echo "observer-tmux.sh: attach with: tmux attach -t $SESSION" >&2
  exit 0
fi

LOGD="$ROOT/intake/.bridge-logs"
mkdir -p "$LOGD"

# Pane command helpers.
discord_cmd="bash -lc 'echo \"[discord-bridge] $LOGD/discord-bridge.log\"; touch \"$LOGD/discord-bridge.log\"; tail -F \"$LOGD/discord-bridge.log\"'"
linear_cmd="bash -lc 'echo \"[linear-bridge] $LOGD/linear-bridge.log\"; touch \"$LOGD/linear-bridge.log\"; tail -F \"$LOGD/linear-bridge.log\"'"
pipeline_cmd="bash -lc 'echo \"[pipeline] tail latest _runs/**/pipeline*.log\"; while true; do f=\$(ls -1t \"$ROOT\"/_runs/*/pipeline*.log 2>/dev/null | sed -n 1p); if [[ -n \"\$f\" ]]; then echo \"--- following: \$f ---\"; tail -n 120 -F \"\$f\"; else echo \"no pipeline log yet; sleeping...\"; sleep 3; fi; done'"
gateway_cmd="bash -lc 'echo \"[openclaw gateway] expecting /tmp/openclaw-gateway.log\"; if [[ -f /tmp/openclaw-gateway.log ]]; then tail -F /tmp/openclaw-gateway.log; else echo \"no /tmp/openclaw-gateway.log yet\"; echo \"tip: openclaw gateway 2>&1 | tee /tmp/openclaw-gateway.log\"; while true; do sleep 15; done; fi'"
pods_cmd="bash -lc 'echo \"[pods] openclaw + bridge pods\"; while true; do date; kubectl get pods -A 2>/dev/null | rg -i \"openclaw|discord-bridge|linear-bridge\" || true; echo; sleep 15; done'"
conductor_cmd="bash -lc 'echo \"[conductor logs] attempting kubectl logs -n openclaw deploy/openclaw-conductor\"; while true; do kubectl logs -n openclaw deploy/openclaw-conductor --tail=120 -f 2>/dev/null || true; echo \"conductor stream unavailable; retrying in 5s\"; sleep 5; done'"

tmux new-session -d -s "$SESSION" -n observer "$discord_cmd"
tmux split-window -t "$SESSION:observer" -h "$linear_cmd"
tmux split-window -t "$SESSION:observer.0" -v "$pipeline_cmd"
tmux split-window -t "$SESSION:observer.1" -v "$gateway_cmd"

if [[ "$CLUSTER" -eq 1 ]]; then
  tmux new-window -t "$SESSION" -n cluster "$pods_cmd"
  tmux split-window -t "$SESSION:cluster" -h "$conductor_cmd"
fi

tmux select-layout -t "$SESSION:observer" tiled >/dev/null 2>&1 || true
tmux set-option -t "$SESSION" -g mouse on >/dev/null 2>&1 || true

echo "observer-tmux.sh: started tmux session: $SESSION" >&2
echo "observer-tmux.sh: attach with: tmux attach -t $SESSION" >&2
if [[ "$ATTACH" -eq 1 ]]; then
  exec tmux attach -t "$SESSION"
fi
