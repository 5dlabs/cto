#!/usr/bin/env bash
# Integration Test: Validate the consolidated Lobster intake pipeline.
#
# This test verifies:
# 1. All workflow files are valid YAML and reference existing prompts/schemas
# 2. intake-util generate-docs and generate-prompts produce expected output
# 3. intake-util tally produces a valid verdict from test ballots
# 4. The pipeline.lobster.yaml correctly wires deliberation and intake workflows
#
# NOTE: This does NOT test live AI calls (openclaw.invoke) — those require
# a running OpenClaw cluster. It validates the pipeline structure and the
# deterministic (non-AI) components.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INTAKE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
FIXTURES_DIR="$SCRIPT_DIR/fixtures"

PASS=0
FAIL=0
ERRORS=""

pass() {
  echo "  PASS: $1"
  PASS=$((PASS + 1))
}

fail() {
  echo "  FAIL: $1"
  ERRORS="${ERRORS}\n--- $1 ---\n$2\n"
  FAIL=$((FAIL + 1))
}

# =========================================================================
# Test 1: Workflow YAML validation
# =========================================================================
echo "=== Test 1: Workflow YAML Validation ==="

for workflow in pipeline.lobster.yaml intake.lobster.yaml deliberation.lobster.yaml voting.lobster.yaml codebase-analysis.lobster.yaml task-refinement.lobster.yaml; do
  filepath="$INTAKE_DIR/workflows/$workflow"
  if [ ! -f "$filepath" ]; then
    fail "$workflow exists" "File not found: $filepath"
    continue
  fi

  # Validate YAML syntax (requires python3 or yq)
  if command -v python3 >/dev/null 2>&1; then
    if python3 -c "import yaml; yaml.safe_load(open('$filepath'))" 2>/dev/null; then
      pass "$workflow is valid YAML"
    else
      fail "$workflow is valid YAML" "YAML parse error"
    fi
  else
    pass "$workflow exists (YAML validation skipped — no python3)"
  fi
done

# =========================================================================
# Test 2: Prompt templates exist
# =========================================================================
echo ""
echo "=== Test 2: Prompt Templates ==="

for prompt in parse-prd-system.md parse-prd-user.md analyze-complexity-system.md \
  analyze-complexity-user.md expand-task-system.md expand-task-user.md \
  vote-system.md vote-user.md optimist-soul.md pessimist-soul.md \
  deliberation-conductor-system.md compile-design-brief-system.md \
  codebase-analysis-system.md; do
  filepath="$INTAKE_DIR/prompts/$prompt"
  if [ -f "$filepath" ]; then
    pass "$prompt exists"
  else
    fail "$prompt exists" "File not found: $filepath"
  fi
done

# =========================================================================
# Test 3: JSON schemas exist and are valid
# =========================================================================
echo ""
echo "=== Test 3: JSON Schemas ==="

for schema in generated-task.schema.json generated-subtask.schema.json \
  complexity-analysis.schema.json vote-ballot.schema.json \
  decision-point.schema.json decision-vote.schema.json \
  deliberation-result.schema.json; do
  filepath="$INTAKE_DIR/schemas/$schema"
  if [ ! -f "$filepath" ]; then
    fail "$schema exists" "File not found: $filepath"
    continue
  fi

  if python3 -c "import json; json.load(open('$filepath'))" 2>/dev/null; then
    pass "$schema is valid JSON"
  else
    fail "$schema is valid JSON" "JSON parse error"
  fi
done

# =========================================================================
# Test 4: intake-util generate-docs
# =========================================================================
echo ""
echo "=== Test 4: intake-util generate-docs ==="

UTIL_BIN="$INTAKE_DIR/util/src/index.ts"
if [ ! -f "$UTIL_BIN" ]; then
  echo "  SKIP: intake-util not found at $UTIL_BIN"
else
  for size in small medium large; do
    fixture="$FIXTURES_DIR/tasks-${size}.json"
    if [ ! -f "$fixture" ]; then
      echo "  SKIP: $fixture not found"
      continue
    fi

    output_dir="/tmp/pipeline-test-docs-${size}"
    rm -rf "$output_dir"

    if command -v bun >/dev/null 2>&1; then
      if bun run "$UTIL_BIN" generate-docs --task-json "$fixture" --base-path "$output_dir" > /dev/null 2>&1; then
        # Verify expected files exist
        task_dirs=$(find "$output_dir" -name "task.md" 2>/dev/null | wc -l | tr -d ' ')
        if [ "$task_dirs" -gt 0 ]; then
          pass "generate-docs ($size): $task_dirs task docs created"
        else
          fail "generate-docs ($size)" "No task.md files found in $output_dir"
        fi
      else
        fail "generate-docs ($size)" "intake-util exited with error"
      fi
    else
      echo "  SKIP: bun not available for intake-util tests"
      break
    fi

    rm -rf "$output_dir"
  done
