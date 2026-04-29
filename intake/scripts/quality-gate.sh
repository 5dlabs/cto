#!/usr/bin/env bash
set -euo pipefail

ROOT="${WORKSPACE:-$(cd "$(dirname "$0")/../.." && pwd)}"
STAGE=""
INPUT_FILE=""
MIN_SCORE="${INTAKE_QUALITY_GATE_MIN_SCORE:-7}"
PROVIDER="${INTAKE_QUALITY_GATE_PROVIDER:-}"
MODEL="${INTAKE_QUALITY_GATE_MODEL:-}"
PROMPT_FILE=""

while [ $# -gt 0 ]; do
  case "$1" in
    --stage) STAGE="${2:-}"; shift 2 ;;
    --input-file) INPUT_FILE="${2:-}"; shift 2 ;;
    --min-score) MIN_SCORE="${2:-7}"; shift 2 ;;
    --provider) PROVIDER="${2:-}"; shift 2 ;;
    --model) MODEL="${2:-}"; shift 2 ;;
    --prompt) PROMPT_FILE="${2:-}"; shift 2 ;;
    *) echo "quality-gate: unknown arg: $1" >&2; exit 2 ;;
  esac
done

PROMPT_FILE="${PROMPT_FILE:-$ROOT/intake/prompts/quality-gate-system.md}"

if [ -z "$STAGE" ] || [ -z "$INPUT_FILE" ]; then
  echo "quality-gate: usage: --stage <stage> --input-file <file> [--min-score N] [--provider p --model m]" >&2
  exit 2
fi
if [ ! -s "$INPUT_FILE" ]; then
  echo "quality-gate: input file missing or empty: $INPUT_FILE" >&2
  exit 1
fi

if [ -z "$PROVIDER" ] || [ -z "$MODEL" ]; then
  CFG="$ROOT/cto-config.json"
  PROVIDER="${PROVIDER:-$(jq -r '.defaults.intake.models.tiers.fast.provider // "gemini"' "$CFG" 2>/dev/null || echo gemini)}"
  MODEL="${MODEL:-$(jq -r '.defaults.intake.models.tiers.fast.model // "gemini-2.5-flash"' "$CFG" 2>/dev/null || echo gemini-2.5-flash)}"
fi

TMP_INPUT="$(mktemp "${TMPDIR:-/tmp}/intake-quality-input.XXXXXX.json")"
trap 'rm -f "$TMP_INPUT"' EXIT

python3 - "$INPUT_FILE" "$STAGE" "$MIN_SCORE" > "$TMP_INPUT" <<'PY'
import json, sys
from pathlib import Path

input_path = Path(sys.argv[1])
stage = sys.argv[2]
min_score = int(sys.argv[3])
raw = input_path.read_text(encoding="utf-8", errors="ignore")
max_chars = 14000
payload = {
    "stage": stage,
    "min_score": min_score,
    "content": raw[:max_chars],
    "truncated": len(raw) > max_chars,
    "content_length": len(raw),
}
print(json.dumps(payload))
PY

ARGS="$(jq -n \
  --rawfile prompt "$PROMPT_FILE" \
  --rawfile gate_input "$TMP_INPUT" \
  --arg schema "$ROOT/intake/schemas/quality-gate.schema.json" \
  --arg provider "$PROVIDER" \
  --arg model "$MODEL" \
  '{prompt:$prompt,input:($gate_input|fromjson),schema:$schema,provider:$provider,model:$model}')"

OUT="$("$ROOT/intake/scripts/llm-invoke.sh" --tool llm-task --action json --args-json "$ARGS")" || {
  echo "quality-gate: LLM infrastructure unavailable for stage=$STAGE" >&2
  printf '{"skipped":true,"fallback":true,"reason":"llm_unavailable","stage":"%s"}\n' "$STAGE"
  exit 75
}
PASS="$(printf '%s' "$OUT" | jq -r '.pass // false' 2>/dev/null || echo false)"
SCORE="$(printf '%s' "$OUT" | jq -r '.score // 0' 2>/dev/null || echo 0)"

printf '%s\n' "$OUT"

if [ "$PASS" != "true" ]; then
  echo "quality-gate: FAIL stage=$STAGE pass=$PASS score=$SCORE" >&2
  exit 10
fi
if ! printf '%s' "$SCORE" | jq -e --argjson min "$MIN_SCORE" 'type=="number" and . >= $min' >/dev/null 2>&1; then
  echo "quality-gate: FAIL stage=$STAGE score=$SCORE min=$MIN_SCORE" >&2
  exit 10
fi
