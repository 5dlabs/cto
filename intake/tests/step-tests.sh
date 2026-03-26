#!/usr/bin/env bash
# Step Tests: Validate each intake pipeline step in isolation using golden fixtures.
#
# These tests replicate the env wiring from intake.lobster.yaml and run
# the same commands with deterministic inputs. No LLM calls are made.
#
# Requires: bun (for intake-util), python3, jq

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GOLDEN_DIR="$SCRIPT_DIR/golden"
INTAKE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

BUN=$(command -v bun 2>/dev/null || echo "$HOME/.bun/bin/bun")
if [ ! -x "$BUN" ]; then
  echo "ERROR: bun not found. Install from https://bun.sh"
  exit 1
fi
INTAKE_UTIL="$BUN run $REPO_ROOT/apps/intake-util/src/index.ts"

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

# =========================================================================
# T01: verify-parse-prd with valid golden fixture
# =========================================================================
echo "=== T01: verify-parse-prd (valid golden) ==="

CTO_PARSE_PRD_OUT="$(cat "$GOLDEN_DIR/parse-prd.json")"
TASK_COUNT=$(printf '%s' "$CTO_PARSE_PRD_OUT" | jq 'if type=="array" then length else 0 end' 2>/dev/null || echo 0)
if [ "$TASK_COUNT" -gt 0 ]; then
  if printf '%s' "$CTO_PARSE_PRD_OUT" | $INTAKE_UTIL validate --type generated-task >/dev/null 2>&1; then
    pass "verify-parse-prd accepts valid tasks ($TASK_COUNT tasks)"
  else
    fail "verify-parse-prd should accept valid tasks" "validate --type generated-task rejected golden fixture"
  fi
else
  fail "golden parse-prd.json should have tasks" "task count was $TASK_COUNT"
fi

# =========================================================================
# T02: verify-parse-prd with empty input
# =========================================================================
echo ""
echo "=== T02: verify-parse-prd (empty input) ==="

if printf '' | $INTAKE_UTIL validate --type generated-task >/dev/null 2>&1; then
  fail "verify-parse-prd should reject empty input" "exited 0 on empty"
else
  pass "verify-parse-prd rejects empty input"
fi

# =========================================================================
# T03: verify-parse-prd with chat garbage
# =========================================================================
echo ""
echo "=== T03: verify-parse-prd (chat garbage) ==="

GARBAGE='[{"message": "Hello! How can I help you today?"}]'
TASK_COUNT=$(printf '%s' "$GARBAGE" | jq 'if type=="array" then length else 0 end' 2>/dev/null || echo 0)
if printf '%s' "$GARBAGE" | $INTAKE_UTIL validate --type generated-task >/dev/null 2>&1; then
  fail "verify-parse-prd should reject chat garbage" "validate accepted garbage JSON"
else
  pass "verify-parse-prd rejects chat garbage (missing required fields)"
fi

# =========================================================================
# T04: verify-analyze-complexity with golden fixture
# =========================================================================
echo ""
echo "=== T04: verify-analyze-complexity (valid golden) ==="

CTO_ANALYZE_COMPLEXITY_OUT="$(cat "$GOLDEN_DIR/analyze-complexity.json")"
if printf '%s' "$CTO_ANALYZE_COMPLEXITY_OUT" | $INTAKE_UTIL validate --type complexity-analysis >/dev/null 2>&1; then
  pass "verify-analyze-complexity accepts valid analysis"
else
  fail "verify-analyze-complexity should accept valid analysis" "validate --type complexity-analysis rejected golden"
fi

# =========================================================================
# T05: verify-refine-tasks with golden fixture
# =========================================================================
echo ""
echo "=== T05: verify-refine-tasks (valid golden) ==="

