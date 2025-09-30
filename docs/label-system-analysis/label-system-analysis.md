# Multi-Agent Label System - Current State & Harmonization Plan

## Current Production State (PR #21)

### Labels Applied:
- `task-1` - Task correlation (added by Rex during PR creation)
- `service-rust-basic-api` - Service correlation (added by Rex)
- `run-play-workflow-template-fzzvq` - Workflow run tracking (added by Rex)
- `needs-tess` - Added by workflow/Cleo (status label)

## CLI Implementation Status

### ✅ Codex (Primary, Most Complete)
- **Base Script**: 36KB `container-base.sh.hbs`
- **Label Logic**: Complete `ensure_pr_created()` function
- **Correlation Labels**: task-*, service-*, run-*
- **Agent Scripts**: Use `{{> codex_container_base}}` partial
- **Status**: Production-ready

### ✅ Cursor  
- **Base Script**: 31KB `container-base.sh.hbs`
- **Label Logic**: Has label functions
- **Status**: Needs verification vs Codex

### ✅ Factory
- **Base Script**: 29KB `container-base.sh.hbs`  
- **Label Logic**: Has label functions
- **Status**: Needs verification vs Codex

### ⚠️ Claude (Different Architecture)
- **Base Script**: None - monolithic agent scripts
- **Scripts**: Separate 54-90KB files per agent
- **Label Logic**: Built into each agent script individually
- **Status**: Needs harmonization - likely has label logic but different structure

### ❌ OpenCode
- **Base Script**: 375 bytes (stub)
- **Status**: Incomplete/placeholder

## Label Categories

### 1. Correlation Labels (Rex adds during PR creation)
```
task-{id}           - Orange (f29513) - Task correlation
service-{name}      - Green (0e8a16) - Service correlation  
run-{workflow}      - Blue (0366d6) - Workflow correlation
```

### 2. Stage/Status Labels (Workflow & Agents add)
```
needs-fixes         - Red (d73a4a) - Remediation requested by Tess
fixing-in-progress  - Yellow (fbca04) - Rex is actively applying fixes
needs-cleo          - Green (0e8a16) - Awaiting Cleo quality review
needs-tess          - Purple (5319e7) - Awaiting Tess QA review  
approved            - Dark Green (2da44e) - All automated reviews approved
failed-remediation  - Dark Red (b60205) - Remediation failed or aborted
```

## Proposed Harmonized Flow

### Current Flow (Rex → Cleo → Tess) - INCORRECT
```
1. Rex creates PR with correlation labels
2. Workflow waits for PR webhook
3. Cleo reviews code quality → adds "ready-for-qa"
4. Tess tests functionality → GitHub approval
```

### Corrected Flow (Rex → Tess → Cleo) - TARGET
```
1. Rex creates PR with correlation labels (task-*, service-*, run-*)
   └─ Workflow adds "needs-tess" label

2. Tess tests functionality
   └─ GitHub PR approval when tests pass
   └─ Workflow detects approval, adds "needs-cleo" label

3. Cleo reviews code quality  
   └─ GitHub PR approval when quality checks pass
   └─ Workflow detects approval, marks as "approved"
```

### Benefits:
- Uses native GitHub approvals (simpler)
- Only 2 stage labels: needs-tess, needs-cleo
- Correlation labels stay consistent
- Status labels for remediation flow

## Harmonization Tasks

### Phase 1: Audit & Document
- [ ] Compare Codex vs Cursor vs Factory base scripts
- [ ] Document Claude's monolithic label logic
- [ ] Identify any label differences between CLIs

### Phase 2: Design Unified System
- [ ] Define canonical label set
- [ ] Define who adds what labels when
- [ ] Document sensor expectations

### Phase 3: Implementation
- [ ] Update Codex base script (reference implementation)
- [ ] Update Cursor base script to match
- [ ] Update Factory base script to match  
- [ ] Refactor Claude monolithic scripts
- [ ] Update workflow template
- [ ] Update sensors

### Phase 4: Testing
- [ ] Test each CLI independently
- [ ] Test full Rex→Tess→Cleo flow
- [ ] Verify label correlation works

## Open Questions

1. Should Tess add "testing-complete" label or just use GitHub approval?
2. Should Cleo add "ready-for-qa" label or just use GitHub approval?
3. Do we need separate "approved" label or rely on GitHub approval count?
4. How do we handle remediation loop labels (needs-fixes, fixing-in-progress)?

