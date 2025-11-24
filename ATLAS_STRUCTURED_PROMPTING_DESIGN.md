# Atlas Guardian Structured Prompting Design

## Overview
Apply the proven probe/acceptance criteria pattern from Rex/Cleo/Tess to Atlas Guardian for consistent, effective AI-driven PR management.

## Pattern Structure

Each scenario creates a structured CLAUDE.md with:
1. **Scenario Type** - What needs fixing
2. **Probe/Context** - Full details (files, logs, comments)
3. **Acceptance Criteria** - Clear success definition
4. **Instructions** - How to fix
5. **Iteration Tracking** - Loop until complete

## Scenario 1: Merge Conflicts

### Probe Content:
```markdown
# 🔀 MERGE CONFLICT RESOLUTION

## Scenario
PR #{{PR_NUMBER}} has merge conflicts that must be resolved.

## Conflicted Files
{{LIST_OF_CONFLICTED_FILES}}

## Conflict Details
{{FOR_EACH_FILE}}
### File: `{{FILENAME}}`
```diff
{{GIT_DIFF_OUTPUT_WITH_MARKERS}}
```
{{END_FOR_EACH}}

## Instructions
1. Fetch latest main: `git fetch origin main`
2. Your branch is already checked out: `{{PR_BRANCH}}`
3. Rebase: `git rebase origin/main`
4. For each conflicted file:
   - Read both sides of the conflict (<<<<<<< ======= >>>>>>>)
   - Understand what each side is trying to do
   - Merge changes intelligently (keep functionality from both)
   - Remove conflict markers
   - Test that the code works
5. Complete rebase: `git add -A && git rebase --continue`
6. Push: `git push --force-with-lease`

## Acceptance Criteria
- ✅ All conflict markers removed from all files
- ✅ Code compiles/builds successfully
- ✅ Functionality from both branches preserved
- ✅ Tests pass (if applicable)
- ✅ PR mergeable status = MERGEABLE
- ✅ PR merge state = clean

## Current Status
- Iteration: {{ITERATION}}/10
- Conflicted files: {{FILE_COUNT}}
```

## Scenario 2: CI Failures

### Probe Content:
```markdown
# ⚠️ CI FAILURE REMEDIATION

## Scenario
PR #{{PR_NUMBER}} has failing CI checks that must be fixed.

## Failed Checks
{{FOR_EACH_FAILED_CHECK}}
- ❌ {{CHECK_NAME}} (Run: {{RUN_ID}})
{{END_FOR_EACH}}

## Full CI Logs
{{FOR_EACH_FAILED_RUN}}
### Check: {{CHECK_NAME}}
**Run ID**: {{RUN_ID}}
**URL**: {{RUN_URL}}

```
{{FULL_WORKFLOW_LOGS_FROM_GH_CLI}}
```
{{END_FOR_EACH}}

## Instructions
1. Review each failed check's logs above
2. Identify the root cause (build error, test failure, lint issue, etc.)
3. Make targeted fixes to address the specific failures
4. Run the checks locally if possible to verify
5. Commit and push fixes
6. Wait for CI to re-run and pass

## Acceptance Criteria
- ✅ All CI checks pass (SUCCESS or SKIPPED)
- ✅ No new failures introduced
- ✅ statusCheckRollup shows all green
- ✅ Fixes are minimal and targeted

## Current Status
- Iteration: {{ITERATION}}/10
- Failed checks: {{FAILED_COUNT}}
```

## Scenario 3: Quality Issues (Bugbot)

### Probe Content:
```markdown
# 🐛 QUALITY ISSUE RESOLUTION

## Scenario
PR #{{PR_NUMBER}} has quality feedback that must be addressed.

## Quality Feedback
{{FULL_BUGBOT_COMMENT_BODY}}

## Instructions
1. Read the quality feedback above carefully
2. Address all 🔴 critical errors (must fix)
3. Address all 🟡 warnings (should fix)
4. Consider 💡 suggestions (optional)
5. Make minimal, targeted changes
6. Test that fixes work
7. Commit and push

## Acceptance Criteria
- ✅ All 🔴 errors resolved
- ✅ All 🟡 warnings resolved
- ✅ No new quality issues introduced
- ✅ Existing functionality preserved
- ✅ Tests pass

## Current Status
- Iteration: {{ITERATION}}/10
- Errors: {{ERROR_COUNT}}
- Warnings: {{WARNING_COUNT}}
```

## Implementation Strategy

### Atlas Container Script Flow

