#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
WF="$ROOT/intake/workflows/intake.lobster.yaml"

assert_contains() {
  local needle="$1"
  local label="$2"
  if ! grep -Fq "$needle" "$WF"; then
    echo "not ok - $label" >&2
    echo "missing: $needle" >&2
    exit 1
  fi
  echo "ok - $label"
}

assert_not_contains() {
  local needle="$1"
  local label="$2"
  if grep -Fq "$needle" "$WF"; then
    echo "not ok - $label" >&2
    echo "unexpected: $needle" >&2
    exit 1
  fi
  echo "ok - $label"
}

assert_contains 'WF_OUT_FILE="$root/.intake/step-outputs/generate-workflows.json"' 'generate-workflows writes large payload to a step-output file'
assert_contains 'printf '\''%s'\'' "$WF_OUT_FILE"' 'generate-workflows stdout is only the workflow output path'
assert_contains 'WF_FILE="$CTO_GENERATE_WORKFLOWS_OUT"' 'downstream workflow steps treat generate-workflows stdout as a file path'
assert_contains 'jq '\''.task_workflows | length'\'' "$WF_FILE"' 'validate-workflows reads generated workflow JSON from file'
assert_contains 'intake-util write-files --base-path .tasks/docs --type workflows < "$WF_FILE"' 'write-workflows streams workflow JSON from file'
assert_not_contains 'printf '\''%s'\'' "$CTO_GENERATE_WORKFLOWS_OUT" | jq '\''.task_workflows' 'validate-workflows no longer pipes large workflow JSON through env'
assert_not_contains 'printf '\''%s'\'' "$CTO_GENERATE_WORKFLOWS_OUT" | intake-util write-files --base-path .tasks/docs --type workflows' 'write-workflows no longer pipes large workflow JSON through env'

echo "Workflow output file tests passed"
