# Systemic Agent Workflow Issues - Deep Analysis & Remediation Plan

**Date:** 2025-11-19  
**Scope:** Complete agent workflow system (Rex → Cleo → Cipher → Tess → Atlas)  
**Status:** Critical systemic issues identified requiring comprehensive fix

---

## Executive Summary

Through extensive testing and log analysis, I've identified **4 critical systemic issues** that prevent the multi-agent workflow from functioning correctly. These issues affect:
- ❌ Atlas PR monitoring (completely broken - no pods running)
- ❌ Tess QA validation (containers hang and never exit)
- ❌ Agent handoffs (unbound variables cause crashes)
- ❌ PR review posting (duplicate approvals from retry loops)

## Issue 1: Atlas PR Guardian Sensor - COMPLETELY BROKEN 🚨

### Evidence
```bash
$ kubectl get pods -n agent-platform -l agent=atlas
No resources found

$ kubectl logs -n argo -l sensor-name=atlas-pr-guardian --tail=50
Event discarded due to filtering error: expr filter error 
(Unable to access unexported field 'sender' in token 'body.sender.login')
```

### Root Cause
The Atlas PR guardian sensor has an expr filter at line 49:
```yaml
exprs:
  - expr: "!(body.sender.login contains 'atlas-5dlabs')"
    fields:
      - name: body.sender.login
        path: body.sender.login
```

**Problem**: Argo Events expr evaluation engine cannot access `body.sender.login` with the dot notation in the `fields` section. This is a known Argo Events limitation with nested field access in expr filters.

### Impact
- **100% of events are discarded** - Atlas never triggers
- No PR monitoring happens
- No conflict resolution occurs
- No automatic PR merging happens
- Atlas design is completely non-functional

### Solution
Remove the broken expr filter and handle Atlas self-filtering in the workflow logic instead:
```yaml
# Remove the exprs section entirely
# Add this check in the Atlas container script instead:
if [[ "${SENDER_LOGIN:-}" == *"atlas-5dlabs"* ]]; then
  echo "Event from Atlas itself, skipping to prevent infinite loop"
  exit 0
fi
```

---

## Issue 2: Tess Containers Hang After Claude Completes 🚨

### Evidence
From task-6 Tess log (line 417):
```
{"type":"result","subtype":"success",...}  ← Claude finishes
[LOG ENDS HERE - No cleanup, no exit, container hangs forever]
```

Expected but missing:
```
Line 2063: ║  IMPLEMENTATION TASK COMPLETE  ║
Line 2086: Claude has completed successfully
Line 2515: exit 0
```

### Root Cause Analysis

**Problem 1: Wrong Template Used**

Controller template mapping (templates.rs lines 2570, 2609, 2657, 2737, 2791):
```rust
// ALL CLIs map Tess to integration template:
"5DLabs-Tess" => "code/integration/container-tess.sh.hbs"  // 2515 lines!

// But CLI-specific templates exist and are ignored:
"code/cursor/container-tess.sh.hbs"   // 4 lines, uses partial
"code/factory/container-tess.sh.hbs"  // 4 lines, uses partial
```

The `integration/container-tess.sh.hbs` template:
- Designed for implementation agents (Rex/Blaze)
- Has FIFO cleanup logic (lines 2032-2053)
- Has hook script execution (lines 2068-2084)
- Has complex git logic
- **Never reaches exit code** when used with modern CLIs

**Problem 2: FIFO Cleanup Hangs**

Lines 2032-2053 try to clean up FIFOs that may not exist or may be held open by other processes. If the FIFO cleanup fails or hangs, the script never reaches the exit code.

**Problem 3: Conditional Logic Maze**

The template has deeply nested if/else blocks that can lead to paths where exit is never reached.

### Solution
1. **Fix template mapping** in controller to use CLI-specific templates
2. **Simplify integration/container-tess.sh.hbs** to not hang
3. **Add timeout guards** to prevent infinite hangs

---

## Issue 3: Agent Handoff Failures (Cipher → Tess) 🚨

