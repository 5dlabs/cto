# Healer: Play Orchestration Design

> **Primary Goal**: Ensure play workflows complete from beginning to end by tracking what *should* happen, what *did* happen, and what *needs* to happen — then **fixing the code** when things go wrong.

## Executive Summary

Healer is not just an ops watchdog — it's an **intelligent agent** that:
1. **Tracks** workflow execution across parallel tasks in real-time
2. **Detects** deviations from expected behavior
3. **Diagnoses** root causes by analyzing logs, code, and agent behavior
4. **Fixes** issues by writing code (not just restarting pods)
5. **Optimizes** by gathering intelligence to improve agent prompts

### Key Principles

- **Code-First Remediation**: When something fails, Healer investigates and submits a fix, not just a restart
- **Parallel-Native**: Always assumes parallel task execution; tracks both batch and individual tasks
- **No History Cruft**: Purge state after runs; focus on getting to first successful completion
- **30-Minute Rule**: Any stage taking >30 minutes is suspicious and warrants investigation

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
├── stage.rs            # Stage enum and transitions
├── batch.rs            # PlayBatch - parallel task tracking
├── task.rs             # TaskState - individual task tracking
├── tracker.rs          # PlayTracker - health checks and remediation
├── remediate.rs        # Code-based fix generation
├── insights.rs         # Intelligence gathering for optimization
└── cleanup.rs          # State purging after runs
```

### Core Types

```rust
// play/spec.rs - What SHOULD happen

/// Stage timeout - 30 minutes is the target, anything longer is suspicious
pub const STAGE_TIMEOUT: Duration = Duration::from_secs(30 * 60);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Stage {
    Pending,
    ImplementationInProgress,  // Rex
    QualityInProgress,         // Cleo
    SecurityInProgress,        // Cipher
    TestingInProgress,         // Tess
    WaitingAtlasIntegration,   // Atlas
    WaitingPrMerged,
    Completed,
    Failed,
}

impl Stage {
    pub fn agent(&self) -> Option<&'static str> {
        match self {
            Self::ImplementationInProgress => Some("Rex"),
            Self::QualityInProgress => Some("Cleo"),
            Self::SecurityInProgress => Some("Cipher"),
            Self::TestingInProgress => Some("Tess"),
            Self::WaitingAtlasIntegration => Some("Atlas"),
            _ => None,
        }
    }
    
    pub fn timeout(&self) -> Duration {
        STAGE_TIMEOUT // 30 minutes for all stages
    }
}

// play/batch.rs - Parallel task tracking

/// A batch of parallel tasks being executed
#[derive(Debug, Clone)]
pub struct PlayBatch {
    /// Project-level identifier
    pub project_name: String,
    pub repository: String,
    
    /// All tasks in the batch
    pub tasks: Vec<TaskState>,
    
    /// Batch-level status
    pub started_at: DateTime<Utc>,
    pub status: BatchStatus,
}

#[derive(Debug, Clone)]
pub enum BatchStatus {
    InProgress { completed: usize, total: usize },
    Completed,
    Failed { failed_tasks: Vec<String> },
}

impl PlayBatch {
    /// Get overall progress as percentage
    pub fn progress(&self) -> f64 {
        let completed = self.tasks.iter()
            .filter(|t| matches!(t.status, TaskStatus::Completed))
            .count();
        (completed as f64 / self.tasks.len() as f64) * 100.0
    }
    
    /// Get tasks that are stuck (>30 min in current stage)
    pub fn stuck_tasks(&self) -> Vec<&TaskState> {
        let now = Utc::now();
        self.tasks.iter()
            .filter(|t| {
                if let TaskStatus::InProgress { stage, stage_started } = &t.status {
                    now.signed_duration_since(*stage_started) > chrono::Duration::minutes(30)
                } else {
                    false
                }
            })
            .collect()
    }
}

/// Individual task within a batch
#[derive(Debug, Clone)]
pub struct TaskState {
    pub task_id: String,
    pub status: TaskStatus,
    pub pr_number: Option<u32>,
    pub active_coderun: Option<String>,
    pub issues: Vec<DetectedIssue>,
}

