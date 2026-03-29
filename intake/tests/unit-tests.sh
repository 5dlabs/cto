#!/usr/bin/env bash
# Unit Tests for intake-util CLI commands.
#
# Tests deterministic (non-network) commands by piping fixture data
# and checking exit codes + output structure.
#
# Requires: bun (for running intake-util), python3

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FIXTURES_DIR="$SCRIPT_DIR/fixtures"

# Find bun
BUN=$(command -v bun 2>/dev/null || echo "$HOME/.bun/bin/bun")
if [ ! -x "$BUN" ]; then
  echo "ERROR: bun not found. Install from https://bun.sh"
  exit 1
fi
INTAKE_UTIL="$BUN run $SCRIPT_DIR/../../apps/intake-util/src/index.ts"

PASS=0
FAIL=0
ERRORS=""

pass() {
  echo "  PASS: $1"
  PASS=$((PASS + 1))
}

fail() {
  echo "  FAIL: $1"
  ERRORS="${ERRORS}\n--- $1 ---\n${2:-unknown}\n"
  FAIL=$((FAIL + 1))
}

# JSON helpers — use piping to avoid escaping issues
jf() {
  # Usage: echo "$json" | jf "field_name"  → prints field value
  python3 -c "import json,sys; d=json.load(sys.stdin); v=d['$1']; print(str(v).lower() if isinstance(v,bool) else str(v))"
}

jvalid() {
  # Usage: echo "$json" | jvalid  → exit 0 if valid==true
  python3 -c "import json,sys; d=json.load(sys.stdin); sys.exit(0 if d.get('valid')==True else 1)"
}

jlen() {
  # Usage: echo "$json" | jlen  → prints array length
  python3 -c "import json,sys; print(len(json.load(sys.stdin)))"
}

jidx() {
  # Usage: echo "$json" | jidx 0 field  → prints arr[0][field]
  python3 -c "import json,sys; d=json.load(sys.stdin); print(d[$1]['$2'])"
}

# =========================================================================
# validate --type tasks
# =========================================================================
echo "=== validate --type tasks ==="

if output=$(echo '[{"id":1,"title":"T","description":"D","dependencies":[]}]' | $INTAKE_UTIL validate --type tasks 2>&1); then
  echo "$output" | jvalid && pass "valid tasks → valid:true" || fail "valid tasks → valid:true" "$output"
else
  fail "valid tasks exit 0" "$output"
fi

if echo '[{"title":"T"}]' | $INTAKE_UTIL validate --type tasks >/dev/null 2>&1; then
  fail "invalid tasks should exit non-zero" "exited 0"
else
  pass "invalid tasks (missing id) → exit 1"
fi

if echo 'not json' | $INTAKE_UTIL validate --type tasks >/dev/null 2>&1; then
  fail "invalid json should exit non-zero" "exited 0"
else
  pass "invalid JSON → exit 1"
fi

# =========================================================================
# validate --type complexity
# =========================================================================
echo ""
echo "=== validate --type complexity ==="

if output=$(echo '{"overall_complexity":"high","complexity_scores":{}}' | $INTAKE_UTIL validate --type complexity 2>&1); then
  echo "$output" | jvalid && pass "valid complexity → valid:true" || fail "valid complexity" "$output"
else
  fail "valid complexity exit 0" "$output"
fi

if echo '{"something":"else"}' | $INTAKE_UTIL validate --type complexity >/dev/null 2>&1; then
  fail "missing complexity field should exit non-zero" "exited 0"
else
  pass "missing complexity field → exit 1"
fi

# =========================================================================
# validate --type tally
# =========================================================================
echo ""
echo "=== validate --type tally ==="

if output=$(echo '{"verdict":"approve","vote_breakdown":{"approve":3,"revise":1,"reject":1}}' | $INTAKE_UTIL validate --type tally 2>&1); then
  echo "$output" | jvalid && pass "valid tally → valid:true" || fail "valid tally" "$output"
else
  fail "valid tally exit 0" "$output"
fi

if echo '{"vote_breakdown":{}}' | $INTAKE_UTIL validate --type tally >/dev/null 2>&1; then
  fail "tally missing verdict should exit non-zero" "exited 0"
else
  pass "tally missing verdict → exit 1"
