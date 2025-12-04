# Healer: Play Orchestration Design

> **Primary Goal**: Ensure play workflows complete from beginning to end by tracking what *should* happen, what *did* happen, and what *needs* to happen.

## Executive Summary

Healer needs to be the authoritative source of truth for play workflow execution. It should:
1. **Define** the expected workflow structure (stages, agents, transitions)
2. **Track** actual execution state in real-time
3. **Detect** deviations between expected and actual
4. **Guide** workflows back on track through remediation

---

## Current Play Workflow Structure

Based on the workflow templates, here's the canonical play workflow:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         PLAY WORKFLOW STAGES                             │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────┐                                                        │
│  │   PENDING   │  Initial state - workflow started                      │
│  └──────┬──────┘                                                        │
│         │                                                                │
│         ▼                                                                │
│  ┌─────────────────────────────────┐                                    │
│  │  IMPLEMENTATION-IN-PROGRESS     │  Rex writing code                  │
│  │        (Rex)                    │                                    │
│  └──────┬──────────────────────────┘                                    │
│         │                                                                │
│         ▼                                                                │
│  ┌─────────────────────────────────┐                                    │
│  │    QUALITY-IN-PROGRESS          │  Cleo reviewing code               │
│  │        (Cleo)                   │                                    │
│  └──────┬──────────────────────────┘                                    │
│         │                                                                │
│         ▼                                                                │
│  ┌─────────────────────────────────┐                                    │
│  │   SECURITY-IN-PROGRESS          │  Cipher security scan              │
│  │       (Cipher)                  │                                    │
│  └──────┬──────────────────────────┘                                    │
│         │                                                                │
│         ▼                                                                │
│  ┌─────────────────────────────────┐                                    │
│  │    TESTING-IN-PROGRESS          │  Tess running tests                │
│  │        (Tess)                   │                                    │
│  └──────┬──────────────────────────┘                                    │
│         │                                                                │
│         ▼                                                                │
│  ┌─────────────────────────────────┐                                    │
│  │  WAITING-ATLAS-INTEGRATION      │  Atlas merging PR                  │
│  │       (Atlas)                   │                                    │
│  └──────┬──────────────────────────┘                                    │
│         │                                                                │
│         ▼                                                                │
│  ┌─────────────────────────────────┐                                    │
│  │    WAITING-PR-MERGED            │  PR merged to main                 │
│  └──────┬──────────────────────────┘                                    │
│         │                                                                │
│         ▼                                                                │
│  ┌─────────────┐                                                        │
│  │  COMPLETED  │  Workflow finished successfully                        │
│  └─────────────┘                                                        │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Proposed Architecture

### New Module: `crates/healer/src/play/`

```
play/
├── mod.rs              # Public API
├── spec.rs             # PlaySpec - the expected workflow definition
├── state.rs            # PlayState - current execution state
├── tracker.rs          # PlayTracker - reconciles spec vs state
├── metrics.rs          # Metrics collection and export
└── transitions.rs      # Valid stage transitions and guards
```

### Core Types

