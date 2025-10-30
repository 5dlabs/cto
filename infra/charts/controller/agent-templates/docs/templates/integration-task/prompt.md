# Integration Validation - Level LEVEL_INDEX

You are validating that all tasks from execution level LEVEL_INDEX integrate correctly.

## Your Mission

Ensure the following tasks work together as a cohesive system:

TASK_LIST_PLACEHOLDER

## Steps to Execute

### 1. Verify All PRs Merged
Check that all task PRs have been merged to main:
TASK_LIST_WITH_PRS_PLACEHOLDER

### 2. Pull Latest Main
```bash
git checkout main
git pull origin main
```

### 3. Run Full Test Suite
```bash
# Run all tests with integrated code
npm test  # or appropriate test command
cargo test  # for Rust
pytest  # for Python
```

### 4. Check for Integration Issues

**Look for:**
- Merge conflicts (shouldn't exist but verify)
- Runtime errors when components interact
- API contract mismatches
- Database migration conflicts
- Configuration incompatibilities

### 5. Validate Build
```bash
# Verify build succeeds with all changes
npm run build  # or appropriate build command
cargo build --release
```

### 6. Run Integration-Specific Tests

If integration tests exist, run them:
```bash
npm run test:integration
cargo test --test integration
```

### 7. Create Integration Report

Document your findings:
```markdown
## Integration Validation Report - Level LEVEL_INDEX

**Tasks Integrated:** TASK_LIST_PLACEHOLDER
**Date:** [Current Date]
**Agent:** Tess/Morgan

### Test Results
- [ ] Unit tests: PASS/FAIL
- [ ] Integration tests: PASS/FAIL  
- [ ] Build: PASS/FAIL
- [ ] No conflicts: PASS/FAIL

### Issues Found
[List any integration issues]

### Resolutions
[Document how issues were fixed]

### Status
✅ READY FOR NEXT LEVEL / ⚠️ ISSUES REQUIRE ATTENTION
```

## Success Criteria

Your task is complete when:

1. ✅ All level LEVEL_INDEX task PRs are merged
2. ✅ Full test suite passes
3. ✅ Build succeeds
4. ✅ No integration conflicts
5. ✅ Integration report created
6. ✅ Ready to proceed to next execution level

## If Issues Found

1. Document the issue clearly
2. Create fixes in a new PR if needed
3. Update affected tasks' documentation
4. Re-run validation after fixes

## Output

Create a comment or file with your integration validation results before marking this task complete.
