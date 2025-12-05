# Cipher Agent Completion Regression - Fix

## Problem Summary

Cipher (Codex) agent was failing to complete tasks autonomously, instead asking questions and retrying 10 times without declaring success.

## Root Causes

### 1. Broken Session ID Extraction (PRIMARY)

**Issue:** The session extraction regex didn't match Codex's actual output format.

**Expected Pattern:**
```
codex session [0-9a-fA-F-]+
```

**Actual Codex Output:**
```
session id: 019a8011-ad91-7ac0-89e3-75d4e923973d
```

**Result:** 
- Session ID never extracted
- `CURRENT_SESSION_ID` remained empty
- Completion probe skipped with warning: `⚠️ No Codex session found - skipping completion probe`
- Agent never asked "Is the task complete?"
- Agent kept iterating and asking user questions

**Fix:** Updated `extract_session_id()` to handle both formats:
```bash
grep -Eoi '(codex session|session id:)[[:space:]]+[0-9a-fA-F-]{8,}' "$log_file" | \
  sed -E 's/(codex session|session id:)[[:space:]]+//i' | \
  tail -n1
```

### 2. Weak Autonomous Execution Guidance (SECONDARY)

**Issue:** Cipher system prompt didn't explicitly prohibit asking questions or handle external blockers.

**Result:**
- Agent asked: "Want me to retry code scanning once `gh` auth is fixed?"
- Agent asked: "Want me to add a PR comment summarizing Gitleaks findings?"
- Agent waited for user input instead of completing autonomously

**Fix:** Added explicit execution requirements to system prompt:

```markdown
## Execution Requirements

**CRITICAL: Execute autonomously without asking questions**

- **DO NOT ask permission** - implement security improvements immediately
- **DO NOT wait for user input** - make decisions and execute
- **DO NOT end messages with questions** - state actions and proceed
- **If blocked** (e.g., GitHub auth unavailable):
  1. Complete ALL work you can do locally
  2. Document the blocker clearly
  3. Provide exact commands to resolve
  4. **STILL DECLARE TASK COMPLETE** if core security work done

**GitHub Code Scanning blockers DO NOT prevent completion**
```

## Verification

### Test Case
```bash
# Input log with "session id:" format
echo "session id: 019a8011-ad91-7ac0-89e3-75d4e923973d" > /tmp/test.log

# Extraction should succeed
result=$(grep -Eoi '(codex session|session id:)[[:space:]]+[0-9a-fA-F-]{8,}' /tmp/test.log | \
  sed -E 's/(codex session|session id:)[[:space:]]+//i' | tail -n1)

# Result: 019a8011-ad91-7ac0-89e3-75d4e923973d ✅
```

## Impact

**Before Fix:**
- Cipher ran 10 attempts without completing
- Asked user questions at end of each attempt
- Created handoff document after max retries
- Wasted compute and LLM tokens

**After Fix:**
- Session ID correctly extracted
- Completion probe runs after each attempt
- Agent declares success when criteria met
- No unnecessary questions or retries

## Files Modified

1. `infra/charts/controller/agent-templates/code/codex/container-base.sh.hbs`
   - Fixed `extract_session_id()` function to handle "session id:" format
   
2. `infra/charts/controller/agent-templates/agents/cipher-system-prompt.md.hbs`
   - Added "Execution Requirements" section
   - Explicit "DO NOT ask questions" guidance
   - Clear completion criteria that don't require GitHub auth
   - Instructions to complete despite external blockers

## Testing Recommendations

1. Trigger Cipher on a test PR with:
   - Valid security issues (e.g., outdated dependencies)
   - Verify agent fixes issues without asking questions
   - Verify completion probe triggers and agent declares "yes"

2. Test with blocked GitHub auth:
   - Verify agent completes local scans
   - Verify agent documents blocker but still completes
   - Verify no "Want me to..." questions in output

## Related Issues

- Similar session extraction patterns may exist in other CLI types (cursor, factory, opencode)
- Should audit all `extract_session_id()` implementations for consistency
- Consider adding session ID validation to prevent silent failures

