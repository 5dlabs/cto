# Per-Level Integration Tasks for Parallel Execution

**Status:** ✅ Complete  
**Date:** 2025-10-30  
**Branch:** `fix/rex-blaze-workspace-isolation`

---

## Problem Statement

When running parallel task execution, multiple tasks in the same execution level can complete independently but may not integrate correctly together. Without validation between levels, integration issues propagate and compound.

**Example Issue:**
```
Level 0: Tasks 1, 2, 3 run in parallel
- Task 1: Build auth system (PR #101)
- Task 2: Build payment system (PR #102)  
- Task 3: Build admin panel (PR #103)

All PRs merge individually ✅
But... admin panel can't access auth APIs ❌
```

## Solution: Per-Level Integration Tasks

Morgan (docs agent) now automatically creates **integration tasks after each execution level with 2+ parallel tasks**.

### Architecture

```
Level 0: [Task 1, 2, 3] (parallel tasks)
  ↓ (all complete and merged)
Task 4: Integration - Level 0
  - Validates tasks 1, 2, 3 work together
  - Runs full test suite on integrated code
  - Checks for conflicts
  - Creates integration report
  ↓ (integration validated)
Level 1: [Task 5] (depends on Task 4, not [1,2,3])
  - Starts with validated foundation
```

## Implementation Details

### 1. Integration Task Templates

**Location:** `agent-templates/docs/templates/integration-task/`

**Files:**
- `task.txt` - Comprehensive integration instructions  
- `task.md` - Formatted task documentation
- `prompt.md` - Step-by-step validation guide
- `acceptance-criteria.md` - Integration checklist
- `task.xml` - Structured metadata
- `README.md` - Template documentation

**Placeholders (Morgan replaces):**
- `LEVEL_INDEX` - Execution level number (0, 1, 2...)
- `TASK_LIST_PLACEHOLDER` - Comma-separated task IDs
- `INTEGRATION_TASK_ID` - ID for this integration task
- `ALL_DEPENDENCY_TASK_IDS` - Tasks this depends on

### 2. Morgan's Enhanced Logic

**Updated:** `agent-templates/docs/claude/prompt.md.hbs` Step 4

**Process:**

1. **Analyze Dependency Graph**
   - Read `tasks.json`
   - Build execution levels from dependencies
   - Level 0 = no deps, Level 1 = depends on L0, etc.

2. **Identify Levels Needing Integration**
   - **Rule:** If level has 2+ tasks → create integration task
   - **Single task levels:** Skip (nothing to integrate)

3. **Create Integration Tasks**
   - For each level with 2+ tasks:
     - Copy templates to `.taskmaster/docs/task-{id}/`
     - Replace placeholders with level-specific info
     - Add to `tasks.json` with dependencies on ALL tasks in level
     - Set `agentHint: "integration"` for routing to Tess

4. **Update Dependencies**
   - Tasks in NEXT level now depend on integration task
   - Example: Task 4 depended on [1,2,3] → now depends on [integration_task]

5. **Create Final Integration Task**
   - After all levels
   - Depends on ALL tasks (including level integration tasks)
   - End-to-end system validation

### 3. Integration Task Content

**What the agent does:**

```bash
# 1. Verify all PRs merged
git checkout main
git pull origin main

# 2. Run full test suite
npm test  # or cargo test, pytest, etc.

# 3. Check for integration issues
- Merge conflicts
- Runtime errors
- API contract mismatches
- Database migration conflicts
- Configuration incompatibilities

# 4. Validate build
npm run build

# 5. Create integration report
- Document what was tested
- List any issues found  
- Show resolutions
- Confirm readiness for next level
```

## Example Workflow

### Input: TaskMaster tasks.json
```json
[
  {"id": 1, "title": "Auth System", "dependencies": []},
  {"id": 2, "title": "Payment System", "dependencies": []},
  {"id": 3, "title": "Admin Panel", "dependencies": []},
  {"id": 4, "title": "Deploy", "dependencies": [1, 2, 3]}
]
```