#[derive(Debug, Clone)]
pub enum TaskStatus {
    Pending,
    InProgress { 
        stage: Stage, 
        stage_started: DateTime<Utc>,
    },
    Completed,
    Failed { 
        stage: Stage, 
        reason: String,
        remediation: Option<RemediationState>,
    },
}

// play/tracker.rs - What NEEDS to happen

/// Watches a batch and triggers remediation when things go wrong
pub struct PlayTracker {
    batch: PlayBatch,
    insights: InsightCollector,
}

impl PlayTracker {
    /// Check all tasks and return any that need intervention
    pub fn check_health(&self) -> Vec<Issue> {
        let mut issues = vec![];
        
        // Check for stuck tasks (>30 min)
        for task in self.batch.stuck_tasks() {
            issues.push(Issue::StageTimeout {
                task_id: task.task_id.clone(),
                stage: task.current_stage(),
                elapsed: task.stage_duration(),
            });
        }
        
        // Check for failed tasks needing remediation
        for task in &self.batch.tasks {
            if let TaskStatus::Failed { stage, reason, remediation: None } = &task.status {
                issues.push(Issue::NeedsRemediation {
                    task_id: task.task_id.clone(),
                    stage: *stage,
                    failure_reason: reason.clone(),
                });
            }
        }
        
        issues
    }
    
    /// Spawn a code-fixing remediation for an issue
    pub async fn remediate(&self, issue: &Issue) -> Result<RemediationState> {
        // 1. Gather context (logs, code, agent output)
        let context = self.gather_context(issue).await?;
        
        // 2. Diagnose root cause
        let diagnosis = self.diagnose(&context).await?;
        
        // 3. Spawn Healer CodeRun to fix the code
        let coderun = self.spawn_fix_coderun(&diagnosis).await?;
        
        Ok(RemediationState {
            coderun_name: coderun.name,
            diagnosis: diagnosis.summary,
            started_at: Utc::now(),
        })
    }
}

#[derive(Debug, Clone)]
pub enum Issue {
    /// Task stuck in stage for >30 minutes
    StageTimeout { 
        task_id: String, 
        stage: Stage, 
        elapsed: Duration,
    },
    /// Task failed and needs code fix
    NeedsRemediation { 
        task_id: String, 
        stage: Stage, 
        failure_reason: String,
    },
    /// Agent behaving suboptimally (optimization opportunity)
    OptimizationOpportunity {
        task_id: String,
        agent: String,
        observation: String,
        suggested_prompt_change: String,
    },
}

// play/insights.rs - Intelligence gathering for optimization

/// Collects observations about agent behavior for prompt optimization
pub struct InsightCollector {
    observations: Vec<AgentObservation>,
}

#[derive(Debug, Clone)]
pub struct AgentObservation {
    pub agent: String,
    pub task_id: String,
    pub timestamp: DateTime<Utc>,
    pub observation_type: ObservationType,
    pub details: String,
}

#[derive(Debug, Clone)]
pub enum ObservationType {
    /// Agent took an inefficient path
    InefficiencyDetected,
    /// Agent made a common mistake
    RepeatedMistake,
    /// Agent succeeded with a good pattern
    SuccessPattern,
    /// Agent used excessive retries
    ExcessiveRetries,
    /// Agent completed faster than expected
    FastCompletion,
}

impl InsightCollector {
    /// Analyze observations and suggest prompt improvements
    pub fn suggest_optimizations(&self) -> Vec<PromptSuggestion> {
        // Group observations by agent and pattern
        // Identify recurring issues
        // Generate suggestions for prompt improvements
        todo!()
    }
}
```

---

## CLI Commands

### Batch & Task Status

```bash
# Show all active tasks in the batch (primary view)
healer play status
# Output: Table of all tasks with stage, duration, health

# Show detailed status for a specific task
healer play status --task-id 5

# Show only stuck/unhealthy tasks
healer play status --stuck
```

### Remediation

```bash
# Manually trigger remediation for a stuck task
healer play remediate --task-id 5

# Show active remediations
healer play remediations

# Cancel a remediation
healer play cancel-remediation --task-id 5
```

### Cleanup

```bash
# Purge all state for completed/failed batch
healer play cleanup