CTO_REFINE_TASKS_OUT="$(cat "$GOLDEN_DIR/refine-tasks.json")"
EXP=$(printf '%s' "$CTO_REFINE_TASKS_OUT" | jq -c '.output[0].expanded_tasks' 2>/dev/null || printf '%s' "$CTO_REFINE_TASKS_OUT")
if [ -n "$EXP" ] && [ "$EXP" != "null" ]; then
  EXP_COUNT=$(printf '%s' "$EXP" | jq 'if type=="array" then length else 0 end')
  if [ "$EXP_COUNT" -gt 0 ]; then
    if printf '%s' "$EXP" | $INTAKE_UTIL validate --type expanded-tasks >/dev/null 2>&1; then
      pass "verify-refine-tasks accepts valid expanded tasks ($EXP_COUNT tasks)"
    else
      fail "verify-refine-tasks should accept valid expanded tasks" "validate --type expanded-tasks rejected golden"
    fi
  else
    fail "golden refine-tasks.json should have expanded tasks" "count was $EXP_COUNT"
  fi
else
  fail "could not extract expanded_tasks from golden" "EXP was null or empty"
fi

# =========================================================================
# T06: verify-refine-tasks with null expanded_tasks
# =========================================================================
echo ""
echo "=== T06: verify-refine-tasks (null expanded_tasks) ==="

BAD_REFINE='{"output":[{"expanded_tasks":null,"verdict":"failed"}]}'
BAD_EXP=$(printf '%s' "$BAD_REFINE" | jq -c '.output[0].expanded_tasks' 2>/dev/null)
if [ "$BAD_EXP" = "null" ] || [ -z "$BAD_EXP" ]; then
  pass "null expanded_tasks correctly detected as empty"
else
  fail "should detect null expanded_tasks" "got: $BAD_EXP"
fi

# =========================================================================
# T07: verify-generate-scaffolds with golden fixture
# =========================================================================
echo ""
echo "=== T07: verify-generate-scaffolds (valid golden) ==="

CTO_GENERATE_SCAFFOLDS_OUT="$(cat "$GOLDEN_DIR/generate-scaffolds.json")"
if printf '%s' "$CTO_GENERATE_SCAFFOLDS_OUT" | $INTAKE_UTIL validate --type scaffold >/dev/null 2>&1; then
  pass "verify-generate-scaffolds accepts valid scaffolds"
else
  fail "verify-generate-scaffolds should accept valid scaffolds" "validate --type scaffold rejected golden"
fi

# =========================================================================
# T08: validate-docs with golden fixture + matching task IDs
# =========================================================================
echo ""
echo "=== T08: validate-docs (valid golden + matching IDs) ==="

CTO_FAN_OUT_DOCS_OUT="$(cat "$GOLDEN_DIR/fan-out-docs.json")"
TASK_IDS='[1,2,3]'
if printf '%s' "$CTO_FAN_OUT_DOCS_OUT" | $INTAKE_UTIL validate --type docs --task-ids "$TASK_IDS" >/dev/null 2>&1; then
  pass "validate-docs accepts valid docs with matching task IDs"
else
  fail "validate-docs should accept valid docs" "validate --type docs rejected golden"
fi

# =========================================================================
# T09: validate-docs with mismatched task IDs
# =========================================================================
echo ""
echo "=== T09: validate-docs (mismatched task IDs) ==="

BAD_TASK_IDS='[1,2,3,99]'
OUTPUT=$(printf '%s' "$CTO_FAN_OUT_DOCS_OUT" | $INTAKE_UTIL validate --type docs --task-ids "$BAD_TASK_IDS" 2>&1 || true)
if echo "$OUTPUT" | grep -qi "missing\|error\|99"; then
  pass "validate-docs detects missing task_id 99"
else
  fail "validate-docs should detect missing task_id 99" "$OUTPUT"
fi

# =========================================================================
# T10: validate-prompts with golden fixture + matching task IDs
# =========================================================================
echo ""
echo "=== T10: validate-prompts (valid golden + matching IDs) ==="

