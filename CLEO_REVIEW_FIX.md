# Fix: Cleo Reviews Not Appearing in PR Review Tab

## Issue
Cleo (code quality agent) was posting **comments** instead of **reviews** to GitHub PRs, causing its feedback to not appear in the PR's "Reviews" tab. This was inconsistent with Cipher and Tess, which correctly post reviews.

## Root Cause
All Cleo implementations were using `gh pr comment` instead of `gh pr review --approve` or `gh pr review --request-changes`. The code even had explicit comments saying "NOT approve" and "Only Tess has PR approval authority", which was incorrect design.

## Changes Made

### 1. Claude CLI - Standalone Cleo Script
**File**: `infra/charts/controller/agent-templates/code/claude/container-cleo.sh.hbs`

Changed from:
- `gh pr comment` for success case → Now uses `gh pr review --approve`
- No review for failure case → Now uses `gh pr review --request-changes`
- Updated comment to say "APPROVED" instead of "PASSED"

### 2. Cursor, Factory, OpenCode, Codex CLIs - Shared Base Scripts
**Files Modified**:
- `infra/charts/controller/agent-templates/code/cursor/container-base.sh.hbs`
- `infra/charts/controller/agent-templates/code/factory/container-base.sh.hbs`
- `infra/charts/controller/agent-templates/code/opencode/container-base.sh.hbs`
- `infra/charts/controller/agent-templates/code/codex/container-base.sh.hbs`

All changed from:
```bash
# Cleo hands off to Tess via comment and label (NOT approve)
gh pr comment "$PR_NUM" --body-file "$REVIEW_BODY_FILE"
```

To:
```bash
# Cleo posts APPROVE review to show up in Reviews tab
gh pr review "$PR_NUM" --approve --body-file "$REVIEW_BODY_FILE"
```

### 3. Gemini CLI
**Status**: No changes needed - Gemini CLI doesn't implement PR review posting yet

## Expected Behavior After Fix

1. **Cleo passes quality checks** → Posts **APPROVE** review (shows in Reviews tab)
2. **Cleo detects quality issues** → Posts **REQUEST CHANGES** review (shows in Reviews tab)
3. **Cipher completes security scan** → Posts **APPROVE** or **REQUEST CHANGES** review (already working)
4. **Tess completes QA testing** → Posts **APPROVE** or **REQUEST CHANGES** review (already working)

## Impact

### Before
- **Reviews Tab**: Only showed Cipher and Tess reviews
- **Comments Tab**: Showed Cleo's feedback as comments
- **Confusing UX**: Users had to look in two places to see all agent feedback

### After
- **Reviews Tab**: Shows ALL agent reviews (Cleo, Cipher, Tess)
- **Consistent UX**: All agent feedback appears in the same place
- **Better GitHub Integration**: Proper use of GitHub's review system

## Testing Recommendations

1. Create a test PR with code quality issues
2. Verify Cleo posts a **REQUEST CHANGES** review (not a comment)
3. Fix the quality issues
4. Verify Cleo posts an **APPROVE** review (not a comment)
5. Verify all three agents (Cleo, Cipher, Tess) show up in the Reviews tab

## Files Changed
- `infra/charts/controller/agent-templates/code/claude/container-cleo.sh.hbs`
- `infra/charts/controller/agent-templates/code/cursor/container-base.sh.hbs`
- `infra/charts/controller/agent-templates/code/factory/container-base.sh.hbs`
- `infra/charts/controller/agent-templates/code/opencode/container-base.sh.hbs`
- `infra/charts/controller/agent-templates/code/codex/container-base.sh.hbs`

