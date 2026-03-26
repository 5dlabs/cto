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

for workflow in pipeline.lobster.yaml intake.lobster.yaml deliberation.lobster.yaml voting.lobster.yaml codebase-analysis.lobster.yaml task-refinement.lobster.yaml decision-voting.lobster.yaml; do
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
  codebase-analysis-system.md \
  voter-architect-soul.md voter-pragmatist-soul.md voter-minimalist-soul.md \
  voter-operator-soul.md voter-strategist-soul.md; do
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
  deliberation-result.schema.json decision-vote-response.schema.json \
  elicitation-request.schema.json elicitation-response.schema.json \
  scaffold.schema.json scale-tasks.schema.json security-report.schema.json \
  remediation-tasks.schema.json tool-manifest.schema.json \
  skill-recommendations.schema.json smart-docs.schema.json \
  smart-prompts.schema.json; do
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
# Test 4: intake-util validate (generic types)
# =========================================================================
echo ""
echo "=== Test 4: intake-util validate (generic types) ==="

UTIL_BIN="$INTAKE_DIR/../apps/intake-util/src/index.ts"
if [ -f "$UTIL_BIN" ] && command -v bun >/dev/null 2>&1; then
  INTAKE_UTIL="bun run $UTIL_BIN"

  # tasks
  if echo '[{"id":1,"title":"T","description":"D","dependencies":[]}]' | $INTAKE_UTIL validate --type tasks 2>/dev/null | python3 -c "import json,sys; assert json.load(sys.stdin)['valid']; print('OK')" 2>/dev/null | grep -q OK; then
    pass "validate --type tasks (valid)"
  else
    fail "validate --type tasks" "Expected valid:true"
  fi

  # tally
  if echo '{"verdict":"approve","vote_breakdown":{"approve":3}}' | $INTAKE_UTIL validate --type tally 2>/dev/null | python3 -c "import json,sys; assert json.load(sys.stdin)['valid']; print('OK')" 2>/dev/null | grep -q OK; then
    pass "validate --type tally (valid)"
  else
    fail "validate --type tally" "Expected valid:true"
  fi

  # debate-turn
  if echo 'This is a debate turn with content.' | $INTAKE_UTIL validate --type debate-turn 2>/dev/null | python3 -c "import json,sys; assert json.load(sys.stdin)['valid']; print('OK')" 2>/dev/null | grep -q OK; then
    pass "validate --type debate-turn (valid)"
  else
    fail "validate --type debate-turn" "Expected valid:true"
  fi
else
  echo "  SKIP: bun or intake-util not available"
fi

# =========================================================================
# Test 5: intake-util generate-workflows
# =========================================================================
echo ""
echo "=== Test 5: intake-util generate-workflows ==="

if [ -f "$UTIL_BIN" ] && command -v bun >/dev/null 2>&1; then
  INTAKE_UTIL="bun run $UTIL_BIN"
  GEN_INPUT='[{"id":1,"title":"Setup","description":"Init project","dependencies":[],"subtasks":[{"id":1,"title":"Init","description":"Create files","dependencies":[]}],"agent":"bolt","stack":"DevOps"}]'

  gen_output=$(echo "$GEN_INPUT" | $INTAKE_UTIL generate-workflows 2>/dev/null || true)
  if echo "$gen_output" | python3 -c "import json,sys; d=json.load(sys.stdin); assert len(d) > 0; assert 'workflow_yaml' in d[0]; print('OK')" 2>/dev/null | grep -q OK; then
    pass "generate-workflows produces workflow_yaml output"
  else
    fail "generate-workflows" "No valid workflow output"
  fi
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
    'generate-scaffolds': ['refine-tasks'],
    'generate-docs': ['refine-tasks', 'generate-scaffolds'],
    'discover-skills': ['refine-tasks'],
    'generate-tool-manifest': ['refine-tasks'],
    'generate-prompts': ['generate-docs', 'discover-skills', 'generate-tool-manifest', 'generate-scaffolds'],
    'generate-scale-tasks': ['refine-tasks'],
    'generate-security-report': ['generate-scale-tasks'],
    'generate-remediation-tasks': ['generate-security-report'],
    'commit-outputs': ['generate-prompts', 'generate-remediation-tasks'],
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
    'codebase-analysis': ['load-config'],
    'deliberation': ['load-config', 'build-infra-context', 'codebase-analysis'],
    'intake': ['load-config', 'deliberation', 'codebase-analysis', 'build-infra-context'],
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
# Test 13: Voting committee soul diversity
# =========================================================================
echo ""
echo "=== Test 13: Voting Committee Soul Diversity ==="

voting_wf="$INTAKE_DIR/workflows/voting.lobster.yaml"

# Check each voter references a distinct soul prompt
for soul in voter-architect-soul voter-pragmatist-soul voter-minimalist-soul \
  voter-operator-soul voter-strategist-soul; do
  if grep -q "$soul" "$voting_wf" 2>/dev/null; then
    pass "voting workflow references $soul"
  else
    fail "voting soul: $soul" "$soul not found in voting workflow"
  fi
done

# Check model diversity — input defaults should have at least 3 distinct providers
provider_count=$(grep -A2 'name: voter_[1-5]_provider' "$voting_wf" 2>/dev/null | grep 'default:' | sort -u | wc -l | tr -d ' ')
if [ "$provider_count" -ge 3 ]; then
  pass "voting committee defaults use $provider_count distinct providers"