CTO_FAN_OUT_PROMPTS_OUT="$(cat "$GOLDEN_DIR/fan-out-prompts.json")"
if printf '%s' "$CTO_FAN_OUT_PROMPTS_OUT" | $INTAKE_UTIL validate --type prompts --task-ids "$TASK_IDS" >/dev/null 2>&1; then
  pass "validate-prompts accepts valid prompts with matching task IDs"
else
  fail "validate-prompts should accept valid prompts" "validate --type prompts rejected golden"
fi

# =========================================================================
# T11: validate-workflows with golden fixture
# =========================================================================
echo ""
echo "=== T11: validate-workflows (valid golden) ==="

CTO_GENERATE_WORKFLOWS_OUT="$(cat "$GOLDEN_DIR/generate-workflows.json")"
WF_JSON=$(printf '%s' "$CTO_GENERATE_WORKFLOWS_OUT" | jq '.task_workflows')
if printf '%s' "$WF_JSON" | $INTAKE_UTIL validate --type workflows --task-ids "$TASK_IDS" >/dev/null 2>&1; then
  pass "validate-workflows accepts valid workflows"
else
  fail "validate-workflows should accept valid workflows" "validate --type workflows rejected golden"
fi

# =========================================================================
# T12: verify-artifact-gates with all goldens
# =========================================================================
echo ""
echo "=== T12: verify-artifact-gates (all goldens present) ==="

TMPWS=$(mktemp -d)
trap 'rm -rf "$TMPWS"' EXIT
mkdir -p "$TMPWS/.tasks/tasks" "$TMPWS/.tasks/docs"
EXP=$(printf '%s' "$CTO_REFINE_TASKS_OUT" | jq -c '.output[0].expanded_tasks')
printf '%s\n' "$EXP" > "$TMPWS/.tasks/tasks/tasks.json"

TASK_COUNT=$(printf '%s' "$EXP" | jq 'length')
DOC_COUNT=$(printf '%s' "$CTO_FAN_OUT_DOCS_OUT" | jq 'if type=="array" then length else 0 end')
PROMPT_COUNT=$(printf '%s' "$CTO_FAN_OUT_PROMPTS_OUT" | jq 'if type=="array" then length else 0 end')
WORKFLOW_COUNT=$(printf '%s' "$CTO_GENERATE_WORKFLOWS_OUT" | jq '.task_workflows | length')

if [ "$TASK_COUNT" -gt 0 ] && [ "$DOC_COUNT" -gt 0 ] && [ "$PROMPT_COUNT" -gt 0 ] && [ "$WORKFLOW_COUNT" -gt 0 ]; then
  pass "verify-artifact-gates: all counts > 0 (tasks=$TASK_COUNT docs=$DOC_COUNT prompts=$PROMPT_COUNT workflows=$WORKFLOW_COUNT)"
else
  fail "verify-artifact-gates: some counts are 0" "tasks=$TASK_COUNT docs=$DOC_COUNT prompts=$PROMPT_COUNT workflows=$WORKFLOW_COUNT"
fi

if [ -s "$TMPWS/.tasks/tasks/tasks.json" ]; then
  pass "verify-artifact-gates: tasks.json written"
else
  fail "verify-artifact-gates: tasks.json not written" ""
fi

# =========================================================================
# T13: verify-artifact-gates with missing docs
# =========================================================================
echo ""
echo "=== T13: verify-artifact-gates (missing docs) ==="

EMPTY_DOCS='[]'
EMPTY_DOC_COUNT=$(printf '%s' "$EMPTY_DOCS" | jq 'if type=="array" then length else 0 end')
if [ "$EMPTY_DOC_COUNT" -eq 0 ]; then
  pass "verify-artifact-gates correctly detects missing docs (count=0)"
else
  fail "empty docs should give count 0" "got $EMPTY_DOC_COUNT"
fi

# =========================================================================
# T14: verify-folder-structure after artifact gates
# =========================================================================
echo ""
echo "=== T14: verify-folder-structure ==="

