#!/usr/bin/env bash
set -euo pipefail

WS="${WORKSPACE:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
OUT_ROOT="${DESIGN_POC_OUT:-$WS/.intake/optimizer-poc/design-intake/$(date +%Y%m%d-%H%M%S)}"
RUNS="${DESIGN_POC_RUNS:-1}"
BASELINE_PROVIDER="${DESIGN_POC_BASELINE_PROVIDER:-stitch}"
CANDIDATE_PROVIDER="${DESIGN_POC_CANDIDATE_PROVIDER:-both}"
KEEP_THRESHOLD="${DESIGN_POC_KEEP_THRESHOLD:-5}"
FRAMER_PROJECT_TARGET="${DESIGN_FRAMER_PROJECT:-${FRAMER_PROJECT_URL:-${FRAMER_PROJECT_ID:-}}}"
INTAKE_AGENT_BIN="${INTAKE_AGENT_BIN:-}"

if ! command -v jq >/dev/null 2>&1; then
  echo "design-intake-optimizer-poc: jq is required." >&2
  exit 1
fi

if [ -z "$INTAKE_AGENT_BIN" ]; then
  if command -v intake-agent >/dev/null 2>&1; then
    INTAKE_AGENT_BIN="intake-agent"
  elif [ -x "$WS/apps/intake-agent/dist/intake-agent" ]; then
    INTAKE_AGENT_BIN="$WS/apps/intake-agent/dist/intake-agent"
  elif [ -x "$WS/apps/intake-agent/intake-agent" ]; then
    INTAKE_AGENT_BIN="$WS/apps/intake-agent/intake-agent"
  else
    echo "design-intake-optimizer-poc: intake-agent binary not found." >&2
    exit 1
  fi
fi

if [[ ! "$RUNS" =~ ^[0-9]+$ ]] || [ "$RUNS" -lt 1 ]; then
  echo "design-intake-optimizer-poc: DESIGN_POC_RUNS must be >= 1." >&2
  exit 1
fi

if [[ ! "$KEEP_THRESHOLD" =~ ^-?[0-9]+(\.[0-9]+)?$ ]]; then
  echo "design-intake-optimizer-poc: DESIGN_POC_KEEP_THRESHOLD must be numeric." >&2
  exit 1
fi

if [ -n "${DESIGN_POC_PRD_PATH:-}" ] && [ -f "${DESIGN_POC_PRD_PATH}" ]; then
  PRD_CONTENT="$(<"${DESIGN_POC_PRD_PATH}")"
elif [ -f "$WS/.intake/run-prd.txt" ]; then
  PRD_CONTENT="$(<"$WS/.intake/run-prd.txt")"
else
  PRD_CONTENT="Build a frontend dashboard with reusable components, responsive behavior, and clear information hierarchy."
fi

DESIGN_PROMPT="${DESIGN_POC_PROMPT:-Modernize UI while preserving product tone and implementation realism.}"
DESIGN_URLS="${DESIGN_POC_URLS:-}"

mkdir -p "$OUT_ROOT/runs"
RUN_MANIFEST="$OUT_ROOT/run-manifest.json"

jq -n \
  --arg ts "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg ws "$WS" \
  --arg out "$OUT_ROOT" \
  --arg baseline "$BASELINE_PROVIDER" \
  --arg candidate "$CANDIDATE_PROVIDER" \
  --argjson runs "$RUNS" \
  --arg threshold "$KEEP_THRESHOLD" \
  --arg bin "$INTAKE_AGENT_BIN" \
  '{
    timestamp: $ts,
    workspace: $ws,
    output_root: $out,
    intake_agent_bin: $bin,
    baseline_provider: $baseline,
    candidate_provider: $candidate,
    runs_per_variant: $runs,
    keep_threshold: ($threshold | tonumber)
  }' > "$RUN_MANIFEST"