else
  fail "provider diversity" "Expected >= 3 default providers, found $provider_count"
fi

# Check model diversity — input defaults should have at least 5 distinct models
model_count=$(grep -A2 'name: voter_[1-5]_model' "$voting_wf" 2>/dev/null | grep 'default:' | sort -u | wc -l | tr -d ' ')
if [ "$model_count" -ge 5 ]; then
  pass "voting committee defaults use $model_count distinct models"
else
  fail "model diversity" "Expected 5 distinct default models, found $model_count"
fi

# Check voter steps use template variables (not hardcoded providers)
if grep -q '{{inputs.voter_1_provider}}' "$voting_wf" 2>/dev/null; then
  pass "voting steps use configurable model inputs"
else
  fail "voting inputs" "Voter steps should reference {{inputs.voter_*}} template variables"
fi

# Check each soul prompt has Identity and Evaluation Lens sections
for soul in voter-architect-soul voter-pragmatist-soul voter-minimalist-soul \
  voter-operator-soul voter-strategist-soul; do
  filepath="$INTAKE_DIR/prompts/${soul}.md"
  if [ -f "$filepath" ]; then
    has_identity=$(grep -c "^# Identity" "$filepath" 2>/dev/null || true)
    has_lens=$(grep -c "^# Evaluation Lens" "$filepath" 2>/dev/null || true)
    has_bias=$(grep -c "^# Scoring Bias" "$filepath" 2>/dev/null || true)
    has_voice=$(grep -c "^# Voice" "$filepath" 2>/dev/null || true)
    if [ "$has_identity" -ge 1 ] && [ "$has_lens" -ge 1 ] && [ "$has_bias" -ge 1 ] && [ "$has_voice" -ge 1 ]; then
      pass "$soul has all 4 sections (Identity, Evaluation Lens, Scoring Bias, Voice)"
    else
      fail "$soul structure" "Missing sections: Identity=$has_identity Lens=$has_lens Bias=$has_bias Voice=$has_voice"
    fi
  fi
done

# =========================================================================
# Test 14: Multi-Source Research
# =========================================================================
echo ""
echo "=== Test 14: Multi-Source Research ==="

research_src="$INTAKE_DIR/../apps/intake-agent/src/operations"

# Check research-sources.ts exists with all provider functions
if [ -f "$research_src/research-sources.ts" ]; then
  pass "research-sources.ts exists"
else
  fail "research-sources.ts" "research-sources.ts not found"
fi

for func in exaSearch perplexityAsk firecrawlExtract selectUrlsForDeepExtract; do
  if grep -q "$func" "$research_src/research-sources.ts" 2>/dev/null; then
    pass "research-sources exports $func"
  else
    fail "research func: $func" "$func not found in research-sources.ts"
  fi
done

# Check prd-research.ts imports from research-sources
if grep -q "from.*research-sources" "$research_src/prd-research.ts" 2>/dev/null; then
  pass "prd-research imports from research-sources"
else
  fail "prd-research import" "prd-research.ts does not import research-sources"
fi

# Check Phase 1 parallel searches
if grep -q "Promise.all" "$research_src/prd-research.ts" 2>/dev/null; then
  pass "prd-research uses Promise.all for parallel searches"
else
  fail "parallel searches" "No Promise.all found in prd-research.ts"
fi

# Check source attribution in memos
if grep -q "Source:" "$research_src/prd-research.ts" 2>/dev/null; then
  pass "prd-research includes source attribution in memos"
else
  fail "source attribution" "No [Source: ...] attribution found"
fi

# =========================================================================
# Test 15: Operator Catalog & Infrastructure Context
# =========================================================================
echo ""
echo "=== Test 15: Operator Catalog & Infrastructure Context ==="

catalog="$INTAKE_DIR/catalogs/operator-catalog.yaml"

if [ -f "$catalog" ]; then
  pass "operator-catalog.yaml exists"
else
  fail "operator-catalog" "operator-catalog.yaml not found"
fi

# Check key operators are in catalog
for op in cloudnative-pg redis-operator seaweedfs kubeai-operator; do
  if grep -q "$op" "$catalog" 2>/dev/null; then
    pass "catalog includes $op"
  else
    fail "catalog: $op" "$op not found in operator-catalog.yaml"
  fi
done

# Check pipeline has build-infra-context step
pipeline="$INTAKE_DIR/workflows/pipeline.lobster.yaml"
if grep -q "build-infra-context" "$pipeline" 2>/dev/null; then
  pass "pipeline has build-infra-context step"
else
  fail "build-infra-context step" "build-infra-context not found in pipeline"
fi

# Check deliberation depends on build-infra-context
if python3 -c "
import yaml
with open('$pipeline') as f:
    wf = yaml.safe_load(f)
for s in wf['steps']:
    if s['name'] == 'deliberation':
        assert 'build-infra-context' in s.get('depends_on', [])
        print('OK')
        break
" 2>/dev/null | grep -q OK; then
  pass "deliberation depends on build-infra-context"
else
  fail "deliberation depends_on" "deliberation does not depend on build-infra-context"
fi

# =========================================================================
# Test 16: Skill Discovery
# =========================================================================
echo ""
echo "=== Test 16: Skill Discovery ==="

intake_wf="$INTAKE_DIR/workflows/intake.lobster.yaml"

if grep -q "discover-skills" "$intake_wf" 2>/dev/null; then
  pass "intake workflow has discover-skills step"
else
  fail "discover-skills step" "discover-skills not found in intake.lobster.yaml"
