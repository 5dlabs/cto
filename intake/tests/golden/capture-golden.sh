#!/usr/bin/env bash
# capture-golden.sh — Capture golden fixture data from a successful pipeline run.
#
# Run this AFTER a successful intake sub-workflow completes.
# It copies step outputs from the lobster run into golden/ for use by step-tests.sh.
#
# Usage: ./capture-golden.sh [workspace-dir]
#   workspace-dir defaults to the repo root (two levels up from this script).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GOLDEN_DIR="$SCRIPT_DIR"
WS="${1:-$(cd "$SCRIPT_DIR/../.." && pwd)}"

echo "Capturing golden fixtures from workspace: $WS"

capture() {
  local name="$1"
  local source="$2"
  if [ -f "$source" ] && [ -s "$source" ]; then
    cp "$source" "$GOLDEN_DIR/$name"
    echo "  captured: $name ($(wc -c < "$source" | tr -d ' ') bytes)"
  else
    echo "  SKIP: $name — source not found or empty: $source"
  fi
}

capture "parse-prd.json"                "$WS/.intake/step-outputs/parse-prd.json"
capture "analyze-complexity.json"       "$WS/.intake/step-outputs/analyze-complexity.json"
capture "refine-tasks.json"             "$WS/.intake/step-outputs/refine-tasks.json"
capture "generate-scaffolds.json"       "$WS/.intake/step-outputs/generate-scaffolds.json"
capture "discover-skills.json"          "$WS/.intake/step-outputs/discover-skills.json"
capture "generate-tool-manifest.json"   "$WS/.intake/step-outputs/generate-tool-manifest.json"
capture "fan-out-docs.json"             "$WS/.intake/step-outputs/fan-out-docs.json"
capture "fan-out-prompts.json"          "$WS/.intake/step-outputs/fan-out-prompts.json"
capture "generate-workflows.json"       "$WS/.intake/step-outputs/generate-workflows.json"
capture "generate-scale-tasks.json"     "$WS/.intake/step-outputs/generate-scale-tasks.json"
capture "generate-security-report.json" "$WS/.intake/step-outputs/generate-security-report.json"
capture "generate-remediation-tasks.json" "$WS/.intake/step-outputs/generate-remediation-tasks.json"

echo ""
echo "To use manually: copy step stdout files into golden/ with the names above."
echo "The step-outputs/ directory is populated when the pipeline runs with CAPTURE_GOLDEN=1."
