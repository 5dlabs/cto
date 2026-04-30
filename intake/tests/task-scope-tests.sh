#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

pass_count=0
pass() {
  pass_count=$((pass_count + 1))
  echo "ok - $*"
}

fail() {
  echo "not ok - $*" >&2
  exit 1
}

PIPELINE="$ROOT/intake/workflows/pipeline.lobster.yaml"
INTAKE="$ROOT/intake/workflows/intake.lobster.yaml"
SYSTEM_PROMPT="$ROOT/intake/prompts/parse-prd-system.md"
USER_PROMPT="$ROOT/intake/prompts/parse-prd-user.md"

check_common_markers() {
  local file="$1" label="$2"
  python3 - "$file" "$label" <<'PY'
import sys
path, label = sys.argv[1], sys.argv[2]
text = open(path).read()
required = [
    'EFFECTIVE_NUM_TASKS_LC',
    'EFFECTIVE_NUM_TASKS_MODE="auto"',
    'for large platforms this is often 15-40 top-level tasks',
    '--argjson num_tasks_raw "$EFFECTIVE_NUM_TASKS"',
]
missing = [item for item in required if item not in text]
if missing:
    print(f'missing {label} task auto-scale markers: ' + ', '.join(missing), file=sys.stderr)
    sys.exit(1)
PY
  pass "$label normalizes literal auto task counts"
}

check_common_markers "$PIPELINE" "pipeline"
check_common_markers "$INTAKE" "intake sub-workflow"

python3 - "$PIPELINE" "$INTAKE" <<'PY'
import sys
for path in sys.argv[1:]:
    text = open(path).read()
    required = [
        'INTAKE_AUTO_MIN_TASKS',
        'FATAL — auto task decomposition returned',
        'refusing to collapse a large project into tiny umbrella tasks',
        'grep -Eio',
        'scope signals',
    ]
    missing = [item for item in required if item not in text]
    if missing:
        print(f'missing hard minimum task-count gate markers in {path}: ' + ', '.join(missing), file=sys.stderr)
        sys.exit(1)
PY
pass "pipeline and intake sub-workflow reject tiny task lists for large PRDs"

python3 - "$INTAKE" <<'PY'
import sys
text = open(sys.argv[1]).read()
required = [
    'EFFECTIVE_NUM_TASKS_MODE:-fixed}" != "auto"',
    'using fallback from $FALLBACK_TASKS_FILE',
]
missing = [item for item in required if item not in text]
if missing:
    print('missing stale fallback protection markers: ' + ', '.join(missing), file=sys.stderr)
    sys.exit(1)
PY
pass "intake sub-workflow does not use stale fallback tasks in auto mode"

if grep -q 'often 15-40 top-level tasks' "$SYSTEM_PROMPT"; then
  pass "system prompt discourages tiny umbrella task lists in auto mode"
else
  fail "system prompt missing large-scope auto task guidance"
fi

if grep -q 'one task per major deployable feature slice' "$USER_PROMPT"; then
  pass "user prompt carries explicit scope-based task count policy"
else
  fail "user prompt missing scope-based task count policy"
fi

echo "Task scope tests passed: $pass_count"