```rust
// play/spec.rs - What SHOULD happen

/// The canonical definition of a play workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaySpec {
    /// Ordered list of stages
    pub stages: Vec<StageSpec>,
    /// Agent assignments per stage
    pub agents: HashMap<Stage, AgentSpec>,
    /// Expected timeouts per stage
    pub timeouts: HashMap<Stage, Duration>,
    /// Required artifacts per stage
    pub artifacts: HashMap<Stage, Vec<ArtifactSpec>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageSpec {
    pub name: Stage,
    pub description: String,
    pub agent: Option<String>,
    pub timeout: Duration,
    pub required_inputs: Vec<String>,
    pub expected_outputs: Vec<String>,
    pub success_criteria: Vec<Criterion>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Stage {
    Pending,
    ImplementationInProgress,
    QualityInProgress,
    SecurityInProgress,
    TestingInProgress,
    WaitingAtlasIntegration,
    WaitingPrMerged,
    Completed,
    Failed,
}

// play/state.rs - What DID happen

/// Live execution state of a play workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayState {
    /// Workflow identifier
    pub workflow_name: String,
    pub task_id: String,
    pub repository: String,
    
    /// Current stage
    pub current_stage: Stage,
    pub stage_started_at: DateTime<Utc>,
    
    /// History of what happened
    pub history: Vec<StageExecution>,
    
    /// Current agent execution (if any)
    pub active_coderun: Option<CodeRunState>,
    
    /// PR state (once created)
    pub pr_state: Option<PrState>,
    
    /// Detected issues
    pub issues: Vec<DetectedIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageExecution {
    pub stage: Stage,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub outcome: StageOutcome,
    pub agent: Option<String>,
    pub coderun_name: Option<String>,
    pub artifacts: Vec<Artifact>,
    pub metrics: StageMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StageOutcome {
    Success,
    Failed { reason: String },
    Skipped { reason: String },
    TimedOut,
    InProgress,
}

// play/tracker.rs - What NEEDS to happen

/// Reconciles PlaySpec with PlayState to determine actions
pub struct PlayTracker {
    spec: PlaySpec,
    state: PlayState,
}

impl PlayTracker {
    /// Get current deviation from expected state
    pub fn deviation(&self) -> Option<Deviation> { ... }
    
    /// Get recommended next action
    pub fn next_action(&self) -> Option<RemediationAction> { ... }
    
    /// Check if workflow is on track
    pub fn is_healthy(&self) -> bool { ... }
    
    /// Get completion percentage
    pub fn progress(&self) -> f64 { ... }
    
    /// Time remaining before timeout
    pub fn time_remaining(&self) -> Option<Duration> { ... }
}

#[derive(Debug, Clone)]
pub enum Deviation {
    /// Stage took longer than expected
    StageTimeout { stage: Stage, elapsed: Duration, limit: Duration },
    /// Agent failed
    AgentFailed { agent: String, stage: Stage, reason: String },
    /// Unexpected stage transition
    InvalidTransition { from: Stage, to: Stage },
    /// Missing expected artifact
    MissingArtifact { stage: Stage, artifact: String },
    /// Stage stuck (no progress)
    StageStuck { stage: Stage, since: Duration },
}
```

---

## Metrics Collection

### Key Metrics to Track

| Metric | Description | Source |
|--------|-------------|--------|
| `play_stage_duration_seconds` | Time spent in each stage | Workflow timestamps |
| `play_agent_retries_total` | Number of retries per agent | CodeRun status |
| `play_completion_rate` | % of workflows that complete | Historical data |
| `play_stage_success_rate` | Success rate per stage | Stage outcomes |
| `play_time_to_completion` | Total workflow duration | Start to merge |
| `play_deviations_total` | Count of deviations detected | Tracker |
| `play_remediations_total` | Count of remediations triggered | Healer |

### Prometheus Integration

```rust
// play/metrics.rs

use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramVec,
    register_counter_vec, register_gauge_vec, register_histogram_vec,
};

lazy_static! {
    pub static ref STAGE_DURATION: HistogramVec = register_histogram_vec!(
        "healer_play_stage_duration_seconds",
        "Duration of each play workflow stage",
        &["stage", "agent", "task_id"],
        vec![30.0, 60.0, 120.0, 300.0, 600.0, 1200.0, 1800.0, 3600.0]
    ).unwrap();
    
    pub static ref ACTIVE_WORKFLOWS: GaugeVec = register_gauge_vec!(
        "healer_play_active_workflows",
        "Number of active play workflows by stage",
        &["stage"]
    ).unwrap();
    
    pub static ref DEVIATIONS: CounterVec = register_counter_vec!(
        "healer_play_deviations_total",
        "Total deviations detected by type",
        &["deviation_type", "stage"]
    ).unwrap();
}
```

---

## State Synchronization

### Sources of Truth

| Data | Primary Source | Backup Source |
|------|----------------|---------------|
| Workflow stage | K8s ConfigMap (`play-task-{id}`) | Workflow labels |
| Agent status | CodeRun CRD | Pod status |
| PR state | GitHub API | Workflow outputs |
| Logs | Loki | kubectl logs |
| Metrics | Prometheus | Calculated from events |

### Sync Strategy

```rust
impl PlayTracker {
    /// Sync state from all sources
    pub async fn sync(&mut self, ctx: &SyncContext) -> Result<()> {
        // 1. Get workflow/configmap state from K8s
        let k8s_state = ctx.k8s.get_play_state(&self.state.task_id).await?;
        
        // 2. Get active CodeRun if any
        let coderun = ctx.k8s.get_active_coderun(&self.state.task_id).await?;
        
        // 3. Get PR state from GitHub (if PR exists)
        if let Some(pr_number) = self.state.pr_state.as_ref().map(|p| p.number) {
            let pr = ctx.github.get_pr_state(pr_number).await?;
            self.state.pr_state = Some(pr);
        }
        
        // 4. Check for alerts/issues from Loki
        let issues = ctx.loki.query_issues(&self.state.workflow_name).await?;
        
        // 5. Reconcile and update
        self.reconcile(k8s_state, coderun, issues)?;
        
        Ok(())
    }
}
```

