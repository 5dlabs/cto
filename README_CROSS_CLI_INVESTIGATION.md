# Cross-CLI Workflow Resume Investigation

## Quick Navigation

This investigation comprehensively analyzes how the Multi-Agent orchestration platform tracks and resumes workflows across all 6 CLI types (Claude, Codex, Cursor, Factory, OpenCode, and the Blaze/Rex implementation agents).

### Documentation Files Created

1. **CROSS_CLI_RESUME_ANALYSIS.md** (22KB)
   - Complete technical analysis of state persistence layers
   - 12 detailed sections covering every aspect of resume functionality
   - Code references and line numbers for all key files
   - Architecture deep-dive with examples
   - **Start here** if you want the comprehensive technical reference

2. **INVESTIGATION_SUMMARY.md** (8.8KB)
   - Executive summary of key findings
   - Quick reference to critical files
   - State persistence summary table
   - Resume flow scenarios (Claude → Factory → Codex)
   - **Start here** if you want a quick overview

3. **STATE_ARCHITECTURE_DIAGRAM.txt** (17KB)
   - ASCII diagrams showing state flow
   - Visual representation of PVC allocation strategy
   - Stage progression with sensor logic
   - Cross-CLI transition step-by-step flow
   - **Start here** if you're a visual learner

## Key Finding: It's Not About CLAUDE.md

The system does NOT rely on CLAUDE.md alone for cross-CLI resume. It uses a sophisticated multi-layer architecture:

### State Storage Hierarchy

1. **Workflow Labels** (Kubernetes metadata)
   - `task-id`: Which task is being worked
   - `repository`: Which repository (label-friendly format)
   - `current-stage`: Current execution stage (implementation/code-quality/testing/etc)
   - **Function**: Primary resume signal; sensors match by labels

2. **ConfigMaps** (Kubernetes objects)
   - `play-progress-{repo}`: Task progress tracking
   - **Function**: Persists state when workflow is deleted; restored on recreation

3. **PVCs** (Persistent Volumes)
   - `workspace-{service}`: SHARED workspace for implementation agents (Rex, Blaze)
   - `workspace-{service}-{agent}`: ISOLATED workspaces for quality/testing agents
   - **Function**: Agent work persistence; prevents data loss across workflow recreation

4. **Agent Session Files** (On PVCs)
   - `CLAUDE.md`: Universal memory file (CLI-agnostic)
   - Agent-specific state in `.agent-state/` directories
   - **Function**: Session context continuity

5. **GitHub Events + Sensors** (Event-driven orchestration)
   - Sensors detect PR creation, approval, merge events
   - Match events to workflows by labels
   - Verify stage before resuming (prevents wrong progression)
   - **Function**: External event coordination

## How Resume Works

### Scenario: Workflow Deleted & Recreated (Same Task)

```
BEFORE:
  Workflow: play-task-5-abc123
  Labels: task-id=5, repository=5dlabs-cto, current-stage=waiting-pr-created
  PVCs: workspace-cto (PERSISTS after workflow deletion)
  ConfigMap: play-progress-5dlabs-cto (PERSISTS after workflow deletion)

DELETE: kubectl delete workflow play-task-5-abc123

RECREATE:
  New Workflow: play-task-5-xyz789 (different name)
  Labels: task-id=5, repository=5dlabs-cto, current-stage=waiting-pr-created (restored from ConfigMap)
  
  GitHub PR Created Event:
    → Sensor extracts: task-id=5 from PR labels
    → Sensor finds: workflow with task-id=5, repository=5dlabs-cto, workflow-type=play
    → Sensor verifies: current-stage == "waiting-pr-created"? YES
    → Sensor resumes: kubectl patch workflow --type=merge -p '{"spec":{"suspend":false}}'
    → Workflow continues from exact suspension point
    → New agent mounts: workspace-cto (same PVC) → previous work available
    → continueSession: true → loads CLAUDE.md → context restored

RESULT:
  ✅ No data loss
  ✅ No stage regression
  ✅ Same PVC mounts (history preserved)
  ✅ Stage verified before resume (safety check)
```

### Scenario: CLI Transition (Claude → Factory → Codex)

```
Claude + Rex (Implementation):
  - PVC: workspace-cto (SHARED)
  - Creates: CLAUDE.md, implementation code, PR #314
  - Stage: waiting-pr-created

Factory + Blaze (Implementation):
  - Can mount: workspace-cto (SAME PVC)
  - Can read: CLAUDE.md from previous session
  - continueSession: true → resumes context
  - Builds on Rex's code

Codex + Cleo (Code Quality):
  - PVC: workspace-cto-cleo (ISOLATED)
  - New clean workspace
  - Can read: implementation code (copied from workspace-cto)
  - Creates: CLEO.md
  - Runs quality checks

Event: GitHub PR Review (Cleo[bot] approves)
  → Sensor finds workflow by labels
  → Verifies: current-stage == "waiting-quality-complete"
  → Resumes workflow for Tess stage

Claude + Tess (Testing):
  - PVC: workspace-cto-tess (ISOLATED)
  - New clean workspace
  - Can reference: implementation work via ConfigMap
  - Creates: TESS.md with test results
```

