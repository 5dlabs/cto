#!/usr/bin/env bash
# Full deliberate path with Stitch design; still no TTS when audio_debug is false.
# Conservative: single OpenClaw retry, optional INTAKE_NUM_TASKS (default 10), optional fan-out cap.
# Visual verification: tee log + summary on success.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

PROJECT_NAME="${1:-sigma-1}"
REPO_URL="${2:-https://github.com/5dlabs/sigma-1}"
TARGET_REPO_PATH="${3:-/Users/jonathon/5dlabs/sigma-1}"
NUM_TASKS="${INTAKE_NUM_TASKS:-10}"
LOG_FILE="$ROOT/.intake/full-e2e-run.log"

# Generate run ID for observability correlation
export INTAKE_RUN_ID="${PROJECT_NAME}-$(date -u +%Y%m%d-%H%M%S)"
INTAKE_LOG_DIR="$ROOT/.intake/logs"
mkdir -p "$INTAKE_LOG_DIR"

# Emit run_start event
printf '{"ts":"%s","event":"run_start","run_id":"%s","project_name":"%s","mode":"full","deliberate":true,"design_mode":"ingest_plus_stitch","num_tasks":%s}\n' \
  "$(date -u +"%Y-%m-%dT%H:%M:%SZ")" "$INTAKE_RUN_ID" "$PROJECT_NAME" "$NUM_TASKS" \
  >> "$INTAKE_LOG_DIR/runs.jsonl"

rm -f \
  "$ROOT/.intake/intake-sub-workflow.log" \
  "$ROOT/.intake/intake-summary.json" \
  "$ROOT/.intake/audio/architecture-deliberation.status.json" \
  "$ROOT/.intake/audio/architecture-deliberation.log" \
  "$ROOT/.intake/audio/design-deliberation.status.json" \
  "$ROOT/.intake/audio/design-deliberation.log" \
  "$ROOT/.tasks/audio/architecture-deliberation.mp3" \
  "$ROOT/.tasks/audio/architecture-deliberation.transcript.json" \
  "$ROOT/.tasks/audio/design-deliberation.mp3" \
  "$ROOT/.tasks/audio/design-deliberation.transcript.json"
rm -rf "$ROOT/.intake/design" "$ROOT/.tasks/design"

ARGS_JSON="$(jq -nc \
  --arg prd_path ".intake/run-prd.txt" \
  --arg project_name "$PROJECT_NAME" \
  --arg repository_url "$REPO_URL" \
  --arg github_org "5dlabs" \
  --arg deliberate "true" \
  --arg audio_debug "false" \
  --arg design_mode "both" \
  --arg design_provider "both" \
  --arg num_tasks "$NUM_TASKS" \
  --arg target_repo_local_path "$TARGET_REPO_PATH" \
  '{prd_path:$prd_path,project_name:$project_name,repository_url:$repository_url,github_org:$github_org,deliberate:$deliberate,audio_debug:$audio_debug,design_mode:$design_mode,design_provider:$design_provider,num_tasks:$num_tasks,target_repo_local_path:$target_repo_local_path}')"

RUN_START_EPOCH=$(date +%s)

op run --env-file="$ROOT/intake/local.env.op" -- env \
  OPENCLAW_MAX_RETRIES=1 \
  INTAKE_PREFLIGHT_BRIDGES_SKIP=true \
  INTAKE_AUDIO_MUTE=true \
  INTAKE_FAN_OUT_CONCURRENCY="${INTAKE_FAN_OUT_CONCURRENCY:-4}" \
  INTAKE_RUN_ID="$INTAKE_RUN_ID" \
  WORKSPACE="$PWD" \
  lobster run intake/workflows/pipeline.lobster.yaml --args-json "$ARGS_JSON" 2>&1 | tee "$LOG_FILE"

EXIT_CODE=${PIPESTATUS[0]}
RUN_DURATION=$(( $(date +%s) - RUN_START_EPOCH ))

# Emit run_complete event
printf '{"ts":"%s","event":"run_complete","run_id":"%s","project_name":"%s","exit_code":%d,"duration_sec":%d}\n' \
  "$(date -u +"%Y-%m-%dT%H:%M:%SZ")" "$INTAKE_RUN_ID" "$PROJECT_NAME" "$EXIT_CODE" "$RUN_DURATION" \
  >> "$INTAKE_LOG_DIR/runs.jsonl"

echo ""
echo "=== Visual verification (no voice) ==="
echo "  Run ID:     $INTAKE_RUN_ID"
echo "  Duration:   ${RUN_DURATION}s"
echo "  Exit code:  $EXIT_CODE"
echo "  Log:        $LOG_FILE"
echo "  Summary:    $ROOT/.intake/intake-summary.json"
echo "  Tasks dir:  $ROOT/.tasks/docs (when present)"
echo "  Metrics:    $INTAKE_LOG_DIR/ (view in Grafana at http://localhost:3001)"
if [[ -f "$ROOT/.intake/intake-summary.json" ]]; then
  echo "--- intake-summary.json (tail) ---"
  tail -n 20 "$ROOT/.intake/intake-summary.json" || true
fi