---

## CLI Commands

```bash
# Show current play state
healer play status --task-id 4

# Show play history/timeline
healer play history --task-id 4

# Show what should happen vs what happened
healer play diff --task-id 4

# Get metrics for a play run
healer play metrics --task-id 4

# Force advance to next stage (admin override)
healer play advance --task-id 4 --stage quality-in-progress

# Reset a stuck play
healer play reset --task-id 4 --to-stage implementation
```

---

## Integration with Existing Alerts

Map existing alert types to play stages:

| Alert | Relevant Stage | Deviation Type |
|-------|----------------|----------------|
| A2 (Silent Failure) | Any | `AgentFailed` |
| A3 (Stale Progress) | Any | `StageStuck` |
| A7 (Pod Failure) | Any | `AgentFailed` |
| A8 (Step Timeout) | Any | `StageTimeout` |
| A9 (Stuck CodeRun) | Any | `StageStuck` |

---

## Dashboard View

```
╔══════════════════════════════════════════════════════════════════════════╗
║                     PLAY WORKFLOW: task-4                                 ║
╠══════════════════════════════════════════════════════════════════════════╣
║                                                                           ║
║  Repository: 5dlabs/cto-parallel-test                                    ║
║  PR: #42 (open)                                                          ║
║  Started: 2024-12-04 10:30:00 UTC                                        ║
║  Elapsed: 45m 23s                                                        ║
║                                                                           ║
║  Progress: ██████████████████░░░░░░░░░░░░ 60%                            ║
║                                                                           ║
║  ┌───────────────────────────────────────────────────────────────────┐   ║
║  │ Stage                    │ Status  │ Duration │ Agent │ Retries  │   ║
║  ├───────────────────────────────────────────────────────────────────┤   ║
║  │ ✅ Implementation        │ Done    │ 12m 34s  │ Rex   │ 0        │   ║
║  │ ✅ Quality               │ Done    │ 8m 12s   │ Cleo  │ 1        │   ║
║  │ 🔄 Security              │ Active  │ 5m 02s   │Cipher │ 0        │   ║
║  │ ⏳ Testing               │ Pending │ -        │ Tess  │ -        │   ║
║  │ ⏳ Atlas Integration     │ Pending │ -        │ Atlas │ -        │   ║
║  │ ⏳ Merge                 │ Pending │ -        │ -     │ -        │   ║
║  └───────────────────────────────────────────────────────────────────┘   ║
║                                                                           ║
║  Health: 🟢 ON TRACK                                                     ║
║  Est. Completion: ~25 minutes                                            ║
║                                                                           ║
╚══════════════════════════════════════════════════════════════════════════╝
```

---

## Implementation Plan

### Phase 1: Core Data Structures (MVP)
- [ ] Define `PlaySpec` and `Stage` enum
- [ ] Define `PlayState` and tracking types
- [ ] Implement basic state loading from K8s ConfigMap
- [ ] Add `healer play status` CLI command

### Phase 2: State Synchronization
- [ ] Implement real-time K8s watch for workflow state
- [ ] Add GitHub PR state polling
- [ ] Integrate Loki for issue detection
- [ ] Build `PlayTracker` reconciliation logic

### Phase 3: Metrics & Observability
- [ ] Export Prometheus metrics
- [ ] Build Grafana dashboard
- [ ] Add historical tracking
- [ ] Implement `healer play metrics` command

### Phase 4: Remediation Integration
- [ ] Connect deviation detection to alert system
- [ ] Auto-trigger remediation on deviations
- [ ] Add admin override commands
- [ ] Build recovery playbooks

---

## Open Questions

1. **Persistence**: Should play state be persisted beyond K8s ConfigMaps? (Database? CRD?)
2. **Multi-task**: How to track parallel task execution in project workflows?
3. **Timeouts**: What are reasonable default timeouts per stage?
4. **Blast radius**: Should Healer auto-cancel stuck workflows after N failures?

---

*Last updated: December 2024*