score_run() {
  local variant="$1"
  local run_dir="$2"
  local started_at_epoch="$3"
  local ended_at_epoch="$4"
  local exit_code="$5"
  local run_index="$6"
  local response_file="$run_dir/response.json"
  local context_file="$run_dir/design-context.json"
  local metrics_file="$run_dir/metrics.json"
  local duration_s=$((ended_at_epoch - started_at_epoch))
  local response_tmp="$run_dir/response.safe.json"
  local context_tmp="$run_dir/context.safe.json"

  if [ -s "$response_file" ]; then
    cp "$response_file" "$response_tmp"
  else
    printf '{}\n' > "$response_tmp"
  fi
  if [ -s "$context_file" ]; then
    cp "$context_file" "$context_tmp"
  else
    printf '{}\n' > "$context_tmp"
  fi

  jq -n \
    --slurpfile resp "$response_tmp" \
    --slurpfile ctx "$context_tmp" \
    --arg variant "$variant" \
    --argjson run_index "$run_index" \
    --argjson duration_s "$duration_s" \
    --argjson exit_code "$exit_code" '
    ($resp[0] // {}) as $resp |
    ($ctx[0] // {}) as $ctx |
    ($resp.success == true and $exit_code == 0 and ($ctx | type == "object")) as $ok |
    ($ctx.providers // {}) as $providers |
    ($providers
      | to_entries
      | map(select(.value.status == "generated"))
      | length) as $generated_providers |
    ($ctx.normalized_candidates // []) as $candidates |
    ($candidates | map(select(.status == "generated")) | length) as $generated_candidates |
    ($candidates | map(select(.status == "failed")) | length) as $failed_candidates |
    (([$providers[]?.warnings[]?] | length) // 0) as $warnings_count |
    ([
      ($ctx.component_library?.path // empty),
      ($ctx.design_system?.path // empty)
    ] | map(select(length > 0)) | length) as $declared_artifacts |
    ({
      variant: $variant,
      run_index: $run_index,
      exit_code: $exit_code,
      success: $ok,
      duration_s: $duration_s,
      generated_providers: $generated_providers,
      generated_candidates: $generated_candidates,
      failed_candidates: $failed_candidates,
      warnings_count: $warnings_count,
      declared_artifacts: $declared_artifacts
    }) as $m |
    $m + {
      score:
        (
          (if $m.success then 100 else 0 end) +
          ($m.generated_providers * 20) +
          ($m.generated_candidates * 5) +
          ($m.declared_artifacts * 10) -
          ($m.failed_candidates * 3) -
          ($m.warnings_count * 2) -
          ($m.duration_s * 0.2)
        )
    }
  ' > "$metrics_file"
}

run_variant() {
  local variant_name="$1"
  local provider_mode="$2"
  local variant_dir="$OUT_ROOT/runs/$variant_name"
  mkdir -p "$variant_dir"

  local i
  for ((i = 1; i <= RUNS; i++)); do
    local run_dir="$variant_dir/run-$i"
    mkdir -p "$run_dir"
    local started_at_epoch
    local ended_at_epoch
    local exit_code
    started_at_epoch="$(date +%s)"

    jq -n \
      --arg prd "$PRD_CONTENT" \
      --arg out "$run_dir" \
      --arg mode "$provider_mode" \
      --arg framer_project "$FRAMER_PROJECT_TARGET" \
      --arg project_name "optimizer-poc-${variant_name}-${i}" \
      --arg prompt "$DESIGN_PROMPT" \
      --arg urls "$DESIGN_URLS" '
      {
        operation: "design_intake",
        payload: {
          prd_content: $prd,
          project_name: $project_name,
          design_prompt: $prompt,
          design_provider: $mode,
          design_framer_project: $framer_project,
          design_urls: $urls,
          output_dir: $out
        }
      }' > "$run_dir/request.json"

    if "$INTAKE_AGENT_BIN" < "$run_dir/request.json" > "$run_dir/response.json" 2> "$run_dir/stderr.log"; then
      exit_code=0
    else
      exit_code=$?
    fi
    ended_at_epoch="$(date +%s)"

    score_run "$variant_name" "$run_dir" "$started_at_epoch" "$ended_at_epoch" "$exit_code" "$i"
  done
}

run_variant baseline "$BASELINE_PROVIDER"
run_variant candidate "$CANDIDATE_PROVIDER"

SUMMARY_JSON="$OUT_ROOT/summary.json"
REPORT_MD="$OUT_ROOT/report.md"

jq -s \
  --arg baseline "$BASELINE_PROVIDER" \
  --arg candidate "$CANDIDATE_PROVIDER" \
  --argjson threshold "$KEEP_THRESHOLD" '
  def avg($arr): if ($arr | length) == 0 then 0 else (($arr | add) / ($arr | length)) end;
  . as $runs |
  ($runs | map(select(.variant == "baseline"))) as $b |
  ($runs | map(select(.variant == "candidate"))) as $c |
  ({
    baseline: {
      provider_mode: $baseline,
      runs: ($b | length),
      successful_runs: ($b | map(select(.success == true)) | length),
      avg_score: avg($b | map(.score)),
      avg_duration_s: avg($b | map(.duration_s)),
      avg_generated_candidates: avg($b | map(.generated_candidates))
    },
    candidate: {
      provider_mode: $candidate,
      runs: ($c | length),
      successful_runs: ($c | map(select(.success == true)) | length),
      avg_score: avg($c | map(.score)),
      avg_duration_s: avg($c | map(.duration_s)),
      avg_generated_candidates: avg($c | map(.generated_candidates))
    },
    score_delta: (avg($c | map(.score)) - avg($b | map(.score))),
    keep_threshold: $threshold
  }) as $s |
  $s + {
    decision: (
      if ($s.baseline.successful_runs == 0 and $s.candidate.successful_runs == 0) then "NO_DECISION"
      elif ($s.candidate.successful_runs == 0) then "DISCARD"
      elif ($s.baseline.successful_runs == 0) then "KEEP"
      elif $s.score_delta >= $threshold then "KEEP"
      else "DISCARD"
      end
    )
  }
' "$OUT_ROOT"/runs/*/run-*/metrics.json > "$SUMMARY_JSON"

jq -r '
  [
    "# Design Intake Optimizer POC Report",
    "",
    "## Providers",
    "- baseline: \(.baseline.provider_mode)",
    "- candidate: \(.candidate.provider_mode)",
    "",
    "## Aggregate",
    "- baseline avg score: \(.baseline.avg_score)",
    "- candidate avg score: \(.candidate.avg_score)",
    "- score delta (candidate - baseline): \(.score_delta)",
    "- threshold: \(.keep_threshold)",
    "- decision: \(.decision)",
    "",
    "## Operational",
    "- baseline successful runs: \(.baseline.successful_runs)/\(.baseline.runs)",
    "- candidate successful runs: \(.candidate.successful_runs)/\(.candidate.runs)",
    "- baseline avg duration (s): \(.baseline.avg_duration_s)",
    "- candidate avg duration (s): \(.candidate.avg_duration_s)",
    "- baseline avg generated candidates: \(.baseline.avg_generated_candidates)",
    "- candidate avg generated candidates: \(.candidate.avg_generated_candidates)"
  ] | join("\n")
' "$SUMMARY_JSON" > "$REPORT_MD"

echo "POC complete."
echo "Manifest: $RUN_MANIFEST"
echo "Summary:  $SUMMARY_JSON"
echo "Report:   $REPORT_MD"
echo "Decision: $(jq -r '.decision' "$SUMMARY_JSON")"
