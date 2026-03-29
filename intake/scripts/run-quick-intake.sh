#!/usr/bin/env bash
# Conservative fast path: no TTS (audio_debug false + INTAKE_AUDIO_MUTE), stub design agent,
# single OpenClaw retry. Task count is determined by the LLM based on project scope
# (override with INTAKE_NUM_TASKS if needed).
# Visual verification: tee log + summary paths printed on success.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

PROJECT_NAME="${1:-sigma-1}"
REPO_URL="${2:-https://github.com/5dlabs/sigma-1}"
TARGET_REPO_PATH="${3:-/Users/jonathon/5dlabs/sigma-1}"
NUM_TASKS="${INTAKE_NUM_TASKS:-0}"
LOG_FILE="$ROOT/.intake/quick-intake-run.log"

rm -f "$ROOT/.intake/intake-sub-workflow.log" "$ROOT/.intake/intake-summary.json"

ARGS_JSON="$(jq -nc \
  --arg prd_path ".intake/run-prd.txt" \
  --arg project_name "$PROJECT_NAME" \
  --arg repository_url "$REPO_URL" \
  --arg github_org "5dlabs" \
  --arg deliberate "false" \
  --arg audio_debug "false" \
  --arg design_mode "ingest" \
  --argjson num_tasks "$NUM_TASKS" \
  --arg target_repo_local_path "$TARGET_REPO_PATH" \
  '{prd_path:$prd_path,project_name:$project_name,repository_url:$repository_url,github_org:$github_org,deliberate:$deliberate,audio_debug:$audio_debug,design_mode:$design_mode,num_tasks:$num_tasks,target_repo_local_path:$target_repo_local_path}')"

op run --env-file="$ROOT/intake/local.env.op" -- env \
  OPENCLAW_MAX_RETRIES=1 \
  INTAKE_PREFLIGHT_BRIDGES_SKIP=true \
  INTAKE_AUDIO_MUTE=true \
  INTAKE_FAN_OUT_CONCURRENCY="${INTAKE_FAN_OUT_CONCURRENCY:-2}" \
  INTAKE_AGENT_BIN=cat \
  INLINE_VOTE_TIMEOUT_SEC=90 \
  INTAKE_STRICT_CONTENT_GATES="${INTAKE_STRICT_CONTENT_GATES:-true}" \
  INTAKE_ENABLE_LLM_GATES="${INTAKE_ENABLE_LLM_GATES:-true}" \
  WORKSPACE="$PWD" \
  lobster run intake/workflows/pipeline.lobster.yaml --args-json "$ARGS_JSON" 2>&1 | tee "$LOG_FILE"

echo ""
echo "=== Visual verification (no voice) ==="
echo "  Log:        $LOG_FILE"
echo "  Summary:    $ROOT/.intake/intake-summary.json"
echo "  Tasks dir:  $ROOT/.tasks/docs (when present)"
if [[ -f "$ROOT/.intake/intake-summary.json" ]]; then
  echo "--- intake-summary.json (tail) ---"
  tail -n 20 "$ROOT/.intake/intake-summary.json" || true
fi