fi

# =========================================================================
# validate --type debate-turn
# =========================================================================
echo ""
echo "=== validate --type debate-turn ==="

if output=$(cat "$FIXTURES_DIR/debate-turn-with-dps.md" | $INTAKE_UTIL validate --type debate-turn 2>&1); then
  echo "$output" | jvalid && pass "valid debate turn → valid:true" || fail "valid debate turn" "$output"
else
  fail "valid debate turn exit 0" "$output"
fi

if echo '' | $INTAKE_UTIL validate --type debate-turn >/dev/null 2>&1; then
  fail "empty debate turn should exit non-zero" "exited 0"
else
  pass "empty debate turn → exit 1"
fi

# =========================================================================
# validate --type decision-points
# =========================================================================
echo ""
echo "=== validate --type decision-points ==="

if output=$(echo '[{"id":"dp-1","question":"Which DB?"},{"id":"dp-2","question":"Auth?"}]' | $INTAKE_UTIL validate --type decision-points 2>&1); then
  echo "$output" | jvalid && pass "valid decision-points → valid:true" || fail "valid DPs" "$output"
else
  fail "valid decision-points exit 0" "$output"
fi

if echo '[{"question":"no id"}]' | $INTAKE_UTIL validate --type decision-points >/dev/null 2>&1; then
  fail "DP missing id should exit non-zero" "exited 0"
else
  pass "DP missing id → exit 1"
fi

# =========================================================================
# validate --type decision-tally
# =========================================================================
echo ""
echo "=== validate --type decision-tally ==="

if output=$(echo '{"winning_option":"jwt","consensus_strength":0.6,"tally":{"jwt":3,"oauth":2}}' | $INTAKE_UTIL validate --type decision-tally 2>&1); then
  echo "$output" | jvalid && pass "valid decision-tally → valid:true" || fail "valid decision-tally" "$output"
else
  fail "valid decision-tally exit 0" "$output"
fi

if echo '{"tally":{}}' | $INTAKE_UTIL validate --type decision-tally >/dev/null 2>&1; then
  fail "decision-tally missing winner should exit non-zero" "exited 0"
else
  pass "decision-tally missing winner → exit 1"
fi

# =========================================================================
# validate --type deliberation-result
# =========================================================================
echo ""
echo "=== validate --type deliberation-result ==="

if output=$(echo '{"design_brief":"# Architecture","decision_points":[{"id":"dp-1","question":"DB?"}]}' | $INTAKE_UTIL validate --type deliberation-result 2>&1); then
  echo "$output" | jvalid && pass "valid delib result → valid:true" || fail "valid delib result" "$output"
else
  fail "valid deliberation-result exit 0" "$output"
fi

if echo '{"something":"else"}' | $INTAKE_UTIL validate --type deliberation-result >/dev/null 2>&1; then
  fail "delib result missing fields should exit non-zero" "exited 0"
else
  pass "delib result missing fields → exit 1"
fi

# =========================================================================
# validate --type expanded-tasks
# =========================================================================
echo ""
echo "=== validate --type expanded-tasks ==="

if output=$(echo '[{"id":1,"subtasks":[{"id":1,"title":"sub","description":"desc","dependencies":[]}]},{"id":2,"subtasks":[{"id":1,"title":"sub2","description":"desc2","dependencies":[]}]}]' | $INTAKE_UTIL validate --type expanded-tasks 2>&1); then
  echo "$output" | jvalid && pass "valid expanded-tasks → valid:true" || fail "valid expanded-tasks" "$output"
else
  fail "valid expanded-tasks exit 0" "$output"
fi

if echo '[{"subtasks":[]}]' | $INTAKE_UTIL validate --type expanded-tasks --strict >/dev/null 2>&1; then
  fail "expanded-tasks missing id should exit non-zero" "exited 0"
else
  pass "expanded-tasks missing id (strict) → exit 1"
fi

# =========================================================================
# validate --type scaffolds
# =========================================================================
echo ""
echo "=== validate --type scaffolds ==="

if output=$(echo '{"scaffolds":[{"task_id":1},{"task_id":2}]}' | $INTAKE_UTIL validate --type scaffolds 2>&1); then
  echo "$output" | jvalid && pass "valid scaffolds → valid:true" || fail "valid scaffolds" "$output"