fi

if [ -f "$INTAKE_DIR/prompts/skill-discovery-system.md" ]; then
  pass "skill-discovery-system.md prompt exists"
else
  fail "skill-discovery prompt" "skill-discovery-system.md not found"
fi

# Check generate-prompts depends on discover-skills
if python3 -c "
import yaml
with open('$intake_wf') as f:
    wf = yaml.safe_load(f)
for s in wf['steps']:
    if s['name'] == 'generate-prompts':
        assert 'discover-skills' in s.get('depends_on', [])
        print('OK')
        break
" 2>/dev/null | grep -q OK; then
  pass "generate-prompts depends on discover-skills"
else
  fail "generate-prompts depends_on" "generate-prompts does not depend on discover-skills"
fi

# =========================================================================
# Test 17: Tool Manifest
# =========================================================================
echo ""
echo "=== Test 17: Tool Manifest ==="

if grep -q "generate-tool-manifest" "$intake_wf" 2>/dev/null; then
  pass "intake workflow has generate-tool-manifest step"
else
  fail "generate-tool-manifest step" "generate-tool-manifest not found"
fi

for f in tool-catalog.yaml; do
  if [ -f "$INTAKE_DIR/catalogs/$f" ]; then
    pass "$f catalog exists"
  else
    fail "catalog: $f" "$f not found in catalogs/"
  fi
done

if [ -f "$INTAKE_DIR/prompts/tool-manifest-system.md" ]; then
  pass "tool-manifest-system.md prompt exists"
else
  fail "tool-manifest prompt" "tool-manifest-system.md not found"
fi

if [ -f "$INTAKE_DIR/schemas/tool-manifest.schema.json" ]; then
  pass "tool-manifest.schema.json exists"
else
  fail "tool-manifest schema" "tool-manifest.schema.json not found"
fi

# =========================================================================
# Test 18: Bolt Infrastructure Ordering
# =========================================================================
echo ""
echo "=== Test 18: Bolt Infrastructure Ordering ==="

parse_prompt="$INTAKE_DIR/prompts/parse-prd-system.md"

if grep -q "Infrastructure Task Ordering" "$parse_prompt" 2>/dev/null; then
  pass "parse-prd prompt has Infrastructure Task Ordering section"
else
  fail "bolt ordering" "Infrastructure Task Ordering not found in parse-prd-system.md"
fi

if grep -q "Secrets Distribution" "$parse_prompt" 2>/dev/null; then
  pass "parse-prd prompt has Secrets Distribution Pattern"
else
  fail "secrets pattern" "Secrets Distribution not found in parse-prd-system.md"
fi

if [ -f "$INTAKE_DIR/prompts/bolt-dev-infra-pattern.md" ]; then
  pass "bolt-dev-infra-pattern.md exists"
else
  fail "bolt-dev-infra" "bolt-dev-infra-pattern.md not found"
fi

# =========================================================================
# Test 19: Security Pipeline (scale → security → remediation)
# =========================================================================
echo ""
echo "=== Test 19: Security Pipeline ==="

for step in generate-scale-tasks generate-security-report generate-remediation-tasks; do
  if grep -q "$step" "$intake_wf" 2>/dev/null; then
    pass "intake workflow has $step step"
  else
    fail "step: $step" "$step not found in intake.lobster.yaml"
  fi
done

# Check sequential dependency chain
if python3 -c "
import yaml
with open('$intake_wf') as f:
    wf = yaml.safe_load(f)
steps = {s['name']: s.get('depends_on', []) for s in wf['steps']}
assert 'generate-scale-tasks' in steps.get('generate-security-report', [])
assert 'generate-security-report' in steps.get('generate-remediation-tasks', [])
assert 'generate-remediation-tasks' in steps.get('commit-outputs', [])
print('OK')
" 2>/dev/null | grep -q OK; then
  pass "security pipeline chain: scale -> security -> remediation -> commit"
else
  fail "security chain" "Security pipeline dependency chain is broken"
fi

# Check prompts exist
for prompt in bolt-production-hardening-system security-report-system security-remediation-system; do
  if [ -f "$INTAKE_DIR/prompts/${prompt}.md" ]; then
    pass "${prompt}.md prompt exists"
  else
    fail "prompt: $prompt" "${prompt}.md not found"
  fi
done

# Check schemas exist
for schema in scale-tasks security-report remediation-tasks; do
  if [ -f "$INTAKE_DIR/schemas/${schema}.schema.json" ]; then
    pass "${schema}.schema.json exists"
  else
    fail "schema: $schema" "${schema}.schema.json not found"
  fi
done

# Check security report uses frontier tier (configurable, defaults to Opus)
if grep -A15 "name: generate-security-report" "$intake_wf" | grep -q "model_frontier" 2>/dev/null; then
  pass "security report uses frontier model tier"
else
  fail "security model" "Security report should use frontier model tier ({{inputs.model_frontier}})"
fi

# =========================================================================
# Test 20: GitHub CLI in Agent Image
# =========================================================================
echo ""
echo "=== Test 20: GitHub CLI in Agent Image ==="

dockerfile="$INTAKE_DIR/../infra/images/agents/Dockerfile"
if [ -f "$dockerfile" ]; then
  if grep -q "gh" "$dockerfile" 2>/dev/null; then
    pass "Dockerfile includes gh CLI installation"
  else
    fail "gh CLI" "gh not found in Dockerfile"
  fi
