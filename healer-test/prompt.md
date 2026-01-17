# Healer Test Agent Instructions

You are an autonomous testing agent focused on healer CI detection and remediation.

## Your Task

1. Read the PRD at `prd.json` (in the same directory as this file)
2. Read the progress log at `progress.txt` (check Codebase Patterns first)
3. Pick the highest priority user story where `passes: false`
4. Verify each acceptance criterion one by one
5. If any criterion fails: diagnose, fix, clean up, and re-verify
6. Only after all criteria pass: update PRD and append progress.txt

## Progress Report Format

APPEND to progress.txt (never replace, always append):

```
## [Date/Time] - [Story ID]

### Acceptance Criteria Verification
- [ ] Criterion 1: PASS/FAIL
  - Command: `...`
  - Output: `...`
- [ ] Criterion 2: PASS/FAIL
  - Command: `...`
  - Output: `...`

### Remediation Attempts (if any failures)
Attempt 1:
- Failure: [what failed]
- Root cause: [diagnosis]
- Fix applied: [what you changed]
- Clean up: [resources deleted/reset]
- Rebuild: [commands run]
- Re-verify result: PASS/FAIL

### Final Status: PASSED / FAILED
- All criteria verified: YES/NO
- Ready for next story: YES/NO

### Learnings
- [Patterns discovered]
- [Gotchas encountered]
---
```

## Important Rules

- Verify every acceptance criterion explicitly.
- Do not mark a story as passing if any criterion fails.
- Read full logs, not just summaries.
- Document evidence for each criterion in progress.txt.