else
  fail "valid scaffolds exit 0" "$output"
fi

# =========================================================================
# parse-decision-points
# =========================================================================
echo ""
echo "=== parse-decision-points ==="

DP_JSON=$(python3 -c "import json; print(json.dumps({'content': open('$FIXTURES_DIR/debate-turn-with-dps.md').read(), 'speaker': 'optimist'}))")
if output=$(echo "$DP_JSON" | $INTAKE_UTIL parse-decision-points 2>&1); then
  count=$(echo "$output" | jlen 2>/dev/null)
  [ "$count" = "2" ] && pass "extracts 2 DPs from debate turn" || fail "expected 2 DPs, got $count" ""

  dp1_id=$(echo "$output" | jidx 0 id 2>/dev/null)
  [ "$dp1_id" = "dp-1" ] && pass "first DP id = dp-1" || fail "expected dp-1, got $dp1_id" ""

  dp1_cat=$(echo "$output" | jidx 0 category 2>/dev/null)
  [ "$dp1_cat" = "architecture" ] && pass "first DP category = architecture" || fail "expected architecture, got $dp1_cat" ""
else
  fail "parse-decision-points exit 0" "$output"
fi

NO_DP_JSON=$(python3 -c "import json; print(json.dumps({'content': open('$FIXTURES_DIR/debate-turn-no-dps.md').read(), 'speaker': 'pessimist'}))")
if output=$(echo "$NO_DP_JSON" | $INTAKE_UTIL parse-decision-points 2>&1); then
  count=$(echo "$output" | jlen 2>/dev/null)
  [ "$count" = "0" ] && pass "returns empty array for no DPs" || fail "expected 0 DPs, got $count" ""
else
  fail "parse-decision-points (no DPs) exit 0" "$output"
fi

# =========================================================================
# tally-decision-votes (majority)
# =========================================================================
echo ""
echo "=== tally-decision-votes ==="

if output=$(cat "$FIXTURES_DIR/decision-votes-majority.json" | $INTAKE_UTIL tally-decision-votes 2>&1); then
  winner=$(echo "$output" | jf winning_option 2>/dev/null)
  [ "$winner" = "jwt" ] && pass "majority winner = jwt" || fail "expected jwt, got $winner" ""

  consensus=$(echo "$output" | jf consensus_strength 2>/dev/null)
  [ "$consensus" = "0.6" ] && pass "consensus_strength = 0.6" || fail "expected 0.6, got $consensus" ""

  escalated=$(echo "$output" | jf escalated 2>/dev/null)
  [ "$escalated" = "false" ] && pass "not escalated" || fail "expected false, got $escalated" ""
else
  fail "tally-decision-votes (majority) exit 0" "$output"
fi

# =========================================================================
# tally-decision-votes (tie)
# =========================================================================
echo ""
echo "=== tally-decision-votes (tie) ==="

if output=$(cat "$FIXTURES_DIR/decision-votes-tie.json" | $INTAKE_UTIL tally-decision-votes 2>&1); then
  winner=$(echo "$output" | jf winning_option 2>/dev/null)
  [ "$winner" = "None" ] && pass "tie → winning_option = null" || fail "expected None, got $winner" ""

  escalated=$(echo "$output" | jf escalated 2>/dev/null)
  [ "$escalated" = "true" ] && pass "tie → escalated = true" || fail "expected true, got $escalated" ""
else
  fail "tally-decision-votes (tie) exit 0" "$output"
fi

# =========================================================================
# classify-output
# =========================================================================
echo ""
echo "=== classify-output ==="

if output=$(cat "$FIXTURES_DIR/claude-cli-output.txt" | $INTAKE_UTIL classify-output --cli claude 2>&1); then
  type=$(echo "$output" | jf type 2>/dev/null)
  [ "$type" = "action" ] && pass "claude CLI output with Tool: → action" || fail "expected action, got $type" ""
else
  fail "classify-output (claude) exit 0" "$output"
fi

