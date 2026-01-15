# Cursor-Inspired Multi-Agent Improvements - Technical Architecture

## Overview

This document describes the technical architecture for implementing improvements to the CTO platform's multi-agent orchestration, based on insights from Cursor's research article "Scaling long-running autonomous coding" (January 2026).

## Background: Key Insights from Cursor

Cursor ran hundreds of concurrent agents on single projects for weeks, generating over a million lines of code. Their key findings:

1. **Flat coordination failed**: Agents with equal status using shared files/locks became bottlenecks, risk-averse, and churned without progress
2. **Planners + Workers worked**: Separate planning and execution layers enabled effective scaling
3. **Judge pattern critical**: A judge agent at end of each cycle determines whether to continue, complete, or restart
4. **Fresh starts combat drift**: Periodic clean context prevents accumulation of mistakes
5. **Model selection by role**: Different models excel at planning vs coding vs evaluation
6. **Simplicity wins**: Removing the integrator role reduced bottlenecks

## Architecture Changes

### Current Architecture

```
PRD → Morgan (once) → tasks.json → Bolt → Rex/Blaze → Cleo → Cipher → Tess → Atlas → Merged
                                    ↑                                              |
                                    └──────────── retry with context ─────────────┘
```

### Improved Architecture

```
PRD → Morgan (continuous planner)
              ↓
       ┌──────┴──────┐
       ↓             ↓
    tasks.json   sub-planners (parallel)
       ↓
    Bolt (infrastructure)
       ↓
    ┌──────────────────────────────────────────┐
    │         Isolated Worker Pool              │
    │  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐      │
    │  │ Rex │  │Blaze│  │Nova │  │Grizz│      │
    │  └─────┘  └─────┘  └─────┘  └─────┘      │
    │     (no inter-worker coordination)        │
    └──────────────────────────────────────────┘
              ↓
    ┌──────────────────────────────────────────┐
    │         Cleo Judge + Quality              │
    │                                           │
    │  Evaluate: acceptance, quality, drift     │
    │  Decide: continue | complete | fresh_start│
    └──────────────────────────────────────────┘
              ↓
       ┌──────┴──────────┬───────────────┐
       ↓                 ↓               ↓
   continue          complete       fresh_start
       ↓                 ↓               ↓
   Workers          Cipher/Tess      Clean context
   iterate              ↓            restart workers
                  Atlas (merge-only)
                        ↓
                    Merged PR
```

## Component Details

### 1. Cleo Judge Mode (US-001)

**Location**: `templates/agents/cleo/`

**New Files**:
- `templates/agents/cleo/judge.md.hbs` - Judge mode prompt partial

**Modified Files**:
- `templates/_shared/partials/acceptance-probe.sh.hbs` - Integrate judge evaluation
- `infra/gitops/manifests/argo-workflows/play-workflow-template.yaml` - Add judge step

**Decision Output Schema**:
```json
{
  "decision": "continue" | "complete" | "fresh_start",
  "reasoning": "string explaining the decision",
  "drift_indicators": ["list of detected drift patterns"],
  "acceptance_completion": 0.95,
  "quality_score": 0.85,
  "risk_aversion_detected": false
}
```

**Environment Variables**:
- `ENABLE_JUDGE_AGENT`: Enable/disable judge evaluation (default: true)
- `JUDGE_DECISION_FILE`: Path to write decision (default: /workspace/.judge_decision)

### 2. Continuous Planning (US-002)

**Location**: `templates/agents/morgan/`

**New Files**:
- `templates/agents/morgan/planner.md.hbs` - Continuous planner mode

**Modified Files**:
- `cto-config.template.json` - Add `continuousPlanning` flag
- `infra/gitops/manifests/argo-workflows/play-workflow-template.yaml` - Add planner hooks

**Planner Events**:
```yaml
# Argo workflow hooks for planner wake-up
hooks:
  onTaskComplete:
    expression: "tasks.any.status == 'Succeeded'"
    action: notify-planner
  onStageComplete:
    expression: "steps['implementation'].status == 'Succeeded'"
    action: evaluate-next-tasks
```

**Sub-Planner Pattern**:
```
Morgan (root planner)
    ├── Sub-planner: Frontend area
    ├── Sub-planner: Backend area
    └── Sub-planner: Infrastructure area
```

### 3. Fresh Start Mechanism (US-003)

**Location**: `templates/_shared/partials/`

**Modified Files**:
- `templates/_shared/partials/retry-loop.sh.hbs`

**Logic**:
```bash
# Fresh start triggers:
# 1. Retry count exceeds threshold
# 2. Judge signals fresh_start
# 3. Explicit drift detection

FRESH_START_THRESHOLD="${FRESH_START_THRESHOLD:-3}"

should_fresh_start() {
    local attempt=$1
    local judge_decision="${JUDGE_DECISION:-continue}"
    
    if [ "$attempt" -gt "$FRESH_START_THRESHOLD" ]; then
        echo "true"
    elif [ "$judge_decision" = "fresh_start" ]; then
        echo "true"
    else
        echo "false"
    fi
}

if [ "$(should_fresh_start $attempt)" = "true" ]; then
    export CONTINUE_SESSION="false"
    export FRESH_START="true"
    # Clear context markers
    rm -f /workspace/.conversation_id
    rm -f /workspace/.session_state
fi
```

**Config**:
```json
{
  "defaults": {
    "play": {
      "freshStartThreshold": 3
    }
  }
}
```

### 4. Role-Specific Models (US-004)

**Location**: `crates/config/src/types.rs`, `cto-config.template.json`

