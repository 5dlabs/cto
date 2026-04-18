#!/usr/bin/env bash
set -euo pipefail

WS="${WORKSPACE:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
INTAKE_AGENT_BIN="${INTAKE_AGENT_BIN:-}"

if [ -z "$INTAKE_AGENT_BIN" ]; then
  if command -v intake-agent >/dev/null 2>&1; then
    INTAKE_AGENT_BIN="intake-agent"
  elif [ -x "$WS/apps/intake-agent/dist/intake-agent" ]; then
    INTAKE_AGENT_BIN="$WS/apps/intake-agent/dist/intake-agent"
  elif [ -x "$WS/apps/intake-agent/intake-agent" ]; then
    INTAKE_AGENT_BIN="$WS/apps/intake-agent/intake-agent"
  else
    echo "design-intake-dry-run: intake-agent binary not found." >&2
    exit 1
  fi
fi

BASE_DIR="${1:-$WS/.intake/design-dry-run}"
PRD_CONTENT="${2:-Build a frontend dashboard with reusable components and responsive UX.}"
mkdir -p "$BASE_DIR"

run_mode() {
  local mode="$1"
  local out_dir="$BASE_DIR/$mode"
  mkdir -p "$out_dir"
  echo "== design-intake dry run: $mode =="
  jq -n \
    --arg prd "$PRD_CONTENT" \
    --arg out "$out_dir" \
    --arg mode "$mode" \
    '{
      operation: "design_intake",
      payload: {
        prd_content: $prd,
        project_name: ("dry-run-" + $mode),
        design_prompt: "Validate provider routing and artifact generation.",
        design_provider: $mode,
        output_dir: $out
      }
    }' | "$INTAKE_AGENT_BIN" > "$out_dir/response.json"

  jq -e '.success == true' "$out_dir/response.json" >/dev/null
  test -f "$out_dir/design-context.json"
  test -f "$out_dir/component-library.json"
  test -f "$out_dir/design-system.md"
  echo "ok: $mode"
}

run_mode stitch
run_mode auto

echo "Design intake dry-run complete. Artifacts in: $BASE_DIR"
