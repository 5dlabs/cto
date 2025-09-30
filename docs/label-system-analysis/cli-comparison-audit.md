# CLI Implementation Audit - Label System Comparison

## Executive Summary

**Codex/Cursor/Factory** - Use shared base scripts, have correlation labels ONLY
**Claude** - Monolithic scripts, has BOTH correlation AND status labels
**OpenCode** - Incomplete/stub implementation

---

## Detailed Comparison

### 1. Codex (36KB base script)

#### Correlation Labels ✅
```bash
TASK_LABEL="task-{{task_id}}"           # Line 344
SERVICE_LABEL="service-{{service}}"      # Line 345  
RUN_LABEL="run-{{workflow_name}}"        # Line 339
```

**Who adds**: Rex during `ensure_pr_created()` function
**When**: During PR creation (automatic fallback if agent doesn't create PR)
**Implementation**: Lines 235-417

#### Status Labels ❌
**None** - No status label logic in Codex

---

### 2. Cursor (31KB base script)

#### Correlation Labels ✅
```bash
TASK_LABEL="task-{{task_id}}"           # Line 320
SERVICE_LABEL="service-{{service}}"      # Line 321
RUN_LABEL="run-{{workflow_name}}"        # Line 315
```

**Implementation**: Identical to Codex, just different line numbers

#### Status Labels ❌
**None** - No status label logic in Cursor

---

### 3. Factory (29KB base script)

#### Correlation Labels ✅
```bash
TASK_LABEL="task-{{task_id}}"           # Line 324
SERVICE_LABEL="service-{{service}}"      # Line 325
RUN_LABEL="run-{{workflow_name}}"        # Line 319
```

**Implementation**: Identical to Codex/Cursor

#### Status Labels ❌
**None** - No status label logic in Factory

---

### 4. Claude (Monolithic: 1.3K-2.2K lines per agent)

#### Correlation Labels ✅

**Rex** (`container-rex.sh.hbs` - 1796 lines):
```bash
TASK_LABEL="task-${TASK_ID}"            # Line 1470, 1578, 1665
RUN_LABEL="run-${WORKFLOW_NAME}"         # Line 1666
SERVICE_LABEL="service-${SERVICE_NAME}"  # Line 1667
```

**Implementation**: Lines 1665-1731
- Creates labels if they don't exist
- Adds labels to existing PR via `gh pr edit --add-label`
- Has retry logic and error handling

**Tess** (`container-tess.sh.hbs` - 2167 lines):
- Uses existing labels, doesn't create them

**Cleo** (`container-cleo.sh.hbs` - 1310 lines):
- Uses existing labels, doesn't create them

#### Status Labels ✅

**All Claude agents define**:
```bash
STATUS_LABEL_NEEDS_FIXES="needs-fixes"              # Red (d73a4a)
STATUS_LABEL_FIXING="fixing-in-progress"            # Yellow (fbca04)
STATUS_LABEL_NEEDS_CLEO="needs-cleo"                # Green (0e8a16)
STATUS_LABEL_NEEDS_TESS="needs-tess"                # Purple (5319e7)
STATUS_LABEL_APPROVED="approved"                     # Dark Green (2da44e)
STATUS_LABEL_FAILED="failed-remediation"            # Dark Red (b60205)
STATUS_LABEL_NEEDS_TESTS_LEGACY="needs tests"       # Legacy label
```

**Who adds what**:

**Rex**:
- Adds `needs-cleo` after creating PR (line 1618)
- Uses `ensure_status_labels()` to create label definitions

**Tess** (`container-tess.sh.hbs`):
- Checks for `ready-for-qa` label before starting (line 986)
- Adds `needs-fixes` when CI fails
- Adds `needs-tess` when CI pending
- Adds `approved` when all CI passes
- Uses GitHub PR reviews (approve/request changes)

**Cleo** (`container-cleo.sh.hbs`):
- Instructed to add `ready-for-qa` label when quality passes (lines 439, 641, 735, 799, 833, 1041)
- Uses `ensure_status_labels()` to create label definitions
- Has helper functions: `pr_add_labels()`, `pr_remove_labels()`

---

## Key Findings

### Consistency Issues

1. **Correlation Labels**:
   - ✅ Codex/Cursor/Factory: Identical implementation
   - ⚠️ Claude: Same labels but different implementation (adds to existing PR vs PR creation)
   - ❌ OpenCode: Missing

2. **Status Labels**:
   - ❌ Codex/Cursor/Factory: Completely missing
   - ✅ Claude: Full implementation with helpers
   - ❌ OpenCode: Missing

3. **Label Color/Description**:
   - Codex/Cursor/Factory: Hardcoded in correlation labels
   - Claude: Defined as variables, passed to functions

### Architectural Differences

**Codex/Cursor/Factory**:
```
container-{agent}.sh.hbs (tiny, 150-200 bytes)
  └─ Includes {{> {cli}_container_base}}
      └─ All logic in container-base.sh.hbs (29-36KB)
```

**Claude**:
```
container-{agent}.sh.hbs (1.3K-2.2K lines each)
  └─ Self-contained, all logic embedded
  └─ No shared base script
```

---

## Label Usage Matrix

| Label | Codex/Cursor/Factory | Claude Rex | Claude Tess | Claude Cleo |
|-------|---------------------|------------|-------------|-------------|
| `task-*` | ✅ Rex (PR creation) | ✅ Post-creation | 📖 Reads | 📖 Reads |
| `service-*` | ✅ Rex (PR creation) | ✅ Post-creation | 📖 Reads | 📖 Reads |
| `run-*` | ✅ Rex (PR creation) | ✅ Post-creation | 📖 Reads | 📖 Reads |
| `needs-fixes` | ❌ | 📖 Reads | ✅ Adds | 📖 Reads |
| `fixing-in-progress` | ❌ | ❌ | ❌ | ❌ |
| `needs-cleo` | ❌ | ✅ Adds | 📖 Reads | 📖 Reads |
| `needs-tess` | ❌ | 📖 Reads | ✅ Adds | 📖 Reads |
| `approved` | ❌ | 📖 Reads | ✅ Adds | 📖 Reads |
| `failed-remediation` | ❌ | 📖 Reads | 📖 Reads | 📖 Reads |
| `ready-for-qa` | ❌ | ❌ | 📖 Checks | ✅ Adds |

**Legend**: ✅ Creates/Adds | 📖 Reads/Checks | ❌ Not implemented

---

## Harmonization Requirements

### Phase 1: Critical (Breaks workflow if not fixed)
1. Add status label logic to Codex/Cursor/Factory base scripts
2. Define who adds `needs-tess` (workflow or Rex?)
3. Define who adds `needs-cleo` (workflow or Tess?)

### Phase 2: Standardization
1. Unify correlation label implementation (during PR creation vs post-creation)
2. Standardize helper functions across all CLIs
3. Remove unused labels (`fixing-in-progress`, `needs tests`)

### Phase 3: Optimization
1. Consider extracting Claude's label logic to shared functions
2. Create unified label color/description constants
3. Add comprehensive error handling

---

## Recommendations

1. **Use Codex base script as template** for correlation labels (cleanest)
2. **Use Claude's status label system** as template (most complete)
3. **Merge both** into Codex/Cursor/Factory base scripts
4. **Refactor Claude** to use shared label functions
5. **Fix OpenCode** from scratch based on final design