else
  echo "  SKIP: Dockerfile not found at expected path"
fi

# =========================================================================
# Test 21: Agent Field in Task Schema
# =========================================================================
echo ""
echo "=== Test 21: Agent Field in Task Schema ==="

schema_file="$INTAKE_DIR/schemas/generated-task.schema.json"

# Check agent field exists in schema
if python3 -c "
import json
with open('$schema_file') as f:
    schema = json.load(f)
props = schema['\$defs']['generated_task']['properties']
assert 'agent' in props, 'agent field missing'
assert 'enum' in props['agent'], 'agent missing enum constraint'
assert 'stack' in props, 'stack field missing'
required = schema['\$defs']['generated_task']['required']
assert 'agent' in required, 'agent not in required'
print('OK')
" 2>/dev/null | grep -q OK; then
  pass "generated-task.schema.json has agent/stack fields with enum and required"
else
  fail "agent field in schema" "agent/stack fields not properly defined in schema"
fi

# Check fixtures have agent field
for size in small medium large; do
  fixture="$FIXTURES_DIR/tasks-${size}.json"
  if [ -f "$fixture" ]; then
    agent_count=$(python3 -c "
import json
with open('$fixture') as f:
    tasks = json.load(f)
print(sum(1 for t in tasks if 'agent' in t))
" 2>/dev/null || echo "0")
    total_count=$(python3 -c "
import json
with open('$fixture') as f:
    tasks = json.load(f)
print(len(tasks))
" 2>/dev/null || echo "0")
    if [ "$agent_count" = "$total_count" ] && [ "$total_count" -gt 0 ]; then
      pass "tasks-${size}.json: all $total_count tasks have agent field"
    else
      fail "fixture agent field ($size)" "Only $agent_count of $total_count tasks have agent field"
    fi
  fi
done

# Check that old deterministic generate-docs/generate-prompts have been removed
# (replaced by LLM-powered openclaw.invoke llm-task in workflows)
if [ ! -f "$INTAKE_DIR/util/src/generate-docs.ts" ] && [ ! -f "$INTAKE_DIR/util/src/generate-prompts.ts" ]; then
  pass "Old deterministic generate-docs.ts and generate-prompts.ts removed (replaced by LLM-task)"
else
  fail "old generators removed" "generate-docs.ts or generate-prompts.ts still exists in intake/util"
fi

# =========================================================================
# Test 22: Code Scaffold Pipeline
# =========================================================================
echo ""
echo "=== Test 22: Code Scaffold Pipeline ==="

# Check generate-scaffolds step exists in intake workflow
if grep -q "generate-scaffolds" "$intake_wf" 2>/dev/null; then
  pass "intake workflow has generate-scaffolds step"
else
  fail "generate-scaffolds step" "generate-scaffolds not found in intake.lobster.yaml"
fi

# Check scaffold prompt exists
if [ -f "$INTAKE_DIR/prompts/scaffold-generation-system.md" ]; then
  pass "scaffold-generation-system.md prompt exists"
else
  fail "scaffold prompt" "scaffold-generation-system.md not found"
fi

# Check scaffold schema exists and is valid JSON
scaffold_schema="$INTAKE_DIR/schemas/scaffold.schema.json"
if [ -f "$scaffold_schema" ]; then
  if python3 -c "import json; json.load(open('$scaffold_schema'))" 2>/dev/null; then
    pass "scaffold.schema.json exists and is valid JSON"
  else
    fail "scaffold schema" "scaffold.schema.json is not valid JSON"
  fi
else
  fail "scaffold schema" "scaffold.schema.json not found"
fi

# Check generate-scaffolds depends on refine-tasks
if python3 -c "
import yaml
with open('$intake_wf') as f:
    wf = yaml.safe_load(f)
for s in wf['steps']:
    if s['name'] == 'generate-scaffolds':
        assert 'refine-tasks' in s.get('depends_on', [])
        print('OK')
        break
" 2>/dev/null | grep -q OK; then
  pass "generate-scaffolds depends on refine-tasks"
else
  fail "generate-scaffolds depends_on" "generate-scaffolds does not depend on refine-tasks"
fi

# Check generate-docs depends on generate-scaffolds
if python3 -c "
import yaml
with open('$intake_wf') as f:
    wf = yaml.safe_load(f)
for s in wf['steps']:
    if s['name'] == 'generate-docs':
        assert 'generate-scaffolds' in s.get('depends_on', [])
        print('OK')
        break
" 2>/dev/null | grep -q OK; then
  pass "generate-docs depends on generate-scaffolds"
else
  fail "generate-docs depends_on scaffolds" "generate-docs does not depend on generate-scaffolds"
fi

# Check generate-prompts depends on generate-scaffolds
if python3 -c "
import yaml
with open('$intake_wf') as f:
    wf = yaml.safe_load(f)
for s in wf['steps']:
    if s['name'] == 'generate-prompts':
        assert 'generate-scaffolds' in s.get('depends_on', [])
        print('OK')
        break
" 2>/dev/null | grep -q OK; then
  pass "generate-prompts depends on generate-scaffolds"
else
  fail "generate-prompts depends_on scaffolds" "generate-prompts does not depend on generate-scaffolds"
fi

# Check scaffold prompt references agent/stack fields
if grep -q "agent.*stack\|stack.*agent" "$INTAKE_DIR/prompts/scaffold-generation-system.md" 2>/dev/null; then
  pass "scaffold prompt references agent and stack fields"
else
  fail "scaffold prompt content" "scaffold prompt does not reference agent/stack"
fi

# Check scaffold schema has required structure
if python3 -c "
import json
with open('$scaffold_schema') as f:
    schema = json.load(f)
scaffold_props = schema['properties']['scaffolds']['items']['properties']
assert 'task_id' in scaffold_props
assert 'file_structure' in scaffold_props
assert 'interfaces' in scaffold_props
assert 'function_signatures' in scaffold_props
assert 'test_stubs' in scaffold_props
assert 'skip_reason' in scaffold_props
print('OK')
" 2>/dev/null | grep -q OK; then
  pass "scaffold schema has all required properties"
else
  fail "scaffold schema structure" "scaffold schema missing expected properties"
fi

# =========================================================================
# Test 23: Model Configuration Pipeline
# =========================================================================
echo ""
echo "=== Test 23: Model Configuration Pipeline ==="

# Check cto-config.json has tiers and committee structure
config_file="$INTAKE_DIR/../cto-config.json"
if [ -f "$config_file" ]; then
  if python3 -c "
import json
with open('$config_file') as f:
    c = json.load(f)
models = c['defaults']['intake']['models']
assert 'tiers' in models, 'missing tiers'
assert 'committee' in models, 'missing committee'
tiers = models['tiers']
for tier in ['primary', 'fast', 'frontier']:
    assert tier in tiers, f'missing tier: {tier}'
    assert 'provider' in tiers[tier], f'{tier} missing provider'
    assert 'model' in tiers[tier], f'{tier} missing model'
committee = models['committee']
assert len(committee) == 5, f'expected 5 committee members, got {len(committee)}'
for i, v in enumerate(committee):
    assert 'provider' in v, f'committee[{i}] missing provider'
    assert 'model' in v, f'committee[{i}] missing model'
print('OK')
" 2>/dev/null | grep -q OK; then
    pass "cto-config.json has tiers + committee model structure"
  else
    fail "config model structure" "cto-config.json missing tiers/committee structure under defaults.intake.models"
  fi
else
  fail "cto-config.json" "cto-config.json not found"
fi

# Check pipeline has load-config step
pipeline="$INTAKE_DIR/workflows/pipeline.lobster.yaml"
if grep -q "name: load-config" "$pipeline" 2>/dev/null; then
  pass "pipeline has load-config step"
else
  fail "load-config step" "load-config not found in pipeline.lobster.yaml"
fi

# Check load-config reads cto-config.json
if grep -q "cto-config.json" "$pipeline" 2>/dev/null; then
  pass "load-config reads cto-config.json"
else
  fail "load-config reads config" "cto-config.json not referenced in pipeline"
fi

# Check pipeline threads model inputs to sub-workflows
for input_name in model_primary_provider model_primary model_fast_provider model_fast model_frontier_provider model_frontier; do
  if grep -q "$input_name" "$pipeline" 2>/dev/null; then
    pass "pipeline threads $input_name to sub-workflows"
  else
    fail "pipeline threading: $input_name" "$input_name not found in pipeline.lobster.yaml"
  fi
done

# Check all workflow steps use template variables instead of hardcoded providers
for wf in intake.lobster.yaml task-refinement.lobster.yaml voting.lobster.yaml deliberation.lobster.yaml codebase-analysis.lobster.yaml; do
  filepath="$INTAKE_DIR/workflows/$wf"
  # Count hardcoded provider/model pairs in step commands (not in input defaults)
  hardcoded=$(python3 -c "
import yaml
with open('$filepath') as f:
    wf = yaml.safe_load(f)
count = 0
for s in wf.get('steps', []):
    cmd = s.get('command', '')
    # Count literal provider strings in commands (not in input defaults)
    if '\"provider\": \"anthropic\"' in cmd or '\"provider\": \"openai\"' in cmd or '\"provider\": \"google\"' in cmd:
        count += 1
print(count)
" 2>/dev/null || echo "error")
  if [ "$hardcoded" = "0" ]; then
    pass "$wf has no hardcoded provider/model in step commands"
  elif [ "$hardcoded" = "error" ]; then
    echo "  SKIP: could not parse $wf for hardcoded provider check"
  else
    fail "$wf hardcoded models" "Found $hardcoded step commands with hardcoded provider/model"
  fi
done

# Check voter inputs are threaded from pipeline -> intake -> task-refinement -> voting
for wf_name in intake.lobster.yaml task-refinement.lobster.yaml; do
  filepath="$INTAKE_DIR/workflows/$wf_name"
  if grep -q "voter_1_provider" "$filepath" 2>/dev/null && grep -q "voter_5_model" "$filepath" 2>/dev/null; then
    pass "$wf_name threads voter inputs"
  else
    fail "$wf_name voter threading" "voter inputs not fully threaded in $wf_name"
  fi
done

# =========================================================================
# Test 24: NATS Removal — No NATS References in Bridges
# =========================================================================
echo ""
echo "=== Test 24: NATS Removal ==="

BRIDGE_DIR="$INTAKE_DIR/../apps"

# Discord bridge: no nats imports, no nats-tap.ts
if [ -f "$BRIDGE_DIR/discord-bridge/src/nats-tap.ts" ]; then
  fail "discord-bridge nats-tap removed" "nats-tap.ts still exists"
else
  pass "discord-bridge nats-tap.ts removed"
fi

if grep -rq '"nats"' "$BRIDGE_DIR/discord-bridge/package.json" 2>/dev/null; then
  fail "discord-bridge nats dependency" "nats still in package.json"
else
  pass "discord-bridge package.json has no nats dependency"
fi

# Linear bridge: no nats imports, no nats-tap.ts, no webhook-server.ts
if [ -f "$BRIDGE_DIR/linear-bridge/src/nats-tap.ts" ]; then
  fail "linear-bridge nats-tap removed" "nats-tap.ts still exists"
else
  pass "linear-bridge nats-tap.ts removed"
fi

if [ -f "$BRIDGE_DIR/linear-bridge/src/webhook-server.ts" ]; then
  fail "linear-bridge webhook-server removed" "webhook-server.ts still exists (replaced by http-server.ts)"
else
  pass "linear-bridge webhook-server.ts removed (replaced by http-server.ts)"
fi

if grep -rq '"nats"' "$BRIDGE_DIR/linear-bridge/package.json" 2>/dev/null; then
  fail "linear-bridge nats dependency" "nats still in package.json"
else
  pass "linear-bridge package.json has no nats dependency"
fi

# intake-util: no nats imports
if [ -f "$BRIDGE_DIR/intake-util/src/nats-notify.ts" ]; then
  fail "intake-util nats-notify removed" "nats-notify.ts still exists"
else
  pass "intake-util nats-notify.ts removed (replaced by bridge-notify.ts)"
fi

if [ -f "$BRIDGE_DIR/intake-util/src/nats-wait-elicitation.ts" ]; then
  fail "intake-util nats-wait removed" "nats-wait-elicitation.ts still exists"
else
  pass "intake-util nats-wait-elicitation.ts removed"
fi

# No NATS references in workflow files
for wf in pipeline.lobster.yaml intake.lobster.yaml deliberation.lobster.yaml decision-voting.lobster.yaml; do
  filepath="$INTAKE_DIR/workflows/$wf"
  if grep -q "nats-notify\|nats-wait-elicitation\|NATS_URL" "$filepath" 2>/dev/null; then
    fail "$wf NATS references" "Found NATS references in $wf"
  else
    pass "$wf has no NATS references"
  fi
done

# =========================================================================
# Test 25: HTTP Bridge Architecture
# =========================================================================
echo ""
echo "=== Test 25: HTTP Bridge Architecture ==="

# Discord bridge has http-server.ts
if [ -f "$BRIDGE_DIR/discord-bridge/src/http-server.ts" ]; then
  pass "discord-bridge http-server.ts exists"
else
  fail "discord-bridge http-server" "http-server.ts not found"
fi

# Linear bridge has http-server.ts and run-registry.ts
if [ -f "$BRIDGE_DIR/linear-bridge/src/http-server.ts" ]; then
  pass "linear-bridge http-server.ts exists"
else
  fail "linear-bridge http-server" "http-server.ts not found"
fi

if [ -f "$BRIDGE_DIR/linear-bridge/src/run-registry.ts" ]; then
  pass "linear-bridge run-registry.ts exists"
else
  fail "linear-bridge run-registry" "run-registry.ts not found"
fi

# intake-util HTTP commands exist
for cmd_file in bridge-notify.ts bridge-elicitation.ts linear-activity.ts linear-plan.ts run-registry-client.ts invoke-agent.ts log.ts classify-output.ts; do
  if [ -f "$BRIDGE_DIR/intake-util/src/$cmd_file" ]; then
    pass "intake-util $cmd_file exists"
  else
    fail "intake-util $cmd_file" "$cmd_file not found"
  fi
done

# Cloudflare TunnelBinding manifest exists
if [ -f "$INTAKE_DIR/../infra/manifests/linear-bridge/tunnel-binding.yaml" ]; then
  pass "tunnel-binding.yaml exists"
  if grep -q "agents.5dlabs.ai" "$INTAKE_DIR/../infra/manifests/linear-bridge/tunnel-binding.yaml" 2>/dev/null; then
    pass "tunnel-binding routes agents.5dlabs.ai"
  else
    fail "tunnel-binding hostname" "agents.5dlabs.ai not found in tunnel-binding.yaml"
  fi
else
  fail "tunnel-binding" "tunnel-binding.yaml not found"
fi

# Discord bridge HTTP service manifest exists
if [ -f "$INTAKE_DIR/../infra/manifests/discord-bridge/service-http.yaml" ]; then
  pass "discord-bridge service-http.yaml exists"
else
  fail "discord-bridge service" "service-http.yaml not found"
fi

# =========================================================================
# Test 26: Toggleable Verification Steps
# =========================================================================
echo ""
echo "=== Test 26: Toggleable Verification Steps ==="

intake_wf="$INTAKE_DIR/workflows/intake.lobster.yaml"
delib_wf="$INTAKE_DIR/workflows/deliberation.lobster.yaml"

# Check verify and breakpoints inputs exist in intake workflow
for input_name in verify breakpoints; do
  if grep -q "name: $input_name" "$intake_wf" 2>/dev/null; then
    pass "intake workflow has $input_name input"
  else
    fail "intake $input_name input" "$input_name input not found in intake.lobster.yaml"
  fi
done

# Check verify input exists in deliberation workflow
if grep -q "name: verify" "$delib_wf" 2>/dev/null; then
  pass "deliberation workflow has verify input"
else
  fail "deliberation verify input" "verify input not found in deliberation.lobster.yaml"
fi

# Check verify steps exist (grep for verify- prefixed step names)
verify_steps=$(grep -c "name: verify-" "$intake_wf" 2>/dev/null || true)
if [ "$verify_steps" -ge 3 ]; then
  pass "intake workflow has $verify_steps verify steps"
else
  fail "intake verify steps" "Expected >= 3 verify steps, found $verify_steps"
fi

# Check verify steps use when: condition
if grep -A1 "name: verify-" "$intake_wf" | grep -q "when:" 2>/dev/null; then
  pass "intake verify steps are conditional (have when:)"
else
  fail "verify conditional" "verify steps should have when: condition"
fi

# =========================================================================
# Test 27: Linear Activity Integration
# =========================================================================
echo ""
echo "=== Test 27: Linear Activity Integration ==="

# Check linear-activity calls exist in workflows
if grep -q "linear-activity\|bridge-notify" "$delib_wf" 2>/dev/null; then
  pass "deliberation workflow has linear-activity/bridge-notify calls"
else
  fail "deliberation linear-activity" "No linear-activity or bridge-notify calls in deliberation"
fi

# Check bridge-notify replaced nats-notify in deliberation
bridge_notify_count=$(grep -c "bridge-notify" "$delib_wf" 2>/dev/null || true)
if [ "$bridge_notify_count" -ge 1 ]; then
  pass "deliberation uses bridge-notify ($bridge_notify_count calls)"
else
  fail "deliberation bridge-notify" "No bridge-notify calls found"
fi

# Check linear_session_id input threaded through pipeline
if grep -q "linear_session_id" "$INTAKE_DIR/workflows/pipeline.lobster.yaml" 2>/dev/null; then
  pass "pipeline.lobster.yaml threads linear_session_id"
else
  fail "pipeline linear_session_id" "linear_session_id not found in pipeline"
fi

# =========================================================================
# Test 28: Agent Communication Mode
# =========================================================================
echo ""
echo "=== Test 28: Agent Communication Mode ==="

# Check cto-config.template.json has agentCommunication
config_template="$INTAKE_DIR/../cto-config.template.json"
if grep -q "agentCommunication" "$config_template" 2>/dev/null; then
  pass "cto-config.template.json has agentCommunication field"
else
  fail "agentCommunication" "agentCommunication not found in template config"
fi

# Check invoke-agent supports both modes
invoke_src="$BRIDGE_DIR/intake-util/src/invoke-agent.ts"
if [ -f "$invoke_src" ]; then
  if grep -q "subagent" "$invoke_src" && grep -q "a2a" "$invoke_src" && grep -q "acp" "$invoke_src"; then
    pass "invoke-agent supports subagent and a2a modes with acp alias"
  else
    fail "invoke-agent modes" "invoke-agent.ts missing subagent/a2a mode support"
  fi
else
  fail "invoke-agent.ts" "invoke-agent.ts not found"
fi

# =========================================================================
# Test 29: Run Registration
# =========================================================================
echo ""
echo "=== Test 29: Run Registration ==="

# Check pipeline has preflight, start notify, register-run, deregister-run steps
pipeline="$INTAKE_DIR/workflows/pipeline.lobster.yaml"
if grep -q "id: preflight" "$pipeline" 2>/dev/null; then
  pass "pipeline has preflight step"
else
  fail "preflight step" "preflight not found in pipeline"
fi
if grep -q "notify-pipeline-start" "$pipeline" 2>/dev/null; then
  pass "pipeline has notify-pipeline-start step"
else
  fail "notify-pipeline-start step" "notify-pipeline-start not found in pipeline"
fi
if grep -q "register-run" "$pipeline" 2>/dev/null; then
  pass "pipeline has register-run step"
else
  fail "register-run step" "register-run not found in pipeline"
fi

if grep -q "deregister-run" "$pipeline" 2>/dev/null; then
  pass "pipeline has deregister-run step"
else
  fail "deregister-run step" "deregister-run not found in pipeline"
fi

# Check run_id input exists
if grep -q "run_id" "$pipeline" 2>/dev/null; then
  pass "pipeline has run_id input"
else
  fail "pipeline run_id" "run_id not found in pipeline"
fi

# =========================================================================
# Test 30: Design Intake + Stitch wiring
# =========================================================================
echo ""
echo "=== Test 30: Design Intake + Stitch Wiring ==="

pipeline="$INTAKE_DIR/workflows/pipeline.lobster.yaml"
intake_wf="$INTAKE_DIR/workflows/intake.lobster.yaml"
delib_wf="$INTAKE_DIR/workflows/deliberation.lobster.yaml"
agent_index="$INTAKE_DIR/../apps/intake-agent/src/index.ts"
agent_types="$INTAKE_DIR/../apps/intake-agent/src/types.ts"

for arg_name in design_prompt design_artifacts_path design_urls design_mode; do
  if grep -q "$arg_name" "$pipeline" 2>/dev/null; then
    pass "pipeline has $arg_name argument"
  else
    fail "pipeline arg: $arg_name" "$arg_name not found in pipeline args"
  fi
done

if grep -q "id: materialize-design-inputs" "$pipeline" 2>/dev/null; then
  pass "pipeline has materialize-design-inputs step"
else
  fail "materialize-design-inputs step" "materialize-design-inputs not found in pipeline"
fi

if grep -q "design_context" "$pipeline" 2>/dev/null; then
  pass "pipeline threads design_context"
else
  fail "pipeline design_context threading" "design_context not found in pipeline"
fi

if grep -q "design_context" "$intake_wf" 2>/dev/null; then
  pass "intake workflow accepts design_context"
else
  fail "intake design_context" "design_context not found in intake workflow"
fi

if grep -q "design_context" "$delib_wf" 2>/dev/null; then
  pass "deliberation workflow accepts design_context"
else
  fail "deliberation design_context" "design_context not found in deliberation workflow"
fi

if grep -q "design_intake" "$agent_index" 2>/dev/null && grep -q "design_intake" "$agent_types" 2>/dev/null; then
  pass "intake-agent supports design_intake operation"
else
  fail "design_intake operation" "design_intake operation missing from intake-agent"
fi

for fixture in \
  design-intake-greenfield-frontend.json \
  design-intake-existing-project.json \
  design-intake-backend-only.json \
  design-intake-sketch-mockup.json \
  design-intake-sigma1-site-improvement.json; do
  fixture_path="$FIXTURES_DIR/$fixture"
  if [ ! -f "$fixture_path" ]; then
    fail "fixture exists: $fixture" "Missing fixture: $fixture_path"
    continue
  fi
  if python3 -c "import json; json.load(open('$fixture_path'))" 2>/dev/null; then
    pass "fixture valid JSON: $fixture"
  else
    fail "fixture JSON: $fixture" "Could not parse JSON fixture: $fixture_path"
  fi
done

sigma_fixture="$FIXTURES_DIR/design-intake-sigma1-site-improvement.json"
if python3 -c "
import json
with open('$sigma_fixture') as f:
    data = json.load(f)
urls = data.get('design_urls') or []
assert any('sigma1.led.video' in u for u in urls)
assert data.get('project_name') == 'sigma-1'
print('OK')
" 2>/dev/null | grep -q OK; then
  pass "sigma-1 fixture includes sigma1 URL and project name"
else
  fail "sigma-1 fixture content" "sigma fixture missing expected sigma1 fields"
fi

if grep -q "id: verify-artifact-gates" "$intake_wf" 2>/dev/null; then
  pass "intake workflow has verify-artifact-gates step"
else
  fail "verify-artifact-gates step" "verify-artifact-gates not found in intake workflow"
fi

if grep -q ".tasks/tasks/tasks.json" "$intake_wf" 2>/dev/null; then
  pass "intake workflow writes tasks snapshot artifact"
else
  fail "tasks snapshot artifact" ".tasks/tasks/tasks.json write not found in intake workflow"
fi

if grep -q "id: save-design-bundle" "$pipeline" 2>/dev/null; then
  pass "pipeline has save-design-bundle step"
else
  fail "save-design-bundle step" "save-design-bundle not found in pipeline"
fi

if grep -q "id: persist-design-metadata" "$pipeline" 2>/dev/null; then
  pass "pipeline has persist-design-metadata step"
else
  fail "persist-design-metadata step" "persist-design-metadata not found in pipeline"
fi

if grep -q "stitch-credentials" "$pipeline" 2>/dev/null && grep -q "STITCH_API_KEY is required" "$pipeline" 2>/dev/null; then
  pass "pipeline has hard stitch credential gate"
else
  fail "stitch credential gate" "Missing hard stitch credential gate in pipeline"
fi

linear_http="$INTAKE_DIR/../apps/linear-bridge/src/http-server.ts"
discord_http="$INTAKE_DIR/../apps/discord-bridge/src/http-server.ts"
if grep -q "/history/design" "$linear_http" 2>/dev/null && grep -q "/history/design-snapshot" "$linear_http" 2>/dev/null; then
  pass "linear bridge exposes design history/snapshot endpoints"
else
  fail "linear design history endpoints" "Missing /history/design and/or /history/design-snapshot in linear bridge"
fi
if grep -q "/history/design" "$discord_http" 2>/dev/null && grep -q "/history/design-snapshot" "$discord_http" 2>/dev/null; then
  pass "discord bridge exposes design history/snapshot endpoints"
else
  fail "discord design history endpoints" "Missing /history/design and/or /history/design-snapshot in discord bridge"
fi

if command -v bun >/dev/null 2>&1; then
  tmp_out="/tmp/design-intake-it-$$"
  mkdir -p "$tmp_out"
  if bun --eval "
import { readFileSync, existsSync } from 'node:fs';
import { designIntake } from '$INTAKE_DIR/../apps/intake-agent/src/operations/design-intake.ts';
const fixture = JSON.parse(readFileSync('$sigma_fixture', 'utf8'));
const res = await designIntake({
  prd_content: fixture.prd_content ?? '',
  design_prompt: fixture.design_prompt ?? '',
  design_urls: JSON.stringify(fixture.design_urls ?? []),
  design_mode: 'ingest_only',
  output_dir: '$tmp_out',
  project_name: fixture.project_name ?? 'sigma-1'
});
if (!existsSync('$tmp_out/design-context.json')) throw new Error('missing design-context.json');
if (!existsSync('$tmp_out/crawled/urls.json')) throw new Error('missing crawled/urls.json');
if (!res.hasFrontend) throw new Error('expected hasFrontend=true');
if (!Array.isArray(res.frontendTargets) || !res.frontendTargets.includes('web')) throw new Error('expected web frontend target');
" >/dev/null 2>&1; then
    pass "designIntake executes and materializes expected outputs"
  else
    fail "designIntake execution" "designIntake execution smoke failed for sigma fixture"
  fi
else
  fail "bun availability" "bun is required for designIntake execution smoke test"
fi

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