fi

# =========================================================================
# Test 5: intake-util generate-prompts
# =========================================================================
echo ""
echo "=== Test 5: intake-util generate-prompts ==="

if [ -f "$UTIL_BIN" ] && command -v bun >/dev/null 2>&1; then
  for size in small medium large; do
    fixture="$FIXTURES_DIR/tasks-${size}.json"
    if [ ! -f "$fixture" ]; then
      echo "  SKIP: $fixture not found"
      continue
    fi

    output_dir="/tmp/pipeline-test-prompts-${size}"
    rm -rf "$output_dir"

    if bun run "$UTIL_BIN" generate-prompts --task-json "$fixture" --output-dir "$output_dir" --project-name "test-${size}" > /dev/null 2>&1; then
      prompt_files=$(find "$output_dir" -name "prompt.md" 2>/dev/null | wc -l | tr -d ' ')
      if [ "$prompt_files" -gt 0 ]; then
        pass "generate-prompts ($size): $prompt_files prompt files created"
      else
        fail "generate-prompts ($size)" "No prompt.md files found in $output_dir"
      fi
    else
      fail "generate-prompts ($size)" "intake-util exited with error"
    fi

    rm -rf "$output_dir"
  done
else
  echo "  SKIP: bun or intake-util not available"
fi

# =========================================================================
# Test 6: intake-util tally
# =========================================================================
echo ""
echo "=== Test 6: intake-util tally ==="

if [ -f "$UTIL_BIN" ] && command -v bun >/dev/null 2>&1; then
  # Create test ballots
  test_ballots='[
    {
      "voter_id": "voter-1",
      "scores": {"task_decomposition": 8, "dependency_ordering": 7, "decision_point_coverage": 6, "test_strategy_quality": 7, "agent_assignment": 8},
      "overall_score": 7.4,
      "verdict": "approve",
      "reasoning": "Good task decomposition with clear boundaries.",
      "suggestions": ["Consider splitting task 3"]
    },
    {
      "voter_id": "voter-2",
      "scores": {"task_decomposition": 7, "dependency_ordering": 8, "decision_point_coverage": 7, "test_strategy_quality": 6, "agent_assignment": 7},
      "overall_score": 7.1,
      "verdict": "approve",
      "reasoning": "Well-structured dependencies.",
      "suggestions": ["Add error case testing"]
    },
    {
      "voter_id": "voter-3",
      "scores": {"task_decomposition": 6, "dependency_ordering": 7, "decision_point_coverage": 5, "test_strategy_quality": 5, "agent_assignment": 6},
      "overall_score": 5.9,
      "verdict": "revise",
      "reasoning": "Test strategies need improvement.",
      "suggestions": ["Improve acceptance criteria specificity"]
    }
  ]'

  tally_result=$(echo "$test_ballots" | bun run "$UTIL_BIN" tally 2>/dev/null || true)

  if [ -n "$tally_result" ]; then
    verdict=$(echo "$tally_result" | python3 -c "import json,sys; print(json.load(sys.stdin).get('verdict',''))" 2>/dev/null || true)
    if [ -n "$verdict" ]; then
      pass "tally produces verdict: $verdict"
    else
      fail "tally output" "Could not extract verdict from: $tally_result"
    fi
  else
    fail "tally execution" "No output from intake-util tally"
  fi
else
  echo "  SKIP: bun or intake-util not available"
fi

# =========================================================================
# Test 7: Pipeline workflow structure
# =========================================================================
echo ""
echo "=== Test 7: Pipeline Workflow Structure ==="

pipeline="$INTAKE_DIR/workflows/pipeline.lobster.yaml"

# Check pipeline references deliberation and intake workflows
if grep -q "deliberation.lobster.yaml" "$pipeline" 2>/dev/null; then
  pass "pipeline references deliberation workflow"
else
  fail "pipeline references deliberation" "deliberation.lobster.yaml not found in pipeline"
fi

if grep -q "intake.lobster.yaml" "$pipeline" 2>/dev/null; then
  pass "pipeline references intake workflow"
else
  fail "pipeline references intake" "intake.lobster.yaml not found in pipeline"
fi

