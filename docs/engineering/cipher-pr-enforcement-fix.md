# Fix: Cipher Task 6 PR Enforcement Issue

## Problem

Cipher (security agent) completed Task 6 successfully with:
- ✅ 30 commits made with security improvements
- ✅ All local validations passing (lint, build, audit, gitleaks)
- ✅ Gitleaks CI workflow added
- ✅ ESLint security hardening applied
- ✅ Zero vulnerabilities found

However, the task was marked as incomplete and retried 10 times because the PR enforcement logic only checked for agents matching the pattern `[Rr]ex`, causing Cipher to skip PR creation entirely.

## Root Cause

In the Codex/Factory/Cursor/OpenCode container templates, the PR enforcement logic at the end of execution used:

```bash
elif [[ "{{github_app}}" =~ [Rr]ex ]] && [ "$WORKFLOW_STAGE" = "implementation" ]; then
  echo "🔍 Rex implementation agent - creating PR even though completion unconfirmed"
  ensure_pr_created || echo "⚠️ PR creation failed or no PR was created"
else
  echo "⚠️ Skipping auto PR enforcement due to Codex completion status"
fi
```

This hardcoded check meant that:
- **Rex agents**: PR creation enforced
- **All other agents** (Cipher, Blaze, Atlas, etc.): PR creation skipped

## Solution

Changed the condition to use the already-calculated `COMMITS_MADE` variable instead of checking agent names:

```bash
elif [ "$COMMITS_MADE" -eq 1 ] && [ "$WORKFLOW_STAGE" = "implementation" ]; then
  echo "🔍 Implementation agent ($AGENT_NAME) made commits - ensuring PR exists"
  ensure_pr_created || echo "⚠️ PR creation failed or no PR was created"
else
  echo "⚠️ Skipping auto PR enforcement (no commits or non-implementation stage)"
fi
```

This ensures that **any implementation agent** that makes commits will trigger PR enforcement, not just Rex.

## Files Changed

- `infra/charts/controller/agent-templates/code/codex/container-base.sh.hbs`
- `infra/charts/controller/agent-templates/code/factory/container-base.sh.hbs`
- `infra/charts/controller/agent-templates/code/cursor/container-base.sh.hbs`
- `infra/charts/controller/agent-templates/code/opencode/container-base.sh.hbs`

## Impact

- ✅ Cipher and other security agents will now properly create PRs when they make commits
- ✅ Handoff logic continues to work correctly (based on commit count)
- ✅ No impact on Rex agents (behavior unchanged)
- ✅ Consistent behavior across all CLI agent types

## Testing

After deployment:
1. Trigger a Cipher task with security work
2. Verify PR is created when commits are made
3. Confirm handoff to Cleo happens successfully
4. Check that task doesn't retry unnecessarily

## Related

- Task 6 log: `task-6-cipher-codex-agent-platform-cto-par-a02f65fe-fc951a6x2tc.log`
- Issue: Cipher completed all security work but couldn't create PR due to hardcoded Rex check

