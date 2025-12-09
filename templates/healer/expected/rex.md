# Expected Behaviors: Rex (Implementation Agent)

## Success Patterns
```
✅ git push
✅ git commit
✅ PR created
✅ PR updated
✅ Branch .* pushed
✅ Changes committed
✅ Implementation complete
```

## Failure Indicators
```
❌ error:
❌ fatal:
❌ CONFLICT
❌ merge conflict
❌ failed to push
❌ Permission denied
❌ authentication failed
```

## What to Verify
1. Did Rex create/update the PR?
2. Were commits pushed to the feature branch?
3. Are there any git errors?
4. Did Rex complete without crashes?