# Force cleanup (even if tasks still running)
healer play cleanup --force
```

### Insights & Optimization

```bash
# Show agent performance insights
healer insights show --agent rex

# Show optimization suggestions
healer insights suggest

# Show common failure patterns
healer insights failures

# Export insights for prompt tuning
healer insights export --format json > insights.json
```

---

---

## Dashboard View

### Batch Overview (Parallel Tasks)

```
╔══════════════════════════════════════════════════════════════════════════╗
║                     PLAY BATCH: my-project                                ║
╠══════════════════════════════════════════════════════════════════════════╣
║                                                                           ║
║  Repository: 5dlabs/cto-parallel-test                                    ║
║  Started: 2024-12-04 10:30:00 UTC                                        ║
║  Elapsed: 1h 15m                                                         ║
║                                                                           ║
║  Batch Progress: ████████████████░░░░░░░░░░░░░░ 53% (8/15 tasks)         ║
║                                                                           ║
║  ┌───────────────────────────────────────────────────────────────────┐   ║
║  │ Task │ Stage           │ Status    │ Duration │ PR    │ Health   │   ║
║  ├───────────────────────────────────────────────────────────────────┤   ║
║  │ 1    │ Completed       │ ✅ Done   │ 42m      │ #101  │ 🟢       │   ║
║  │ 2    │ Completed       │ ✅ Done   │ 38m      │ #102  │ 🟢       │   ║
║  │ 3    │ Testing         │ 🔄 Active │ 18m      │ #103  │ 🟢       │   ║
║  │ 4    │ Quality         │ 🔄 Active │ 12m      │ #104  │ 🟢       │   ║
║  │ 5    │ Implementation  │ ⚠️ STUCK  │ 35m      │ -     │ 🔴       │   ║
║  │ 6    │ Security        │ 🔄 Active │ 8m       │ #106  │ 🟢       │   ║
║  │ 7    │ Implementation  │ 🔄 Active │ 22m      │ -     │ 🟢       │   ║
║  │ 8    │ Pending         │ ⏳ Queue  │ -        │ -     │ ⚪       │   ║
║  │ ...  │                 │           │          │       │          │   ║
║  └───────────────────────────────────────────────────────────────────┘   ║
║                                                                           ║
║  🔴 ALERT: Task 5 stuck in Implementation for 35m (>30m threshold)       ║
║     └─ Healer remediation spawned: healer-fix-task5-abc123              ║
║                                                                           ║
╚══════════════════════════════════════════════════════════════════════════╝
```

### Single Task Detail

```
╔══════════════════════════════════════════════════════════════════════════╗
║                     TASK 5 DETAIL                                         ║
╠══════════════════════════════════════════════════════════════════════════╣
║                                                                           ║
║  Stage: Implementation (Rex)                                             ║
║  Duration: 35m 12s  ⚠️ OVER THRESHOLD                                    ║
║  CodeRun: play-task-5-rex-abc123                                         ║
║                                                                           ║
║  Issue Detected:                                                         ║
║  ├─ Type: StageTimeout                                                   ║
║  ├─ Agent: Rex                                                           ║
║  └─ Last Activity: "Attempting to resolve merge conflict" (12m ago)      ║
║                                                                           ║
║  Remediation:                                                            ║
║  ├─ Status: In Progress                                                  ║
║  ├─ Healer CodeRun: healer-fix-task5-abc123                             ║
║  ├─ Diagnosis: "Git conflict on src/lib.rs - Rex missing rebase step"   ║
║  └─ Action: "Writing fix to add pre-commit rebase check"                 ║
║                                                                           ║
╚══════════════════════════════════════════════════════════════════════════╝
```

---

---

## Remediation Philosophy

### Code-First, Not Ops-First

When Healer detects an issue, it **does not**:
- ❌ Just restart the pod
- ❌ Just scale up resources
- ❌ Just retry the same thing

Instead, Healer:
- ✅ Investigates logs and agent output
- ✅ Identifies the root cause in code/prompts
- ✅ Writes a fix and submits a PR
- ✅ Learns from the failure to prevent recurrence

### Remediation Flow

```
┌──────────────────────────────────────────────────────────────────────────┐
│                      HEALER REMEDIATION FLOW                             │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  1. DETECT                                                               │
│     └─ Task stuck >30 min or agent failed                               │
│                                                                          │
│  2. GATHER CONTEXT                                                       │
│     ├─ Loki logs from failed pod                                        │
│     ├─ Agent's last actions/output                                      │
│     ├─ Relevant source code                                             │
│     └─ PR state and CI results                                          │
│                                                                          │
│  3. DIAGNOSE                                                             │
│     ├─ What was the agent trying to do?                                 │
│     ├─ What went wrong?                                                 │
│     └─ Is this a code issue, prompt issue, or infra issue?              │
│                                                                          │
│  4. FIX                                                                  │
│     ├─ Spawn Healer CodeRun with diagnosis                              │
│     ├─ Healer writes fix (code, prompt, config)                         │
│     └─ Submit PR for the fix                                            │
│                                                                          │
│  5. LEARN                                                                │
│     ├─ Record observation for optimization                              │
│     ├─ Update prompt suggestions if pattern detected                    │
│     └─ Purge ephemeral state after run completes                        │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## Prompt Optimization

