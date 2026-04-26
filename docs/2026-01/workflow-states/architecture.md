# CTO Platform Improvements - Architecture

## Overview

This document provides architectural context for implementing the CTO platform improvements defined in `prd.json`. The improvements span four phases:

1. **Phase 0**: Intake clarifications (remove confusion, fix routing)
2. **Phase 1**: Fresh Start mechanism (combat agent drift)
3. **Phase 1.5**: Subagent wiring (parallel subtask execution)
4. **Phase 2**: Linear Task Sync (mid-flight updates)

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           CTO PLATFORM ARCHITECTURE                          │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Linear    │───▶│  PM Server  │───▶│    Argo     │───▶│  Controller │
│  (Issues)   │    │  (Webhook)  │    │ (Workflow)  │    │  (CodeRun)  │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
                          │                                      │
                          ▼                                      ▼
                   ┌─────────────┐                        ┌─────────────┐
                   │   Intake    │                        │   Agent     │
                   │  (Tasks)    │                        │  Container  │
                   └─────────────┘                        └─────────────┘
                          │                                      │
                          ▼                                      ▼
                   ┌─────────────┐                        ┌─────────────┐
                   │  Docs Repo  │◀────────────────────────│ Tool Server │
                   │ (task files)│                        │   (MCP)     │
                   └─────────────┘                        └─────────────┘
```

## Key Components

### 1. Intake Module (`crates/intake/`)

**Purpose**: Parse PRDs and generate task files for agent execution.

**Current Issues**:
- Task 1 hardcoded to Bolt in `bin/cli.rs:1145-1160`
- No mechanism for mid-flight updates
- `local=true` option causes confusion

**Changes Required**:
- Remove Task 1 = Bolt enforcement
- Add `intake update` command for PRD changes
- Add `intake sync-task` command for Linear edits

### 2. Controller (`crates/controller/`)

**Purpose**: Generate agent containers and template files from CodeRun CRDs.

**Current State**:
- SubagentConfig exists but not passed to templates
- No subtasks field in CodeRunSpec
- `group_by` helper not registered

**Changes Required**:
- Pass `subagents` config to template context
- Add `subtasks` field to CodeRunSpec
- Register `group_by` Handlebars helper

### 3. Config Module (`crates/config/`)

**Purpose**: Parse and validate cto-config.json settings.

**Current State**:
- SubagentConfig.should_use() only checks claude
- No fresh_start_threshold in PlayDefaults
- No autoAppendDeployTask in IntakeDefaults

**Changes Required**:
- Add OpenCode to should_use()
- Add fresh_start_threshold field
- Add autoAppendDeployTask field

### 4. Healer (`crates/healer/`)

**Purpose**: Monitor play sessions and spawn remediation agents.

**Current State**:
- Template code for Fresh Start exists
- Config value never passed through

**Changes Required**:
- Pass fresh_start_threshold to template context

## Data Flow

### Task Creation Flow (Current)

```
PRD → Intake → Task Files → Docs Repo → Agent Reads
                  │
                  └─ Task 1 FORCED to Bolt (wrong!)
```

### Task Creation Flow (Desired)

```
PRD → Intake → Task Files → Docs Repo → Agent Reads
                  │
                  └─ ALL tasks routed by content
                  └─ Optional deploy task appended