# Check pipeline has conditional deliberation
if grep -q "when:" "$pipeline" 2>/dev/null || grep -q "deliberate" "$pipeline" 2>/dev/null; then
  pass "pipeline has conditional deliberation"
else
  fail "pipeline conditional deliberation" "No 'when' or 'deliberate' condition found"
fi

# Check intake workflow references task-refinement sub-workflow
intake_wf="$INTAKE_DIR/workflows/intake.lobster.yaml"
if grep -q "task-refinement.lobster.yaml" "$intake_wf" 2>/dev/null; then
  pass "intake references task-refinement workflow"
else
  fail "intake references task-refinement" "task-refinement.lobster.yaml not found in intake workflow"
fi

# Check task-refinement references voting
refinement_wf="$INTAKE_DIR/workflows/task-refinement.lobster.yaml"
if grep -q "voting.lobster.yaml" "$refinement_wf" 2>/dev/null; then
  pass "task-refinement references voting workflow"
else
  fail "task-refinement references voting" "voting.lobster.yaml not found in task-refinement workflow"
fi

# Check that AAA debate files are removed
echo ""
echo "=== Test 8: AAA Debate Code Removed ==="

aaa_debate="$INTAKE_DIR/../apps/intake-agent/src/operations/generate-with-debate.ts"
aaa_templates="$INTAKE_DIR/../apps/intake-agent/src/prompts/debate-templates.ts"

if [ ! -f "$aaa_debate" ]; then
  pass "generate-with-debate.ts removed"
else
  fail "generate-with-debate.ts removed" "File still exists: $aaa_debate"
fi

if [ ! -f "$aaa_templates" ]; then
  pass "debate-templates.ts removed"
else
  fail "debate-templates.ts removed" "File still exists: $aaa_templates"
fi

# Check index.ts doesn't reference generate_with_debate
index_ts="$INTAKE_DIR/../apps/intake-agent/src/index.ts"
if [ -f "$index_ts" ]; then
  if grep -q "generate_with_debate\|generate-with-debate" "$index_ts" 2>/dev/null; then
    fail "index.ts cleaned up" "Still references generate_with_debate"
  else
    pass "index.ts no longer references AAA debate"
  fi
fi

# =========================================================================
# Test 9: Codebase Analysis Pipeline
# =========================================================================
echo ""
echo "=== Test 9: Codebase Analysis Pipeline ==="

pipeline="$INTAKE_DIR/workflows/pipeline.lobster.yaml"

# Check pipeline has include_codebase input
if grep -q "include_codebase" "$pipeline" 2>/dev/null; then
  pass "pipeline has include_codebase input"
else
  fail "pipeline include_codebase" "include_codebase not found in pipeline"
fi

# Check pipeline has repository_url input
if grep -q "repository_url" "$pipeline" 2>/dev/null; then
  pass "pipeline has repository_url input"
else
  fail "pipeline repository_url" "repository_url not found in pipeline"
fi

# Check pipeline references codebase-analysis workflow
if grep -q "codebase-analysis.lobster.yaml" "$pipeline" 2>/dev/null; then
  pass "pipeline references codebase-analysis workflow"
else
  fail "pipeline codebase-analysis" "codebase-analysis.lobster.yaml not found in pipeline"
fi

# Check codebase_context flows to deliberation
if grep -q "codebase_context" "$pipeline" 2>/dev/null; then
  pass "pipeline passes codebase_context downstream"
else
  fail "pipeline codebase_context" "codebase_context not found in pipeline"
fi

# Check intake workflow accepts codebase_context
intake_wf="$INTAKE_DIR/workflows/intake.lobster.yaml"
if grep -q "codebase_context" "$intake_wf" 2>/dev/null; then
  pass "intake workflow accepts codebase_context"
else
  fail "intake codebase_context" "codebase_context not found in intake workflow"
fi

# Check deliberation workflow accepts codebase_context
delib_wf="$INTAKE_DIR/workflows/deliberation.lobster.yaml"
if grep -q "codebase_context" "$delib_wf" 2>/dev/null; then
  pass "deliberation workflow accepts codebase_context"
else
  fail "deliberation codebase_context" "codebase_context not found in deliberation workflow"
fi

# Check parse-prd prompt has non-greenfield section
prd_prompt="$INTAKE_DIR/prompts/parse-prd-system.md"
if grep -q "Non-Greenfield\|non-greenfield\|codebase_context\|Existing Codebase" "$prd_prompt" 2>/dev/null; then
  pass "parse-prd prompt has non-greenfield section"