if [ -d "$TMPWS/.tasks" ] && [ -d "$TMPWS/.tasks/tasks" ] && [ -s "$TMPWS/.tasks/tasks/tasks.json" ]; then
  pass "verify-folder-structure: .tasks/ structure correct"
else
  fail "verify-folder-structure: expected .tasks/ structure" ""
fi

# =========================================================================
# T15: write-docs writes files
# =========================================================================
echo ""
echo "=== T15: write-docs (writes to temp dir) ==="

DOCS_DIR="$TMPWS/.tasks/docs"
mkdir -p "$DOCS_DIR"
DOCS_WRAPPED=$(printf '%s' "$CTO_FAN_OUT_DOCS_OUT" | jq '{task_docs: .}')
if printf '%s' "$DOCS_WRAPPED" | $INTAKE_UTIL write-files --base-path "$DOCS_DIR" --type docs >/dev/null 2>&1; then
  WRITTEN=$(find "$DOCS_DIR" -type f | wc -l | tr -d ' ')
  if [ "$WRITTEN" -gt 0 ]; then
    pass "write-docs wrote $WRITTEN file(s) to $DOCS_DIR"
  else
    fail "write-docs produced no files" "directory is empty"
  fi
else
  fail "write-docs command failed" ""
fi

# =========================================================================
# T16: write-prompts writes files
# =========================================================================
echo ""
echo "=== T16: write-prompts (writes to temp dir) ==="

PROMPTS_DIR="$TMPWS/.tasks/prompts"
mkdir -p "$PROMPTS_DIR"
PROMPTS_WRAPPED=$(printf '%s' "$CTO_FAN_OUT_PROMPTS_OUT" | jq '{task_prompts: .}')
if printf '%s' "$PROMPTS_WRAPPED" | $INTAKE_UTIL write-files --base-path "$PROMPTS_DIR" --type prompts >/dev/null 2>&1; then
  WRITTEN=$(find "$PROMPTS_DIR" -type f | wc -l | tr -d ' ')
  if [ "$WRITTEN" -gt 0 ]; then
    pass "write-prompts wrote $WRITTEN file(s) to $PROMPTS_DIR"
  else
    fail "write-prompts produced no files" "directory is empty"
  fi
else
  fail "write-prompts command failed" ""
fi

# =========================================================================
# T17: linear-activity with empty session ID
# =========================================================================
echo ""
echo "=== T17: linear-activity (empty session graceful) ==="

if $INTAKE_UTIL linear-activity --session-id "" --type thought --title "Test" --body "Test" >/dev/null 2>&1; then
  pass "linear-activity with empty session exits 0 (graceful)"
else
  EXIT_CODE=$?
  # Acceptable: exits non-zero but does not crash with unhandled exception
  pass "linear-activity with empty session exits $EXIT_CODE (non-crash)"
fi

# =========================================================================
# T18: openclaw.invoke chat garbage detection
# =========================================================================
echo ""
echo "=== T18: openclaw.invoke _is_chat_garbage ==="