```

### Subagent Flow (New)

```
┌─────────────────────────────────────────────────────────────────┐
│  Task with subagents.enabled=true                               │
│                                                                  │
│  Primary Agent (Coordinator)                                     │
│       │                                                          │
│       ├─▶ @implementer (subtask 1) ──┐                          │
│       ├─▶ @implementer (subtask 2) ──┼─▶ Level 0 (parallel)     │
│       ├─▶ @researcher (subtask 3)  ──┘                          │
│       │                                                          │
│       └─▶ Wait for Level 0                                       │
│       │                                                          │
│       ├─▶ @tester (subtask 4) ───────┐                          │
│       └─▶ @reviewer (subtask 5) ─────┼─▶ Level 1 (parallel)     │
│                                       │                          │
│       └─▶ Aggregate results           │                          │
└─────────────────────────────────────────────────────────────────┘
```

### Linear Task Sync Flow (New)

```
┌─────────────────────────────────────────────────────────────────┐
│  Source: PRD/Architecture Change                                 │
│                                                                  │
│  User edits PRD.md ─▶ intake update ─▶ Diff tasks               │
│                                            │                     │
│                                            ▼                     │
│                                      Delta PR to Docs            │
│                                            │                     │
│                                            ▼                     │
│                                   Next agent run picks up        │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  Source: Linear Issue Edit                                       │
│                                                                  │
│  User edits issue ─▶ intake sync-task ─▶ Parse issue            │
│                                              │                   │
│                                              ▼                   │
│                                      Update task files           │
│                                              │                   │
│                                              ▼                   │
│                                        PR to Docs                │
└─────────────────────────────────────────────────────────────────┘
```

## Configuration Schema

### cto-config.json (Relevant Fields)

```json
{
  "defaults": {
    "play": {
      "freshStartThreshold": 3,
      "_comment": "After N retries, clear context and restart fresh"
    },
    "intake": {
      "autoAppendDeployTask": false,
      "_comment": "Auto-append Bolt deploy task for deployable projects"
    }
  },
  "agents": {
    "rex": {
      "cli": "claude",
      "model": "claude-opus-4-5-20251101",
      "subagents": {
        "enabled": true,
        "maxConcurrent": 5
      },
      "tools": {
        "remote": ["firecrawl_*", "github_*"],
        "localServers": {}
      }
    }
  }
}
```

### CodeRunSpec (New Fields)

```yaml
apiVersion: agents.platform/v1alpha1
kind: CodeRun
spec:
  # ... existing fields ...
  
  # NEW: Subtasks for parallel execution
  subtasks:
    - id: 1
      title: "Create user model"
      subagentType: "implementer"
      executionLevel: 0
      parallelizable: true
      dependencies: []
    - id: 2
      title: "Write tests"
      subagentType: "tester"
      executionLevel: 1
      parallelizable: true
      dependencies: ["1"]
```

## Testing Strategy

### Unit Tests

| Component | Test File | Coverage |
|-----------|-----------|----------|
| Config parsing | `crates/config/src/types.rs` | fresh_start_threshold, subagent |
| Task routing | `crates/intake/tests/` | Task 1 routing by content |
| Template rendering | `crates/controller/tests/` | group_by helper, subtasks |

### Integration Tests

| Test | Description |
|------|-------------|
| `intake_update` | PRD change generates correct delta PR |
| `intake_sync_task` | Linear edit syncs to task files |
| Fresh Start | Context clears after threshold retries |
| Subagent dispatch | Subtasks render in coordinator prompt |

## Dependencies

### External Dependencies

- **Linear API**: For sync-task command
- **GitHub API**: For creating delta PRs
- **Argo Workflows**: For play orchestration

### Internal Dependencies

```
STORY-001 ──┐
STORY-002 ──┼─▶ STORY-004 ─▶ STORY-005 ─▶ STORY-006 ─▶ STORY-007 ─▶ STORY-008 ─▶ STORY-009
STORY-003 ──┘                                                              │
                                                                           ▼
                                                              STORY-010 ─▶ STORY-011 ─▶ STORY-012
```

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Breaking existing intake | Comprehensive test coverage |
| Subagent CLI compatibility | Check should_use() for each CLI |
| Linear API rate limits | Implement backoff/retry |
| Delta PR conflicts | Use 3-way merge strategy |

## Success Criteria

1. **Phase 0**: All tasks route by content, no hardcoded Bolt assumption
2. **Phase 1**: Fresh Start triggers after configured threshold
3. **Phase 1.5**: Subagent templates render correctly with execution levels
4. **Phase 2**: `intake update` and `intake sync-task` work E2E