```bash
# Main monitoring loop
while PR is open:
  ITERATION=0
  MAX_ITERATIONS=10
  
  while ITERATION < MAX_ITERATIONS:
    # 1. Probe: Detect what needs fixing
    MERGE_CONFLICTS=$(check_for_conflicts)
    CI_FAILURES=$(check_ci_status)
    QUALITY_ISSUES=$(check_bugbot_comments)
    
    # 2. If nothing to fix, check if ready to merge
    if all scenarios clean:
      → AUTO-MERGE
      → EXIT
    
    # 3. Generate CLAUDE.md with combined scenarios
    cat > /workspace/CLAUDE.md << EOF
# 🔱 ATLAS GUARDIAN - PR Quality Gate

**PR**: #{{PR_NUMBER}}
**Iteration**: {{ITERATION}}/10
**Branch**: {{PR_BRANCH}}

---

{{IF MERGE_CONFLICTS}}
[Include Scenario 1 content with files and diffs]
{{END_IF}}

{{IF CI_FAILURES}}
[Include Scenario 2 content with full logs]
{{END_IF}}

{{IF QUALITY_ISSUES}}
[Include Scenario 3 content with full feedback]
{{END_IF}}

---

## 🎯 OVERALL ACCEPTANCE CRITERIA

Your work is complete when ALL of these are true:
- ✅ No merge conflicts (mergeable = MERGEABLE, state = clean)
- ✅ All CI checks passing
- ✅ No unresolved quality issues (no 🔴 or 🟡 in comments)

Fix the issues above, commit, and push. Atlas will verify on the next cycle.
EOF
    
    # 4. Run CLI with structured prompt
    run_cli_with_prompt /workspace/CLAUDE.md
    
    # 5. Wait for changes to take effect
    sleep 30  # Let CI run, GitHub recalculate status
    
    # 6. Increment iteration
    ITERATION++
  done
  
  # Max iterations reached
  → Post escalation comment
  → EXIT with error
done
```

## Key Improvements

### 1. Full Context
- **Merge Conflicts**: File list + full diffs with markers
- **CI Failures**: Complete workflow logs via `gh run view --log`
- **Quality Issues**: Full comment body (already implemented)

### 2. Clear Acceptance Criteria
- Each scenario has specific, measurable success criteria
- Combined overall criteria at the end
- Easy for AI to verify completion

### 3. Iteration Pattern
- Tracks iteration count (1-10)
- Re-probes on each cycle
- Stops when all criteria met or max iterations reached
- Escalates if can't complete

### 4. Combined Scenarios
- Single CLAUDE.md can contain multiple active scenarios
- AI fixes everything in one session
- More efficient than separate workflows per scenario

## Technical Details

### Fetching CI Logs
```bash
# Get failed workflow runs for the PR
FAILED_RUNS=$(gh api "/repos/$OWNER/$REPO/actions/runs?event=pull_request&branch=$PR_BRANCH&status=failure" \
  --jq '.workflow_runs[0:3] | .[] | "\(.id) \(.name)"')

for run_id in $FAILED_RUNS; do
  # Get full logs
  gh run view $run_id --log > /tmp/ci-failure-$run_id.log
  
  # Include in prompt
  cat /tmp/ci-failure-$run_id.log >> /workspace/CLAUDE.md
done
```

### Getting Conflict Files
```bash
# After rebase fails
git fetch origin main
git rebase origin/main 2>&1 || true

# Get list of conflicted files
CONFLICT_FILES=$(git diff --name-only --diff-filter=U)

# Get full diff with markers for each
for file in $CONFLICT_FILES; do
  echo "## Conflict in: $file" >> /workspace/CLAUDE.md
  echo '```diff' >> /workspace/CLAUDE.md
  git diff "$file" >> /workspace/CLAUDE.md
  echo '```' >> /workspace/CLAUDE.md
done
```

### Checking All Scenarios
```bash
check_all_scenarios() {
  # 1. Conflicts
  MERGEABLE=$(gh pr view $PR_NUMBER --json mergeable --jq '.mergeable')
  MERGE_STATE=$(gh pr view $PR_NUMBER --json mergeStateStatus --jq '.mergeStateStatus')
  HAS_CONFLICTS=false
  if [ "$MERGEABLE" = "CONFLICTING" ] || [ "$MERGE_STATE" = "DIRTY" ]; then
    HAS_CONFLICTS=true
  fi
  
  # 2. CI
  FAILED_CI=$(gh pr view $PR_NUMBER --json statusCheckRollup \
    --jq '[.statusCheckRollup[]? | select(.conclusion == "FAILURE")] | length')
  HAS_CI_FAILURES=false
  if [ "$FAILED_CI" -gt 0 ]; then
    HAS_CI_FAILURES=true
  fi
  
  # 3. Quality
  UNRESOLVED=$(gh api "/repos/$OWNER/$REPO/issues/$PR_NUMBER/comments" \
    --jq '[.[] | select(.body | test("🔴|🟡"))] | length')
  HAS_QUALITY_ISSUES=false
  if [ "$UNRESOLVED" -gt 0 ]; then
    HAS_QUALITY_ISSUES=true
  fi
  
  # Return true if ALL scenarios are clean
  if [ "$HAS_CONFLICTS" = "false" ] && \
     [ "$HAS_CI_FAILURES" = "false" ] && \
     [ "$HAS_QUALITY_ISSUES" = "false" ]; then
    return 0  # Ready to merge!
  else
    return 1  # Still have issues
  fi
}
```

## Benefits

1. **Consistency**: Same pattern as Rex/Cleo/Tess
2. **Better Context**: AI gets full information, not snippets
3. **Clear Goals**: Acceptance criteria are explicit and measurable
4. **Iteration Support**: Natural loop until completion
5. **Debugging**: CLAUDE.md file shows exactly what AI was asked to do
6. **Efficiency**: One AI session can fix multiple scenarios
7. **Escalation**: Clear failure path after max iterations

## Migration

- Old code: Ad-hoc inline prompts per scenario
- New code: Structured CLAUDE.md with all scenarios
- Backward compatible: Existing CodeRuns continue running
- Immediate benefit: Much better AI understanding and success rate
