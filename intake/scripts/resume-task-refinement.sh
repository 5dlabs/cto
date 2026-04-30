#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
TASKS_FILE="${1:-$ROOT/.intake/tmp/refine-tasks-input.json}"
COMPLEXITY_SOURCE="${2:-$ROOT/.intake/tmp/refine-complexity-input.json}"
OUT_DIR="${3:-$ROOT/.intake/tmp/resume-task-refinement}"

[[ -f "$TASKS_FILE" ]] || { echo "resume-task-refinement: tasks file not found: $TASKS_FILE" >&2; exit 1; }
[[ -f "$COMPLEXITY_SOURCE" ]] || { echo "resume-task-refinement: complexity file not found: $COMPLEXITY_SOURCE" >&2; exit 1; }

mkdir -p "$OUT_DIR"

NORMALIZED_COMPLEXITY="$OUT_DIR/refine-complexity-normalized.json"
ARGS_FILE="$OUT_DIR/task-refinement-args.json"
STDOUT_LOG="$OUT_DIR/task-refinement.stdout.log"
STDERR_LOG="$OUT_DIR/task-refinement.stderr.log"

jq -c '
  if type == "array" then .
  elif type == "object" and (.complexityAnalysis | type == "array") then .complexityAnalysis
  else error("resume-task-refinement expected complexity array or .complexityAnalysis array")
  end
' "$COMPLEXITY_SOURCE" > "$NORMALIZED_COMPLEXITY"

ROOT="$ROOT" TASKS_FILE="$TASKS_FILE" COMPLEXITY_FILE="$NORMALIZED_COMPLEXITY" python3 - <<'PY' > "$ARGS_FILE"
import json
import os
from pathlib import Path

root = Path(os.environ["ROOT"])
config = json.loads((root / "cto-config.json").read_text())
tiers = config.get("defaults", {}).get("intake", {}).get("models", {}).get("tiers", {})
committee = config.get("defaults", {}).get("intake", {}).get("models", {}).get("committee", [])

def committee_value(index: int, key: str, default: str) -> str:
    if index < len(committee):
        return committee[index].get(key, default)
    return default

args = {
    "tasks_file": os.environ["TASKS_FILE"],
    "complexity_file": os.environ["COMPLEXITY_FILE"],
    "evaluation_criteria": "task decomposition quality, dependency ordering, decision point coverage",
    "max_revisions": "2",
    "model_primary_provider": tiers.get("primary", {}).get("provider", "gemini"),
    "model_primary": tiers.get("primary", {}).get("model", "gemini-3.1-pro-preview"),
    "voter_1_provider": committee_value(0, "provider", "gemini"),
    "voter_1_model": committee_value(0, "model", "gemini-3.1-pro-preview"),
    "voter_2_provider": committee_value(1, "provider", "gemini"),
    "voter_2_model": committee_value(1, "model", "gemini-3.1-pro-preview"),
    "voter_3_provider": committee_value(2, "provider", "gemini"),
    "voter_3_model": committee_value(2, "model", "gemini-3.1-pro-preview"),
    "audio_debug": "false",
}
print(json.dumps(args))
PY

echo "resume-task-refinement: tasks=$TASKS_FILE" >&2
echo "resume-task-refinement: complexity=$COMPLEXITY_SOURCE -> $NORMALIZED_COMPLEXITY" >&2
echo "resume-task-refinement: args=$ARGS_FILE" >&2
echo "resume-task-refinement: logs=$OUT_DIR" >&2

RUNNER="$OUT_DIR/run-task-refinement.py"
cat > "$RUNNER" <<'PY'
import json
import os
import subprocess
import sys

root = os.environ["ROOT"]
args_file = os.environ["ARGS_FILE"]
stdout_log = os.environ["STDOUT_LOG"]
stderr_log = os.environ["STDERR_LOG"]

with open(args_file, "r", encoding="utf-8") as fh:
    args_json = fh.read()

cmd = [
    "lobster",
    "run",
    "--mode",
    "tool",
    f"{root}/intake/workflows/task-refinement.lobster.yaml",
    "--args-json",
    args_json,
]

with open(stdout_log, "w", encoding="utf-8") as stdout_fh, open(stderr_log, "w", encoding="utf-8") as stderr_fh:
    completed = subprocess.run(
        cmd,
        env={**os.environ, "WORKSPACE": root, "INTAKE_EXPAND_BATCH_SIZE": os.environ.get("INTAKE_EXPAND_BATCH_SIZE", "1")},
        stdout=stdout_fh,
        stderr=stderr_fh,
        text=True,
    )

sys.exit(completed.returncode)
PY

if ! op run --env-file="$ROOT/intake/local.env.op" -- \
  env ROOT="$ROOT" ARGS_FILE="$ARGS_FILE" STDOUT_LOG="$STDOUT_LOG" STDERR_LOG="$STDERR_LOG" \
  INTAKE_EXPAND_BATCH_SIZE="${INTAKE_EXPAND_BATCH_SIZE:-1}" python3 "$RUNNER"; then
  echo "resume-task-refinement: FAILED" >&2
  echo "--- stderr ---" >&2
  tail -80 "$STDERR_LOG" >&2 || true
  echo "--- stdout ---" >&2
  tail -80 "$STDOUT_LOG" >&2 || true
  exit 1
fi

cat "$STDOUT_LOG"