### Evidence
From task-4 Cipher log (line 1301):
```
Line 1300: ✅ Cursor confirmed task completion
Line 1301: /task-files/container.sh: line 2122: COMMITS_MADE: unbound variable
[Script crashes, handoff never happens]
```

### Root Cause
Variable `COMMITS_MADE` only initialized in failure path:
```bash
if [ $SUCCESS -ne 1 ]; then
  COMMITS_MADE=0  # ← Only set when SUCCESS != 1
fi
# ...later...
if [ "$COMMITS_MADE" -eq 1 ]; then  # ← Crashes if SUCCESS=1!
```

When agent succeeds on iteration 1, `SUCCESS=1`, so `COMMITS_MADE` is never set, causing unbound variable error in bash strict mode.

### Impact
- Cipher completes successfully
- Bash crashes before PR review logic runs
- `ready-for-qa` label never added
- Tess never triggers
- Workflow stalls

### Solution (Already Fixed in PR #1514)
```bash
# Initialize at top level:
COMMITS_MADE=0
ATTEMPT=1
SUCCESS=0
```

---

## Issue 4: Duplicate Approvals from Retry Loops 🚨

### Evidence
PR #11 shows Cipher approved 3 times with identical content.

### Root Cause
PR review posting happens **inside the retry loop**:
```bash
while [ $ATTEMPT -le $MAX_RETRIES ]; do
  # Claude runs...
  if [ $SECURITY_PASSED -eq 1 ]; then
    gh pr review --approve  # ← Posted EVERY iteration!
    break  # Too late!
  fi
done
```

### Impact
- Cluttered PR history
- Confusing for users
- Waste of GitHub API calls
- Makes it unclear if agent actually succeeded

### Solution (Already Fixed in PR #1514)
Move review posting **outside the loop**:
```bash
while [ $ATTEMPT -le $MAX_RETRIES ]; do
  # Just set flags, don't post
done
# Post review ONCE after loop completes
if [ $SUCCESS -eq 1 ]; then
  gh pr review --approve
fi
```

---

## Issue 5: Cipher Closing PRs Prematurely 🚨

### Evidence
PR #11 was closed by cipher-5dlabs bot even though Cipher said everything passed.

### Root Cause
Cipher system prompt said "STILL DECLARE TASK COMPLETE" which Claude misinterpreted as "close the PR/issue".

### Solution (Already Fixed in PR #1514)
Added explicit restrictions to Cipher prompt:
```markdown
## CRITICAL: PR Management Restrictions
- ❌ DO NOT run `gh pr close` 
- ❌ DO NOT run `gh pr merge`
- ✅ DO post PR reviews
```

---

## Issue 6: Cleo Duplicate Comments 🚨

### Evidence
Cleo posted the same "Quality Review - PASSED" comment twice on multiple PRs.

### Root Cause
When converting Cleo's approval to comment, I left the original comment posting section AND added a new one, causing duplicates.

### Solution (Already Fixed in PR #1511)
Removed duplicate comment block in Claude Cleo template.

---

## Remediation Plan

### Phase 1: Critical Fixes (Blocks All Workflows) ✅ COMPLETE
- ✅ PR #1509: Cipher posts APPROVE reviews (not just comments)
- ✅ PR #1510: CLAUDE_LOG undefined, Cleo approval fixes
- ✅ PR #1511: Cleo duplicate comments removed
- ✅ PR #1514: COMMITS_MADE initialization, reviews outside loops, Cipher PR restrictions

### Phase 2: Atlas Sensor Fix (HIGH PRIORITY) 🔨 IN PROGRESS
**Issue**: Atlas sensor expr filter broken, 100% events discarded
**Fix**: Remove broken expr filter, add sender check in workflow

**Files to modify**:
1. `infra/gitops/resources/sensors/atlas-pr-guardian-sensor.yaml`
   - Remove exprs filter section (lines 45-54)
   - Add sender check as first step in workflow script

**Testing**:
- Verify events no longer discarded in sensor logs
- Verify Atlas CodeRun created when PR opened
- Verify Atlas pod runs and monitors PR

