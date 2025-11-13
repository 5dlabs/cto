# Cross-CLI Resume Investigation - Key Findings & File Index

## Investigation Scope

Researched how the Multi-Agent platform tracks and resumes workflow state across all 6 CLIs (Claude, Codex, Cursor, Factory, OpenCode, Blaze/Rex) to understand how workflows can be deleted and restarted while maintaining full context and progression state.

## Key Discoveries

### 1. Multi-Layer State Architecture (Not Just CLAUDE.md)

The system DOES NOT rely solely on CLAUDE.md for cross-CLI resume. Instead it uses:

- **Workflow Labels** → Stage progression (primary resume signal)
- **PVCs** → Agent workspace persistence (implementation vs. isolated)
- **ConfigMaps** → Task tracking and progress state
- **CLAUDE.md** → Agent-specific session memory (secondary)
- **GitHub Events + Sensors** → Event-driven stage transitions

### 2. Stage-Based Workflow Orchestration

Workflows progress through **named stages** that are:
- Stored as Kubernetes labels (`current-stage` field)
- Verified by sensors before resuming (prevents stage regression)
- Updated atomically when each agent completes
- Independent of which CLI/agent is running

**Stage Flow**: `implementation` → `waiting-pr-created` → `code-quality` → `waiting-ready-for-qa` → `testing` → `waiting-pr-merged` → `completed`

### 3. Agent-Aware PVC Strategy

PVC naming reveals agent classification:

```
workspace-{service}           → Shared by implementation agents (Rex, Blaze)
workspace-{service}-{agent}   → Isolated for quality/testing agents (Cleo, Tess)
```

This means:
- Rex and Blaze share the SAME workspace → direct filesystem handoff
- Cleo/Tess get clean, isolated workspaces → metadata-based coordination

### 4. Resume Mechanics on Deletion

When a workflow is deleted and recreated:

1. **Same labels restored** from ConfigMap (`task-id`, `repository`, `current-stage`)
2. **Same PVC mounted** (PVCs are persistent, NOT deleted)
3. **Previous work available** immediately to new agent
4. **Sensor matches new workflow** by labels, not by name
5. **Stage verification prevents wrong progression** (sensor checks current stage before resuming)

## Critical Files Analyzed

### State Management
- `/controller/src/tasks/play/progress.rs` - PlayProgress ConfigMap storage
- `/controller/src/cli/session.rs` - SessionState management for cross-CLI transitions
- `/controller/src/tasks/workflow.rs` - Workflow resumption and PR context injection

### Agent & Workspace Strategy
- `/controller/src/tasks/code/agent.rs` - AgentClassifier: PVC naming logic, shared vs. isolated
- `/controller/src/tasks/code/resources.rs` - PVC creation/persistence, idempotent mounts

### Stage Orchestration
- `/infra/charts/controller/templates/stage-transitions-template.yaml` - Multi-agent DAG with stage updates
- `/infra/gitops/resources/github-webhooks/stage-aware-*.yaml` - Sensors for stage-based resume

### Workflow Templates
- `/infra/charts/controller/templates/coderun-template.yaml` - CodeRun creation with CLI config
- `/infra/charts/controller/templates/play-project-workflow-template.yaml` - Full project orchestration

### Configuration
- `/infra/charts/controller/templates/agents-configmap.yaml` - Agent definitions, system prompts
- `/infra/charts/controller/templates/agents-config.yaml` - Agents list and routing

### CLI Adapters
- `/controller/src/cli/types.rs` - CLIType enum, UniversalConfig schema
- `/controller/src/cli/mod.rs` - CLI module organization (adapters, session, discovery)

## State Persistence Summary

| Level | Storage | Persistence | Resume Function |
|---|---|---|---|
| **Workflow Stage** | K8s Labels | Until manual deletion | Sensor matches by label, verifies stage |
| **Agent Workspace** | PVC | Indefinite | New agent mounts same PVC |
| **Task Progress** | ConfigMap `play-progress-*` | Until manual deletion | Tracks current task/stage for recovery |
| **Agent Session** | CLAUDE.md on PVC | Until overwrite-memory=true | `continueSession: true` loads file |
| **CLI State** | SessionState | In-memory (with persistence layer) | Records CLI transitions |
| **PR Context** | Workflow annotations | Injected by sensor on resume | Sensor updates PR URL/number |

## How Resume Works Across CLIs

### Scenario: Claude → Factory → Codex (same task)