else
  fail "parse-prd non-greenfield" "No non-greenfield/codebase section found in parse-prd-system.md"
fi

# Check debate agents reference codebase context
opt_prompt="$INTAKE_DIR/prompts/optimist-soul.md"
pes_prompt="$INTAKE_DIR/prompts/pessimist-soul.md"
if grep -q "non-greenfield\|codebase\|existing" "$opt_prompt" 2>/dev/null; then
  pass "optimist prompt references existing codebase"
else
  fail "optimist codebase" "No codebase reference in optimist-soul.md"
fi

if grep -q "non-greenfield\|codebase\|existing" "$pes_prompt" 2>/dev/null; then
  pass "pessimist prompt references existing codebase"
else
  fail "pessimist codebase" "No codebase reference in pessimist-soul.md"
fi

# =========================================================================
# Test 10: depends_on chains form valid DAGs
# =========================================================================
echo ""
echo "=== Test 10: depends_on Chain Validation ==="

if command -v python3 >/dev/null 2>&1; then
  # Validate intake.lobster.yaml sequential chain
  python3 -c "
import yaml, sys

def check_depends_on(filepath, expected_deps):
    with open(filepath) as f:
        wf = yaml.safe_load(f)
    steps = {s['name']: s.get('depends_on', []) for s in wf.get('steps', [])}
    errors = []
    for step_name, expected in expected_deps.items():
        actual = steps.get(step_name, [])
        for dep in expected:
            if dep not in actual:
                errors.append(f'{step_name} missing depends_on: {dep} (has: {actual})')
    return errors

# intake.lobster.yaml chain
errors = check_depends_on('$INTAKE_DIR/workflows/intake.lobster.yaml', {
    'analyze-complexity': ['parse-prd'],
    'review-tasks': ['analyze-complexity'],
    'refine-tasks': ['review-tasks'],
    'generate-docs': ['refine-tasks'],
    'generate-prompts': ['generate-docs'],
    'commit-outputs': ['generate-prompts'],
    'create-pr': ['commit-outputs'],
})
if errors:
    for e in errors:
        print(f'CHAIN_ERROR: {e}')
    sys.exit(1)
else:
    print('OK: intake.lobster.yaml')

# voting.lobster.yaml — tally depends on all voters
errors = check_depends_on('$INTAKE_DIR/workflows/voting.lobster.yaml', {
    'tally': ['voter-1', 'voter-2', 'voter-3', 'voter-4', 'voter-5'],
})
if errors:
    for e in errors:
        print(f'CHAIN_ERROR: {e}')
    sys.exit(1)
else:
    print('OK: voting.lobster.yaml')

# deliberation.lobster.yaml chain
errors = check_depends_on('$INTAKE_DIR/workflows/deliberation.lobster.yaml', {
    'conduct-deliberation': ['research-prd'],
    'compile-brief': ['conduct-deliberation'],
    'save-brief': ['compile-brief'],
})
if errors:
    for e in errors:
        print(f'CHAIN_ERROR: {e}')
    sys.exit(1)
else:
    print('OK: deliberation.lobster.yaml')

# task-refinement.lobster.yaml chain
errors = check_depends_on('$INTAKE_DIR/workflows/task-refinement.lobster.yaml', {
    'vote-round-0': ['expand-round-0'],
    'check-round-0': ['vote-round-0'],
    'expand-round-1': ['check-round-0'],
    'vote-round-1': ['expand-round-1'],
    'check-round-1': ['vote-round-1'],
    'expand-round-2': ['check-round-1'],
    'vote-round-2': ['expand-round-2'],
    'check-round-2': ['vote-round-2'],
    'resolve-output': ['check-round-0', 'check-round-1', 'check-round-2'],
})
if errors:
    for e in errors:
        print(f'CHAIN_ERROR: {e}')
    sys.exit(1)
else:
    print('OK: task-refinement.lobster.yaml')

# pipeline.lobster.yaml
errors = check_depends_on('$INTAKE_DIR/workflows/pipeline.lobster.yaml', {
    'deliberation': ['codebase-analysis'],
    'intake': ['deliberation', 'codebase-analysis'],
})
if errors:
    for e in errors:
        print(f'CHAIN_ERROR: {e}')
    sys.exit(1)
else:
    print('OK: pipeline.lobster.yaml')
" 2>&1
  if [ $? -eq 0 ]; then
    pass "All depends_on chains are valid"
  else
    fail "depends_on chain validation" "$(python3 -c "..." 2>&1)"
  fi
else
  echo "  SKIP: python3 not available for DAG validation"
fi