**The key insight**: CLI doesn't matter. The system tracks:
- Agent role (Rex, Blaze, Cleo, Tess)
- Current stage (implementation, code-quality, testing)
- Workflow labels (task-id, repository)
- PVC persistence (persistent across deletion)

## Universal vs CLI-Specific

### What's Universal (Shared Across All CLIs)
- PVC mount path: `/workspace`
- Memory file: `CLAUDE.md` (universal name)
- Workflow labels: `task-id`, `repository`, `current-stage`
- ConfigMap naming: `play-progress-{repo}`
- CodeRun CRD structure
- Stage progression model
- GitHub event model

### What's CLI-Specific
- Container image (claude-image vs factory-image vs codex-image)
- CLI configuration (model, temperature, max-tokens per CLI)
- GitHub App credentials (per agent)
- Tool implementations (each CLI has adapters)

## Critical Files to Understand

### State Management
- `controller/src/tasks/play/progress.rs` - ConfigMap state storage
- `controller/src/cli/session.rs` - Session management
- `controller/src/tasks/workflow.rs` - Workflow resumption

### Agent & Workspace
- `controller/src/tasks/code/agent.rs` - Agent classification and PVC naming
- `controller/src/tasks/code/resources.rs` - PVC persistence logic

### Stage Orchestration
- `infra/charts/controller/templates/stage-transitions-template.yaml` - DAG with stage updates
- `infra/gitops/resources/github-webhooks/stage-aware-*.yaml` - Event-based resume sensors

### Configuration
- `infra/charts/controller/templates/agents-configmap.yaml` - Agent definitions
- `infra/charts/controller/templates/coderun-template.yaml` - CodeRun template

### CLI System
- `controller/src/cli/types.rs` - CLIType enum, UniversalConfig
- `controller/src/cli/mod.rs` - CLI module organization

## State Persistence Summary

| Component | Storage | Deletion Behavior | Resume Function |
|---|---|---|---|
| Workflow Stage | K8s Labels | NOT deleted with workflow | Sensor matches by label, verifies stage |
| Agent Workspace | PVC | PERSISTENT, reused | New agent mounts same PVC |
| Task Progress | ConfigMap | PERSISTENT unless manual delete | Tracks current task/stage |
| Agent Session | CLAUDE.md on PVC | PERSISTENT | `continueSession: true` loads file |
| CLI State | SessionState | Cleanup on timeout | Records transitions |
| PR Metadata | Workflow annotations | Injected by sensor | Sensor updates on resume |

## FAQ: Common Questions Answered

### Q: If I delete a workflow, can I resume it?
**A**: YES. ConfigMap stores current task/stage. Recreate workflow with same task-id, and it resumes from exact stage.

### Q: Can Claude resume where Factory left off?
**A**: YES. The PVC is agent-agnostic. Implementation agents (Rex/Blaze) share workspace. CLI doesn't matter.

### Q: What if I delete the PVC?
**A**: Data loss for that agent's work. PVCs are meant to PERSIST indefinitely.

### Q: What if I delete the ConfigMap?
**A**: Loss of progress tracking. Workflow can't resume cleanly from previous stage. BUT workflow labels still exist in YAML backups.

### Q: Do I need CLAUDE.md for resume to work?
**A**: NO. CLAUDE.md is optional context. Core resume works via labels + PVC + ConfigMap + sensors.

### Q: How does the system prevent stage regression?
**A**: Sensors verify `current-stage` label before resuming. PR created event only resumes if stage == "waiting-pr-created". Wrong stage = skip resume.

### Q: What's the difference between implementation and quality agents?
**A**: 
- Implementation (Rex, Blaze) → SHARED PVC (`workspace-{service}`) → Direct filesystem handoff
- Quality/Testing (Cleo, Tess) → ISOLATED PVC (`workspace-{service}-{agent}`) → Metadata-based handoff

## Recommendations

### For Better Visibility
1. Extend ConfigMap with explicit stage results
2. Add workflow annotations for agent transitions
3. Track CLI-to-agent mapping in ConfigMap

### For Better Debugging
1. Add labels: `last-sensor-interaction`, `expected-next-event`
2. Add metrics: `workflow_stage_transitions_total`
3. Create AgentHandoff CRD for audit trail (optional)

## Next Steps

1. **Read INVESTIGATION_SUMMARY.md** for a quick overview
2. **Review STATE_ARCHITECTURE_DIAGRAM.txt** for visual understanding
3. **Deep-dive CROSS_CLI_RESUME_ANALYSIS.md** for technical details
4. **Explore key files** listed in each document

---

## Document Manifest

- **CROSS_CLI_RESUME_ANALYSIS.md**: Full technical reference (sections 1-12)
- **INVESTIGATION_SUMMARY.md**: Executive summary with file index
- **STATE_ARCHITECTURE_DIAGRAM.txt**: ASCII diagrams and visual flows
- **README_CROSS_CLI_INVESTIGATION.md**: This file (navigation guide)

---

**Last Updated**: November 12, 2025
**Investigation Scope**: Cross-CLI resume architecture across all 6 CLI types
**Key Finding**: Resume is CLI-agnostic; relies on labels, PVCs, ConfigMaps, and events