### Morgan's Analysis
```
Level 0: [1, 2, 3] ← 3 parallel tasks, CREATE integration
Level 1: [4] ← 1 task, SKIP integration
```

### Morgan Generates
```json
[
  {"id": 1, "title": "Auth System", "dependencies": []},
  {"id": 2, "title": "Payment System", "dependencies": []},
  {"id": 3, "title": "Admin Panel", "dependencies": []},
  {"id": 4, "title": "Integration - Level 0", "dependencies": [1,2,3], "agentHint": "integration"},
  {"id": 5, "title": "Deploy", "dependencies": [4]},  ← Updated from [1,2,3]
  {"id": 6, "title": "Final Integration", "dependencies": [1,2,3,4,5], "agentHint": "integration"}
]
```

### Execution Flow
```
1. Tasks 1, 2, 3 run in parallel (Rex, Cleo, Tess for each)
2. All merge to main independently
3. Task 4 (Integration) starts:
   - Tess validates tasks 1,2,3 work together
   - Runs full test suite
   - Creates integration report
4. Task 4 completes → validates foundation
5. Task 5 (Deploy) starts with validated code
6. Task 6 (Final Integration) validates entire system
```

## Benefits

### For Parallel Execution
- ✅ **Validates integration** between levels
- ✅ **Prevents issue propagation** - catches problems early
- ✅ **Clear gates** - next level can't start until integration validated
- ✅ **Automated validation** - runs full test suite automatically

### For Development Speed  
- ✅ **No manual integration work** - Tess handles it
- ✅ **Issues caught early** - before next level starts
- ✅ **Clear status** - integration task shows system health
- ✅ **Maintains parallel speedup** - integration is quick (30-60 mins)

### For Code Quality
- ✅ **Enforces integration testing** - not just unit tests
- ✅ **Validates compatibility** - components work together
- ✅ **Documents integration** - creates validation reports
- ✅ **Prevents technical debt** - issues fixed immediately

## Agent Assignment

**Primary:** Tess (QA Agent)  
**Why:** Integration validation requires comprehensive testing capabilities

**Alternate:** Morgan (Docs Agent)  
**Why:** Can validate and document integration if Tess unavailable

**Routing:** `agentHint: "integration"` field ensures proper agent selection

## Configuration

**No configuration needed!** Integration tasks are auto-generated by Morgan during docs generation.

**Morgan automatically:**
1. Analyzes dependency graph
2. Identifies execution levels
3. Creates integration tasks where needed
4. Updates dependencies appropriately

## Testing

When Morgan runs next intake or docs job:

1. It will analyze the dependency graph
2. Create integration tasks for levels with 2+ tasks  
3. Generate all template files with customized content
4. Update tasks.json with proper dependencies

**Next Intake/Docs Run:**
- Integration task templates will be available in ConfigMap
- Morgan will use them automatically
- No manual intervention needed

## Files Changed

**Templates:**
- Created: `docs/templates/integration-task/*.md` (6 files)

**Morgan Prompt:**
- Updated: `docs/claude/prompt.md.hbs` Step 4

**ConfigMaps Regenerated:**
- `agent-templates-docs.yaml` now includes integration templates
- `agent-templates-claude.yaml` includes updated Morgan prompt

## Example Integration Task

```markdown
# Task 4: Integration - Level 0

## Tasks Being Integrated
- Task 1: Auth System
- Task 2: Payment System
- Task 3: Admin Panel

## Your Mission
1. Verify all PRs merged to main
2. Pull integrated code
3. Run full test suite
4. Check for integration issues
5. Validate build succeeds
6. Create integration report

## Success Criteria
✅ All level 0 PRs merged
✅ Full test suite passes
✅ Build succeeds
✅ No integration conflicts
✅ Ready for next level
```

## Next Steps

When you run the next intake or docs workflow:
1. Morgan will use the new integration logic
2. Integration tasks will be created automatically
3. Parallel execution will have validation gates
4. Integration will be explicitly validated, not assumed

**No manual work required** - the system handles it automatically! 🎉

