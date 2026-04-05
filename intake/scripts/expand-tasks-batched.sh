#!/usr/bin/env bash
# Batched task expansion — splits N tasks into batches of BATCH_SIZE,
# calls openclaw-invoke-retry for each batch, then merges results.
# This avoids LLM output token limits on large task sets.
#
# Usage: expand-tasks-batched.sh --tasks-file <path> --complexity-file <path> \
#          --provider <provider> --model <model> [--batch-size N]
#
# Outputs merged JSON array to stdout.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

TASKS_FILE=""
COMPLEXITY_FILE=""
PROVIDER="github-copilot"
MODEL="claude-opus-4.6"
BATCH_SIZE="${INTAKE_EXPAND_BATCH_SIZE:-2}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tasks-file)     TASKS_FILE="$2";     shift 2 ;;
    --complexity-file) COMPLEXITY_FILE="$2"; shift 2 ;;
    --provider)       PROVIDER="$2";       shift 2 ;;
    --model)          MODEL="$2";          shift 2 ;;
    --batch-size)     BATCH_SIZE="$2";     shift 2 ;;
    *) echo "expand-tasks-batched: unknown arg $1" >&2; exit 1 ;;
  esac
done

[[ -n "$TASKS_FILE" && -f "$TASKS_FILE" ]] || { echo "expand-tasks-batched: --tasks-file required" >&2; exit 1; }
[[ -n "$COMPLEXITY_FILE" && -f "$COMPLEXITY_FILE" ]] || { echo "expand-tasks-batched: --complexity-file required" >&2; exit 1; }

TOTAL_TASKS=$(jq 'length' "$TASKS_FILE")
SCHEMA="$ROOT/intake/schemas/generated-task.schema.json"
PROMPT_FILE="$ROOT/intake/prompts/expand-task-system.md"
TMPDIR_BATCH=$(mktemp -d)
trap 'rm -rf "$TMPDIR_BATCH"' EXIT

validate_expansion_output() {
  local output_file="$1"
  local expected_ids_json="$2"
  local expected_count="$3"
  jq -e \
    --argjson expected_ids "$expected_ids_json" \
    --argjson expected_count "$expected_count" '
      type == "array" and
      length == $expected_count and
      all(.[]; type == "object" and (.id | type) == "number") and
      all(.[]; (.subtasks | type) == "array" and (.subtasks | length) > 0) and
      ((map(.id) | unique | length) == $expected_count) and
      ((map(.id) | sort) == ($expected_ids | sort))
    ' "$output_file" >/dev/null 2>&1
}

echo "expand-tasks-batched: $TOTAL_TASKS tasks, batch_size=$BATCH_SIZE, provider=$PROVIDER, model=$MODEL" >&2

