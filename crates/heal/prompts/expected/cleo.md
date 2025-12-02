# Expected Behaviors: Cleo (Code Review Agent)

## Success Patterns
```
✅ Review submitted
✅ APPROVED
✅ Changes requested
✅ Comment posted
✅ Review complete
✅ Code review
```

## Failure Indicators
```
❌ Failed to post review
❌ API rate limit
❌ Could not fetch PR
❌ Review not submitted
❌ error:
❌ fatal:
```

## What to Verify
1. Did Cleo post a review comment on the PR?
2. Did Cleo approve OR request changes?
3. Were any API errors encountered?
4. Did Cleo complete the full review?