# =========================================================================
# Test 11: Task refinement revision loop structure
# =========================================================================
echo ""
echo "=== Test 11: Task Refinement Revision Loop ==="

refinement_wf="$INTAKE_DIR/workflows/task-refinement.lobster.yaml"

# Check revision rounds exist (0, 1, 2)
for round in 0 1 2; do
  if grep -q "expand-round-${round}" "$refinement_wf" 2>/dev/null; then
    pass "task-refinement has expand-round-${round}"
  else
    fail "expand-round-${round} exists" "expand-round-${round} not found in task-refinement"
  fi

  if grep -q "vote-round-${round}" "$refinement_wf" 2>/dev/null; then
    pass "task-refinement has vote-round-${round}"
  else
    fail "vote-round-${round} exists" "vote-round-${round} not found in task-refinement"
  fi

  if grep -q "check-round-${round}" "$refinement_wf" 2>/dev/null; then
    pass "task-refinement has check-round-${round}"
  else
    fail "check-round-${round} exists" "check-round-${round} not found in task-refinement"
  fi
done

# Check resolve-output step exists
if grep -q "resolve-output" "$refinement_wf" 2>/dev/null; then
  pass "task-refinement has resolve-output step"
else
  fail "resolve-output exists" "resolve-output not found in task-refinement"
fi

# Check conditional when: on revision rounds (rounds 1 and 2 should be conditional)
when_count=$(grep -c "when:" "$refinement_wf" 2>/dev/null || true)
if [ "$when_count" -ge 6 ]; then
  pass "task-refinement has conditional when: clauses for revision rounds (found $when_count)"
else
  fail "conditional revision rounds" "Expected >= 6 when: clauses, found $when_count"
fi

# Check output_mappings includes expanded_tasks, verdict, warning
for mapping in expanded_tasks verdict revision_count warning; do
  if grep -q "$mapping" "$refinement_wf" 2>/dev/null; then
    pass "task-refinement outputs $mapping"
  else
    fail "output: $mapping" "$mapping not found in task-refinement output_mappings"
  fi
done

# =========================================================================
# Test 12: Pipeline conditional step combinations
# =========================================================================
echo ""
echo "=== Test 12: Pipeline Conditional Steps ==="

pipeline="$INTAKE_DIR/workflows/pipeline.lobster.yaml"

# Check that codebase-analysis has when: condition
if grep -A2 "name: codebase-analysis" "$pipeline" | grep -q "when:" 2>/dev/null; then
  pass "codebase-analysis step is conditional"
else
  fail "codebase-analysis conditional" "codebase-analysis missing when: clause"
fi

# Check that deliberation has when: condition
if grep -A2 "name: deliberation" "$pipeline" | grep -q "when:" 2>/dev/null; then
  pass "deliberation step is conditional"
else
  fail "deliberation conditional" "deliberation missing when: clause"
fi

# Check that intake step always runs (no when: condition)
# intake depends_on conditional steps but should always execute
intake_block=$(python3 -c "
import yaml
with open('$pipeline') as f:
    wf = yaml.safe_load(f)
for s in wf['steps']:
    if s['name'] == 'intake':
        print('has_when' if 'when' in s else 'no_when')
        break
" 2>/dev/null || echo "error")

if [ "$intake_block" = "no_when" ]; then
  pass "intake step has no when: condition (always runs)"
elif [ "$intake_block" = "has_when" ]; then
  fail "intake unconditional" "intake step should not have a when: condition"
else
  echo "  SKIP: could not parse pipeline for intake step condition check"
fi

# Check voting output_mappings exist
voting_wf="$INTAKE_DIR/workflows/voting.lobster.yaml"
if grep -q "output_mappings:" "$voting_wf" 2>/dev/null; then
  pass "voting workflow has output_mappings"
else
  fail "voting output_mappings" "voting.lobster.yaml missing output_mappings"
fi

for mapping in verdict suggestions tally_result; do
  if grep -q "$mapping" "$voting_wf" 2>/dev/null; then
    pass "voting outputs $mapping"
  else
    fail "voting output: $mapping" "$mapping not found in voting output_mappings"
  fi
done

# =========================================================================
# Results
# =========================================================================
echo ""
echo "=== Results ==="
echo "PASS: $PASS  FAIL: $FAIL  TOTAL: $((PASS + FAIL))"

if [ $FAIL -gt 0 ]; then
  echo ""
  echo "=== Failure Details ==="
  echo -e "$ERRORS"
  exit 1
fi

echo ""
echo "All integration tests passed."
exit 0
