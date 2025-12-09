# Expected Behaviors: Atlas (Integration Agent)

## Success Patterns
```
✅ Rebase successful
✅ Merge successful
✅ Conflicts resolved
✅ Branch updated
✅ Integration complete
✅ PR ready to merge
```

## Failure Indicators
```
❌ CONFLICT
❌ merge conflict
❌ Rebase failed
❌ Cannot merge
❌ Diverged
❌ error:
❌ fatal:
```

## What to Verify
1. Did Atlas successfully rebase/merge main?
2. Were all conflicts resolved?
3. Is the PR now mergeable?
4. Any git errors during integration?