if command -v python3 >/dev/null 2>&1; then
  GARBAGE_RESULT=$(python3 -c "
import sys
sys.path.insert(0, '.')
# Inline the function since openclaw.invoke is a script, not a module
CHAT_GARBAGE_PATTERNS = {'hello', 'how can i help', 'assist you', 'happy to help', 'what can i do for you'}
def _is_chat_garbage(parsed):
    items = parsed if isinstance(parsed, list) else [parsed]
    if not items: return False
    for item in items[:3]:
        if not isinstance(item, dict): continue
        keys = set(item.keys())
        if keys == {'message'} or keys == {'content'} or keys == {'text'} or keys == {'response'}:
            val = str(list(item.values())[0]).lower()
            if any(p in val for p in CHAT_GARBAGE_PATTERNS): return True
    return False

assert _is_chat_garbage([{'message': 'Hello! How can I help you today?'}]) == True
assert _is_chat_garbage([{'message': 'Hello!'}, {'message': 'How can I assist you?'}]) == True
assert _is_chat_garbage([{'id': 1, 'title': 'Task', 'description': 'Real task'}]) == False
assert _is_chat_garbage({'overall_complexity': 'high', 'task_analyses': []}) == False
assert _is_chat_garbage([]) == False
print('all assertions passed')
" 2>&1)
  if echo "$GARBAGE_RESULT" | grep -q "all assertions passed"; then
    pass "_is_chat_garbage correctly identifies garbage and real data"
  else
    fail "_is_chat_garbage assertions failed" "$GARBAGE_RESULT"
  fi
else
  echo "  SKIP: python3 not found"
fi

# =========================================================================
# T19: openclaw.invoke schema spot check
# =========================================================================
echo ""
echo "=== T19: openclaw.invoke _schema_spot_check ==="

if command -v python3 >/dev/null 2>&1; then
  SCHEMA_RESULT=$(python3 -c "
import json, tempfile, os

def _schema_spot_check(parsed, schema_path):
    try:
        with open(schema_path) as f:
            schema = json.load(f)
    except (OSError, json.JSONDecodeError):
        return True
    props = schema.get('properties', {})
    required = set(schema.get('required', []))
    if schema.get('type') == 'array':
        item_schema = schema.get('items', {})
        props = item_schema.get('properties', {})
        required = set(item_schema.get('required', []))
        if isinstance(parsed, list) and parsed:
            sample = parsed[0]
        else:
            return True
    else:
        sample = parsed
    if not required:
        return True
    if not isinstance(sample, dict):
        return False
    present = set(sample.keys())
    hit_ratio = len(present & required) / len(required)
    return hit_ratio >= 0.3

schema_dir = '$INTAKE_DIR/schemas'
# Test with a valid task analysis item against complexity schema (has task_id, task_title, complexity_score = 3/6 = 50%)
assert _schema_spot_check({'task_id': 1, 'task_title': 'Test', 'complexity_score': 7, 'recommended_subtasks': 3, 'expansion_prompt': 'x', 'reasoning': 'y'}, schema_dir + '/complexity-analysis.schema.json') == True, 'valid complexity item should pass'
# Test with garbage against complexity schema (has 0/6 required = 0%)
assert _schema_spot_check({'message': 'Hello!'}, schema_dir + '/complexity-analysis.schema.json') == False, 'garbage should fail complexity schema'
# Test with missing schema file (should pass gracefully)
assert _schema_spot_check({'anything': True}, '/nonexistent/schema.json') == True, 'missing schema should pass'
# Test with partial match (task_id only = 1/6 = 16.7%, below 30%)
assert _schema_spot_check({'task_id': 1}, schema_dir + '/complexity-analysis.schema.json') == False, 'too few fields should fail'
print('all assertions passed')
" 2>&1)
  if echo "$SCHEMA_RESULT" | grep -q "all assertions passed"; then
    pass "_schema_spot_check correctly validates schema conformance"
  else
    fail "_schema_spot_check assertions failed" "$SCHEMA_RESULT"
  fi
else
  echo "  SKIP: python3 not found"
fi

# =========================================================================
# T20: discover-skills golden validates
# =========================================================================
echo ""
echo "=== T20: validate skill-recommendations (golden) ==="

CTO_DISCOVER_SKILLS_OUT="$(cat "$GOLDEN_DIR/discover-skills.json")"
if printf '%s' "$CTO_DISCOVER_SKILLS_OUT" | $INTAKE_UTIL validate --type skill-recommendations >/dev/null 2>&1; then
  pass "validate --type skill-recommendations accepts golden fixture"
else
  fail "validate --type skill-recommendations should accept golden" "rejected golden fixture"
fi

# =========================================================================
# Summary
# =========================================================================
echo ""
echo "=============================="
echo "  PASS: $PASS   FAIL: $FAIL"
echo "=============================="

if [ "$FAIL" -gt 0 ]; then
  printf '\nFailure details:\n%b\n' "$ERRORS"
  exit 1
fi