Healer's secondary role is to **gather intelligence** for improving agent prompts.

### What Healer Observes

| Observation | Example | Action |
|-------------|---------|--------|
| Repeated mistake | Rex always forgets to run tests | Suggest adding "always run tests" to prompt |
| Inefficient path | Cleo reviews files that didn't change | Suggest scoping review to diff |
| Excessive retries | Tess retries 5 times on same error | Investigate flaky test patterns |
| Fast completion | Cipher finishes in 2 min | Record as success pattern |
| Common failure mode | Git auth errors 30% of the time | Investigate infra, update error handling |

### Optimization Output

```bash
# After analyzing multiple runs, Healer can suggest:
healer insights suggest --agent rex

📊 Agent: Rex
   Runs Analyzed: 47
   Success Rate: 72%
   Avg Duration: 18m

🔍 Observations:
   • 12 runs failed due to missing imports (suggest: add import verification step)
   • 8 runs had git conflicts (suggest: always pull before push)
   • 5 runs exceeded timeout (suggest: break into smaller commits)

💡 Suggested Prompt Changes:
   1. Add: "Before committing, verify all imports resolve"
   2. Add: "Pull latest changes before pushing to avoid conflicts"
   3. Add: "Make atomic commits - one logical change per commit"
```

---

## Implementation Plan (Revised)

### Phase 1: Batch Tracking (This Sprint)
- [ ] Define `PlayBatch` and `TaskState` types
- [ ] Implement batch state loading from K8s (list all play-task-* ConfigMaps)
- [ ] Add `healer play status` CLI showing batch + per-task status
- [ ] Detect stuck tasks (>30 min in stage)

### Phase 2: Code-Based Remediation
- [ ] Implement context gathering (Loki logs, CodeRun output)
- [ ] Build diagnosis prompt template
- [ ] Spawn Healer CodeRun for fixes (not just alerts)
- [ ] Track remediation state per task

### Phase 3: Optimization Intelligence
- [ ] Implement `InsightCollector` 
- [ ] Record observations during runs
- [ ] Build pattern detection for common issues
- [ ] Add `healer insights` CLI commands

### Phase 4: Continuous Improvement Loop
- [ ] Auto-generate prompt improvement suggestions
- [ ] Track success rates over time
- [ ] Build optimization dashboard
- [ ] Automated prompt A/B testing (stretch goal)

---

## State Management

### Ephemeral by Design

- **No persistent history** — purge after each run
- **ConfigMaps as primary state** — already exists, no new storage needed
- **Focus on NOW** — what's running, what's stuck, what needs fixing

### Cleanup Policy

```rust
impl PlayBatch {
    /// Purge all state after successful completion
    pub async fn cleanup(&self) -> Result<()> {
        for task in &self.tasks {
            // Delete play-task-{id} ConfigMap
            // Delete any remediation state
            // Keep only: final metrics for this run
        }
        Ok(())
    }
}
```

---

*Last updated: December 2024*