**New Rust Types**:
```rust
/// Model configuration by role, not just by agent.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RoleModels {
    /// Model for planning tasks (opus recommended - extended thinking)
    #[serde(default)]
    pub planner: String,
    
    /// Model for implementation workers (codex recommended - fast execution)
    #[serde(default)]
    pub worker: String,
    
    /// Model for judge evaluation (opus recommended - reasoning)
    #[serde(default)]
    pub judge: String,
    
    /// Model for code review (sonnet recommended - balance)
    #[serde(default)]
    pub reviewer: String,
}

impl RoleModels {
    /// Get model for a specific role, with fallback.
    pub fn get_model(&self, role: &str, fallback: &str) -> &str {
        match role {
            "planner" if !self.planner.is_empty() => &self.planner,
            "worker" if !self.worker.is_empty() => &self.worker,
            "judge" if !self.judge.is_empty() => &self.judge,
            "reviewer" if !self.reviewer.is_empty() => &self.reviewer,
            _ => fallback,
        }
    }
}
```

**Config Schema**:
```json
{
  "defaults": {
    "play": {
      "roleModels": {
        "planner": "claude-opus-4-5-20251101",
        "worker": "gpt-5.2-codex",
        "judge": "claude-opus-4-5-20251101",
        "reviewer": "claude-sonnet-4-5-20250514"
      }
    }
  }
}
```

### 5. Worker Isolation (US-005)

**Location**: `templates/_shared/partials/`, `templates/agents/_subagents/`

**Modified Files**:
- `templates/_shared/partials/coordinator.md.hbs` - Remove peer awareness
- `templates/_shared/partials/subagent-dispatch.md.hbs` - Enforce isolation
- `templates/agents/_subagents/*.md.hbs` - Strip coordination context

**Isolation Principles**:

```markdown
## Worker Context (ISOLATED)

You are a focused worker. Your context includes:
- Your specific task description
- Relevant code files
- Task acceptance criteria

Your context does NOT include:
- Other workers' tasks
- Other workers' status
- Overall project progress
- Coordination responsibilities

Focus entirely on YOUR task. Do not:
- Reference other workers
- Wait for peer completion
- Coordinate with peers
- Make decisions for the team
```

**Enforcement**:
```hbs
{{!-- In subagent dispatch --}}
{{#if worker_isolation}}
{{!-- Strip all coordination context --}}
{{else}}
{{> coordinator }}
{{/if}}
```

### 6. Simplified Atlas (US-006)

**Location**: `templates/agents/atlas/`

**Current Atlas Responsibilities**:
- Verify all checks pass
- Review code quality
- Resolve merge conflicts
- Perform final integration
- Merge PR

**Simplified Atlas Responsibilities**:
- Verify CI passes
- Merge PR

**Removed Responsibilities** (handled by others):
- ~~Code quality~~ → Cleo judge
- ~~Merge conflicts~~ → Workers handle themselves
- ~~Final integration~~ → Workers already committed

**Modified Files**:
- `templates/agents/atlas/integration.md.hbs` - Simplify to merge-only

## Configuration Schema Updates

### cto-config.template.json additions

```json
{
  "defaults": {
    "play": {
      "continuousPlanning": false,
      "freshStartThreshold": 3,
      "enableJudgeAgent": true,
      "workerIsolation": true,
      "roleModels": {
        "planner": "claude-opus-4-5-20251101",
        "worker": "gpt-5.2-codex", 
        "judge": "claude-opus-4-5-20251101",
        "reviewer": "claude-sonnet-4-5-20250514"
      }
    }
  }
}
```

## Workflow Template Changes

### play-workflow-template.yaml

```yaml
# New parameters
- name: enable-judge-agent
  description: "Enable Cleo judge mode for iteration decisions"
  value: "true"
- name: fresh-start-threshold
  description: "Retry count before triggering fresh start"
  value: "3"
- name: continuous-planning
  description: "Enable Morgan continuous planning mode"
  value: "false"
- name: worker-isolation
  description: "Enable strict worker context isolation"
  value: "true"

# New step: Cleo Judge (after implementation, before security)
- name: judge-evaluation
  template: cleo-judge
  when: "{{workflow.parameters.enable-judge-agent}} == 'true'"
  arguments:
    parameters:
      - name: task-id
        value: "{{workflow.parameters.task-id}}"

# Modified: Atlas simplified
- name: atlas-merge
  template: atlas-merge-only  # Renamed from atlas-integration
```

## Migration Path

1. **Phase 1**: Add new config options with defaults that preserve current behavior
2. **Phase 2**: Implement Cleo judge and fresh start (low risk, high value)
3. **Phase 3**: Implement worker isolation and simplified Atlas
4. **Phase 4**: Implement continuous planning (highest complexity)
5. **Phase 5**: Enable new features by default after validation

## Observability

### New Metrics
- `cto_judge_decisions_total{decision}` - Count of judge decisions by type
- `cto_fresh_starts_total` - Count of fresh start triggers
- `cto_drift_detected_total` - Count of drift detections
- `cto_planner_tasks_created_total` - Dynamic tasks created by planner

### New Log Events
- `judge.decision` - Judge evaluation result
- `worker.fresh_start` - Fresh start triggered
- `planner.task_created` - Dynamic task creation
- `worker.isolation_enforced` - Worker started with isolated context

## Testing Strategy

1. **Unit Tests**: Test RoleModels struct, config parsing
2. **Template Tests**: Validate Handlebars template rendering
3. **Integration Tests**: Run shortened play workflow with new features
4. **E2E Tests**: Full workflow validation with all features enabled
