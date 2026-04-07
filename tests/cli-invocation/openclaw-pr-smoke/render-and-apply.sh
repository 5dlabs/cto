#!/usr/bin/env bash
# Render and kubectl-apply one CodeRun CRD per agent listed in agents.txt.
# Usage: ./render-and-apply.sh [--dry-run]
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
TPL="$HERE/coderun.yaml.tpl"
AGENTS_FILE="$HERE/agents.txt"
OUT_DIR="$HERE/rendered"
DRY_RUN=0
if [[ "${1:-}" == "--dry-run" ]]; then DRY_RUN=1; fi

mkdir -p "$OUT_DIR"
rm -f "$OUT_DIR"/*.yaml

while IFS= read -r agent; do
  [[ -z "$agent" || "$agent" =~ ^# ]] && continue
  cap="$(echo "$agent" | awk '{print toupper(substr($0,1,1)) tolower(substr($0,2))}')"
  upper="$(echo "$agent" | tr '[:lower:]' '[:upper:]')"
  out="$OUT_DIR/${agent}.yaml"
  sed \
    -e "s/__AGENT_CAPITAL__/$cap/g" \
    -e "s/__AGENT_UPPER__/$upper/g" \
    -e "s/__AGENT__/$agent/g" \
    "$TPL" > "$out"
  echo "rendered: $out"
done < "$AGENTS_FILE"

if [[ $DRY_RUN -eq 1 ]]; then
  echo "--dry-run: not applying"
  exit 0
fi

for f in "$OUT_DIR"/*.yaml; do
  echo "applying: $f"
  kubectl --context=ovh-cluster -n cto apply -f "$f"
done

echo "done. watch with:"
echo "  kubectl --context=ovh-cluster -n cto get coderun -l smoke-wave=openclaw-pr -w"