if output=$(cat "$FIXTURES_DIR/openclaw-json-output.json" | $INTAKE_UTIL classify-output --cli openclaw 2>&1); then
  type=$(echo "$output" | jf type 2>/dev/null)
  [ "$type" = "action" ] && pass "openclaw JSON with tool_use → action" || fail "expected action, got $type" ""

  action=$(echo "$output" | jf action 2>/dev/null)
  [ "$action" = "write_file" ] && pass "openclaw action = write_file" || fail "expected write_file, got $action" ""
else
  fail "classify-output (openclaw) exit 0" "$output"
fi

if output=$(echo "Just a plain response with no tools." | $INTAKE_UTIL classify-output --cli claude 2>&1); then
  type=$(echo "$output" | jf type 2>/dev/null)
  [ "$type" = "response" ] && pass "plain text (final) → response" || fail "expected response, got $type" ""
else
  fail "classify-output (plain) exit 0" "$output"
fi

if output=$(echo "Thinking about the implementation..." | $INTAKE_UTIL classify-output --cli claude --intermediate 2>&1); then
  type=$(echo "$output" | jf type 2>/dev/null)
  [ "$type" = "thought" ] && pass "plain text (intermediate) → thought" || fail "expected thought, got $type" ""

  ephemeral=$(echo "$output" | jf ephemeral 2>/dev/null)
  [ "$ephemeral" = "true" ] && pass "intermediate → ephemeral:true" || fail "expected ephemeral:true" ""
else
  fail "classify-output (intermediate) exit 0" "$output"
fi

# =========================================================================
# generate-workflows
# =========================================================================
echo ""
echo "=== generate-workflows ==="

GEN_TASKS='[{"id":1,"title":"Setup project","description":"Init","dependencies":[],"subtasks":[{"id":1,"title":"Init repo","description":"Create repo","dependencies":[]}],"agent":"bolt","stack":"DevOps"}]'
GEN_OUT_FILE=$(mktemp)
if echo "$GEN_TASKS" | $INTAKE_UTIL generate-workflows > "$GEN_OUT_FILE" 2>&1; then
  if python3 -c "
import json
with open('$GEN_OUT_FILE') as f:
    d = json.load(f)
wfs = d.get('task_workflows', d) if isinstance(d, dict) else d
assert len(wfs) > 0
assert 'workflow_yaml' in wfs[0]
print('OK')
" 2>/dev/null | grep -q OK; then
    pass "generates workflow_yaml"
  else
    fail "expected workflow_yaml in output" ""
  fi
else
  fail "generate-workflows exit 0" ""
fi
rm -f "$GEN_OUT_FILE"

# =========================================================================
# validate (legacy types with --task-ids)
# =========================================================================
echo ""
echo "=== validate (legacy types with --task-ids) ==="

DOCS='[{"task_id":1,"task_md":"# Task 1","decisions_md":"## Decisions","acceptance_md":"## AC"}]'
if output=$(echo "$DOCS" | $INTAKE_UTIL validate --type docs --task-ids '[1]' 2>&1); then
  echo "$output" | jvalid && pass "valid docs → valid:true" || fail "valid docs" "$output"
else
  fail "valid docs exit 0" "$output"
fi

PROMPTS='[{"task_id":1,"prompt_md":"# Prompt","prompt_xml":"<prompt/>"}]'
if output=$(echo "$PROMPTS" | $INTAKE_UTIL validate --type prompts --task-ids '[1]' 2>&1); then
  echo "$output" | jvalid && pass "valid prompts → valid:true" || fail "valid prompts" "$output"
else
  fail "valid prompts exit 0" "$output"
fi

WORKFLOWS_INPUT='[{"task_id":1,"workflow_yaml":"name: task-1\nsteps:\n  - name: setup\n    command: echo\n  - name: create-pr\n    command: gh"}]'
if output=$(echo "$WORKFLOWS_INPUT" | $INTAKE_UTIL validate --type workflows --task-ids '[1]' 2>&1); then
  echo "$output" | jvalid && pass "valid workflows → valid:true" || fail "valid workflows" "$output"
else
  fail "valid workflows exit 0" "$output"
fi

# =========================================================================
# Summary
# =========================================================================
echo ""
echo "========================================"
echo "Results: $PASS passed, $FAIL failed"
echo "========================================"

if [ "$FAIL" -gt 0 ]; then
  echo ""
  echo "Failures:"
  echo -e "$ERRORS"
  exit 1
fi