### Phase 3: Tess Container Exit Fix (HIGH PRIORITY) 🔨 IN PROGRESS
**Issue**: Tess containers hang after Claude completes, never exit
**Fix**: Simplify Tess template to ensure clean exit

**Files to modify**:
1. `controller/src/tasks/code/templates.rs`
   - Update get_claude_container_template
   - Update get_cursor_container_template  
   - Update get_factory_container_template
   - Update get_opencode_container_template
   - Update get_codex_container_template
   - Map Tess to CLI-specific templates (use partials)

2. `infra/charts/controller/agent-templates/code/integration/container-tess.sh.hbs`
   - Simplify FIFO cleanup (lines 2032-2053)
   - Add timeout guards
   - Ensure exit 0 is always reached

**Testing**:
- Run Tess on test PR
- Verify container exits cleanly after Claude completes
- Verify PR approval review is posted
- Verify workflow resumes to next stage

### Phase 4: Batch Integration Verification (MEDIUM PRIORITY) 🔍 NEEDS VERIFICATION
**Issue**: Unknown if batch completion triggers Atlas integration
**Fix**: Verify sensor configuration and workflow completion logic

**Files to check**:
1. `infra/gitops/resources/sensors/atlas-batch-integration-sensor.yaml`
2. Workflow templates for batch completion comments

**Testing**:
- Complete a parallel batch test
- Verify "Batch N Complete" comment is posted
- Verify Atlas integration sensor triggers
- Verify Atlas merges PRs in dependency order

---

## Implementation Approach

### Step 1: Fix Atlas Sensor (Critical Blocker)
```yaml
# Remove this broken section:
exprs:
  - expr: "!(body.sender.login contains 'atlas-5dlabs')"
```

Add to Atlas workflow:
```bash
SENDER_LOGIN="{{workflow.parameters.sender-login}}"
if [[ "$SENDER_LOGIN" == *"atlas-5dlabs"* ]]; then
  exit 0
fi
```

### Step 2: Fix Tess Template Mapping
Update controller code to use CLI-specific templates:
```rust
fn get_claude_container_template(code_run: &CodeRun) -> String {
    match github_app {
        "5DLabs-Tess" => "claude/agents-tess.md.hbs", // Use agent memory, not container
        // OR create claude/container-tess.sh.hbs that uses shared partial
    }
}
```

### Step 3: Create Proper Tess Container Templates
For each CLI, ensure Tess uses the base container with proper stage handling:
- Cursor: Uses `cursor_container_base` partial ✅
- Factory: Uses `factory_container_base` partial ✅  
- OpenCode: Uses `opencode_container_base` partial ✅
- Codex: Uses `codex_container_base` partial ✅
- Claude: Missing - needs to be created! ❌

### Step 4: Simplify Integration Template
If integration template must be used, fix the hang:
- Remove complex FIFO cleanup
- Add explicit exit paths
- Add timeout guards

---

## Expected Outcomes After Fix

1. ✅ **Atlas Monitoring**: PR opened → Atlas pod spawns → Monitors continuously → Resolves conflicts
2. ✅ **Tess Completion**: Tess validates → Posts approval → Container exits cleanly → Workflow resumes
3. ✅ **Clean Handoffs**: Cipher → Tess → Atlas transitions work smoothly
4. ✅ **No Hangs**: All agents exit cleanly after completion
5. ✅ **Batch Integration**: Atlas merges all PRs after batch completion

---

## Verification Plan

After fixes deployed:
1. Start new play workflow with parallel execution
2. Verify Atlas pod appears when PRs created
3. Verify Tess completes and exits (check pod status)
4. Verify workflow progresses through all stages
5. Verify batch completion triggers Atlas integration
6. Verify all PRs merge cleanly

---

## Risk Assessment

**High Risk**:
- Atlas sensor fix affects PR monitoring for cto repo
- Incorrect fix could cause Atlas to run in infinite loops
- Tess template changes affect all CLIs

**Mitigation**:
- Test on cto-parallel-test first (not production cto repo)
- Add defensive checks in workflow scripts
- Keep old templates as backup
- Deploy incrementally and verify each fix

---

This analysis forms the basis for the comprehensive remediation PR.


