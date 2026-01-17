# Objective: Full Play Workflow (Task 1)

Execute the complete play workflow for Task 1, ensuring:
1. **Correct agent** executes based on `agent_hint` from tasks.json
2. **Complete quality sequence** runs: Implementation → Cleo → Cipher → Tess → Atlas

**Retry Configuration:** Each agent gets up to 10 attempts to meet acceptance criteria.

## FIX BUGS, DON'T ACCEPT THEM

If you encounter any bugs or issues:
1. **Investigate** - Find the root cause using logs, code inspection, and debugging
2. **Fix** - Make the necessary code changes to resolve the issue
3. **Verify** - Confirm the fix works before proceeding
4. **Never accept bugs** - Don't just document and move on; always fix first

### Common Issues to Watch For:
- **Wrong agent assigned**: Check if `agent_hint` from tasks.json is being used (fix in MCP or controller)
- **Quality sequence not running**: Check Argo workflow submission and parameters
- **Missing CodeRuns**: Verify the Argo workflow is creating all required stages

## Phase Gate Reference

Use `lifecycle-test/phase-gates.json` for all gate conditions. Each phase must pass ALL gates before proceeding to the next.

## Pre-Conditions (verify first)

```bash
# 1. tasks.json exists with correct structure
gh api repos/5dlabs/prd-alerthub-e2e-test/contents/.tasks/tasks/tasks.json --silent

# 2. Check expected agent for Task 1
gh api repos/5dlabs/prd-alerthub-e2e-test/contents/.tasks/tasks/tasks.json --jq '.content' | base64 -d | jq -r '.tasks[0].agent_hint'
```

## Gates (in sequence)

### 1. Implementation Phase
- `play-coderun-created`: CodeRun exists for implementation
- `correct-agent-assigned`: CodeRun agent matches task's `agent_hint`
- `implementation-succeeded`: CodeRun completed successfully  
- `pr-created`: PR exists in repository
- `pr-author-correct`: PR author matches expected agent (e.g., `app/bolt-5dlabs` for bolt)

**Verify PR author with:**
```bash
gh pr list --repo 5dlabs/prd-alerthub-e2e-test --state all --json author,number --jq '.[0]'
```

### 2. Quality Sequence (must all run in order)
- `cleo-coderun-created` + `cleo-succeeded`: Cleo quality review
- `cipher-coderun-created` + `cipher-succeeded`: Cipher security analysis
- `tess-coderun-created` + `tess-succeeded`: Tess testing
- `atlas-coderun-created` + `atlas-succeeded` + `pr-merged`: Atlas merge

## Failure Protocol

**If ANY gate fails:**
1. Document the failure in `progress.txt`
2. Run cleanup commands from `phase-gates.json`
3. Investigate root cause
4. Fix the issue (code, config, or infrastructure)
5. Restart from the failed phase (NOT from where you left off)

## Evidence Required

- Record all gate check outputs in `report.json`
- For `correct-agent-assigned` failure: Show expected vs actual agent
- For `pr-author-correct` failure: Show `gh pr` output with author info
- For missing quality sequence: List which phases were skipped

## Success Criteria

All phases complete in sequence:
1. Implementation by correct agent (based on `agent_hint`)
2. Cleo review submitted
3. Cipher analysis completed
4. Tess testing passed
5. Atlas merged PR