```
Claude (5DLabs-Rex):
  ├─ Creates workflow with labels: task-id=5, repository=5dlabs-cto
  ├─ Mounts: workspace-cto (SHARED)
  ├─ Creates: CLAUDE.md, PR #314
  └─ Completes, workflow suspended at stage: waiting-pr-created

Factory (5DLabs-Blaze):
  ├─ Can mount: workspace-cto (SAME PVC)
  ├─ Can read: Rex's work from previous files
  ├─ Can read: CLAUDE.md from previous session
  ├─ continueSession: true → Resumes from CLAUDE.md
  └─ Builds on Rex's implementation

Codex (5DLabs-Cleo):
  ├─ New CodeRun CRD (but same workflow labels)
  ├─ Mounts: workspace-cto-cleo (ISOLATED)
  ├─ Gets: Implementation files copied from workspace-cto
  ├─ Creates: CLEO.md (its own memory)
  └─ Runs quality checks on Blaze's code

GitHub Event: PR Review (Cleo[bot] approved)
  ├─ Sensor reads: PR labels → task-id=5
  ├─ Sensor finds: workflow with task-id=5, repository=5dlabs-cto
  ├─ Sensor checks: current-stage == "waiting-quality-complete"?
  ├─ YES: Resume workflow (suspend: false)
  └─ Workflow continues to next stage

Claude (5DLabs-Tess):
  ├─ New CodeRun CRD (same task, same labels)
  ├─ Mounts: workspace-cto-tess (ISOLATED)
  ├─ Runs: E2E tests on implementation
  └─ Creates: TESS.md with test results
```

**The key insight**: The CLI never matters. It's all about:
1. Workflow labels (task-id, repository)
2. Current stage label
3. PVC names (derived from agent, not CLI)
4. GitHub events matching stage expectations

## Universal vs. CLI-Specific

### Universal (Shared Across All CLIs)

```
✓ PVC mount path: /workspace
✓ CLAUDE.md name (universal memory file)
✓ Workflow label schema: task-id, repository, current-stage
✓ ConfigMap naming: play-progress-{repo}
✓ CodeRun CRD structure
✓ Stage progression model
✓ GitHub event model
```

### CLI-Specific (Per CLI)

```
✗ Container image (claude-image vs. factory-image vs. codex-image)
✗ CLI config: model, temperature, max-tokens
✗ GitHub App credentials (per CLI agent combo)
✗ Tool implementations (each CLI adapts tools differently)
✗ Container context (env vars, entry points)
```

## Missing or Implicit State

Currently implemented but could be more explicit:

1. **No "last-agent" tracking** in ConfigMap
   - Could add: `last-agent: "rex"`, `last-agent-completion-time: "..."`
   - Would make handoff audit trail more visible

2. **No stage-completion summary**
   - Could persist: `stage-results.yaml` with PR numbers, timestamps, approval status
   - Would provide better visibility into workflow history

3. **No explicit CLI-transition tracking**
   - CLI transitions are recorded in SessionState.execution_history
   - Could also be reflected in ConfigMap for kubectl visibility

4. **No agent-handoff metadata**
   - Handoff is implicit (via shared/isolated PVCs)
   - Could add: workflow annotations marking agent transitions

## Recommendations

### For Better Resume Visibility

1. **Extend play-progress ConfigMap** with:
   - `last-completed-agent: "blaze"`
   - `stage-results.yaml` (per-stage metadata)
   - `cli-history.yaml` (CLI transitions)

2. **Add Workflow Annotations** for:
   - `agents.platform/current-agent: "cleo"`
   - `agents.platform/stage-history: "implementation→quality→testing"`

3. **Create AgentHandoff CRD** (optional) for:
   - Audit trail of agent transitions
   - Stage completion details
   - CLI used per stage

### For Better Debugging

1. Add label to workflows: `last-sensor-interaction: "2025-11-12T10:15:00Z"`
2. Add annotation: `agents.platform/expected-next-event: "pull_request.approved by 5dlabs-cleo"`
3. Add metric: `workflow_stage_transitions_total{from_stage, to_stage}`

## Files to Review Further

If you need to implement enhancements:

1. **To track stages better**: `infra/charts/controller/templates/stage-transitions-template.yaml`
2. **To add CLI tracking**: `controller/src/cli/session.rs` (already has execution_history)
3. **To enhance progress tracking**: `controller/src/tasks/play/progress.rs`
4. **To modify agent coordination**: `controller/src/tasks/code/agent.rs`
5. **To change workflow templates**: `infra/charts/controller/templates/workflowtemplates/`

## Conclusion

The resume system is **universal and CLI-agnostic** by design:
- Workflows don't care which CLI runs them
- State is stored in K8s labels, PVCs, and ConfigMaps (not CLI-specific)
- Agents transition seamlessly based on roles, not CLI type
- Resume works because of labels, not because we remember "Claude was here"

The architecture is actually quite elegant: **It's not tracking which CLI ran—it's tracking which AGENT ran and which STAGE we're at.** That's sufficient for complete resume.