BATCH_IDX=0
OFFSET=0
while [ "$OFFSET" -lt "$TOTAL_TASKS" ]; do
  END=$((OFFSET + BATCH_SIZE))
  if [ "$END" -gt "$TOTAL_TASKS" ]; then END=$TOTAL_TASKS; fi
  COUNT=$((END - OFFSET))

  echo "expand-tasks-batched: batch $BATCH_IDX — tasks $((OFFSET+1))..$END of $TOTAL_TASKS" >&2

  # Extract batch slice
  BATCH_TASKS_FILE="$TMPDIR_BATCH/batch-${BATCH_IDX}-tasks.json"
  BATCH_COMPLEXITY_FILE="$TMPDIR_BATCH/batch-${BATCH_IDX}-complexity.json"
  jq --argjson s "$OFFSET" --argjson e "$END" '.[$s:$e]' "$TASKS_FILE" > "$BATCH_TASKS_FILE"
  jq --argjson s "$OFFSET" --argjson e "$END" '.[$s:$e]' "$COMPLEXITY_FILE" > "$BATCH_COMPLEXITY_FILE"
  EXPECTED_IDS_JSON="$(jq -c 'map(.id)' "$BATCH_TASKS_FILE")"

  ARGS=$(jq -n \
    --rawfile prompt "$PROMPT_FILE" \
    --rawfile tasks_raw "$BATCH_TASKS_FILE" \
    --rawfile complexity_raw "$BATCH_COMPLEXITY_FILE" \
    --arg schema "$SCHEMA" \
    --arg provider "$PROVIDER" \
    --arg model "$MODEL" \
    '{"prompt": $prompt,
      "input": {"tasks": ($tasks_raw | fromjson), "complexity": ($complexity_raw | fromjson)},
      "schema": $schema, "provider": $provider, "model": $model}')

  BATCH_OUT="$TMPDIR_BATCH/batch-${BATCH_IDX}-out.json"
  if "$ROOT/intake/scripts/openclaw-invoke-retry.sh" --tool llm-task --action json --args-json "$ARGS" > "$BATCH_OUT" 2>"$TMPDIR_BATCH/batch-${BATCH_IDX}-stderr.txt"; then
    if ! validate_expansion_output "$BATCH_OUT" "$EXPECTED_IDS_JSON" "$COUNT"; then
      echo "expand-tasks-batched: batch $BATCH_IDX returned invalid/partial expansion payload — falling back to individual" >&2
      rm -f "$BATCH_OUT"
    fi
  else
    echo "expand-tasks-batched: batch $BATCH_IDX invoke failed — falling back to individual" >&2
    rm -f "$BATCH_OUT"
  fi

  if [ ! -f "$BATCH_OUT" ]; then
    echo "expand-tasks-batched: batch $BATCH_IDX FAILED (tasks $((OFFSET+1))..$END), retrying individually" >&2
    # Fallback: try one task at a time
    for i in $(seq "$OFFSET" $((END - 1))); do
      SINGLE_FILE="$TMPDIR_BATCH/single-${i}.json"
      SINGLE_COMPLEXITY_FILE="$TMPDIR_BATCH/single-${i}-complexity.json"
      jq --argjson idx "$i" '[.[$idx]]' "$TASKS_FILE" > "$SINGLE_FILE"
      jq --argjson idx "$i" '[.[$idx]]' "$COMPLEXITY_FILE" > "$SINGLE_COMPLEXITY_FILE"
      SINGLE_ID_JSON="$(jq -c 'map(.id)' "$SINGLE_FILE")"
      SINGLE_ARGS=$(jq -n \
        --rawfile prompt "$PROMPT_FILE" \
        --rawfile tasks_raw "$SINGLE_FILE" \
        --rawfile complexity_raw "$SINGLE_COMPLEXITY_FILE" \
        --arg schema "$SCHEMA" \
        --arg provider "$PROVIDER" \
        --arg model "$MODEL" \
        '{"prompt": $prompt,
          "input": {"tasks": ($tasks_raw | fromjson), "complexity": ($complexity_raw | fromjson)},
          "schema": $schema, "provider": $provider, "model": $model}')
      SINGLE_OUT="$TMPDIR_BATCH/single-${BATCH_IDX}-${i}-out.json"
      if ! "$ROOT/intake/scripts/openclaw-invoke-retry.sh" --tool llm-task --action json --args-json "$SINGLE_ARGS" > "$SINGLE_OUT" 2>/dev/null; then
        echo "expand-tasks-batched: task $((i+1)) individual expand FAILED" >&2
        exit 1
      fi
      if ! validate_expansion_output "$SINGLE_OUT" "$SINGLE_ID_JSON" "1"; then
        echo "expand-tasks-batched: task $((i+1)) individual expand returned invalid payload" >&2
        exit 1
      fi
    done
    # Merge individual outputs for this batch
    jq -s 'add' "$TMPDIR_BATCH"/single-"${BATCH_IDX}"-*-out.json > "$BATCH_OUT"
  fi

  # Validate batch output has expected task count
  GOT=$(jq 'if type == "array" then length else 0 end' "$BATCH_OUT" 2>/dev/null || echo 0)
  if [ "$GOT" -lt "$COUNT" ]; then
    echo "expand-tasks-batched: batch $BATCH_IDX returned $GOT/$COUNT tasks — padding missing with originals" >&2
    # Merge: use expanded tasks for IDs we got, originals for the rest
    ORIG_SLICE="$TMPDIR_BATCH/batch-${BATCH_IDX}-orig.json"
    jq --argjson s "$OFFSET" --argjson e "$END" '.[$s:$e]' "$TASKS_FILE" > "$ORIG_SLICE"
    jq -n --slurpfile batch "$BATCH_OUT" --slurpfile orig "$ORIG_SLICE" '
      ($batch[0] // [] | map({key: (.id | tostring), value: .}) | from_entries) as $expanded
      | ($orig[0] // []) | map(
          (.id | tostring) as $tid
          | if $expanded[$tid] then $expanded[$tid] else . + {subtasks: []} end
        )
    ' > "$TMPDIR_BATCH/batch-${BATCH_IDX}-merged.json"
    mv "$TMPDIR_BATCH/batch-${BATCH_IDX}-merged.json" "$BATCH_OUT"
  fi

  echo "expand-tasks-batched: batch $BATCH_IDX complete — $GOT tasks expanded" >&2

  OFFSET=$END
  BATCH_IDX=$((BATCH_IDX + 1))
done

# Merge all batch outputs
jq -s 'add' "$TMPDIR_BATCH"/batch-*-out.json

echo "expand-tasks-batched: all $TOTAL_TASKS tasks expanded in $BATCH_IDX batches" >&2
