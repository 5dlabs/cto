# Healer: Current Functionality

> **Self-healing platform monitor** — detects issues and spawns remediation agents

Healer is the CTO platform's operational watchdog. It monitors Kubernetes resources, detects anomalies during workflow execution, and can automatically spawn AI agents to investigate and fix problems.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         HEALER BINARY                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐      │
│  │   kubectl    │    │   GitHub     │    │    Loki      │      │
│  │   --watch    │    │   Polling    │    │   Client     │      │
│  └──────┬───────┘    └──────┬───────┘    └──────┬───────┘      │
│         │                   │                   │               │
│         ▼                   ▼                   ▼               │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    ALERT REGISTRY                        │   │
│  │  ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐│   │
│  │  │ A1 │ │ A2 │ │ A3 │ │ A4 │ │ A5 │ │ A7 │ │ A8 │ │ A9 ││   │
│  │  └────┘ └────┘ └────┘ └────┘ └────┘ └────┘ └────┘ └────┘│   │
│  └─────────────────────────────────────────────────────────┘   │
│         │                                                       │
│         ▼                                                       │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐      │
│  │   Dedup      │───▶│   Template   │───▶│   CodeRun    │      │
│  │   Engine     │    │   Engine     │    │   Spawner    │      │
│  └──────────────┘    └──────────────┘    └──────────────┘      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Alert Types

Healer implements a **reactive alert system** that evaluates events from Kubernetes watches and GitHub polling.

| ID | Name | Description | Severity |
|----|------|-------------|----------|
| **A1** | Comment Order Mismatch | GitHub comment history doesn't match expected K8s agent sequence | Warning |
| **A2** | Silent Agent Failure | Container died but pod still shows "Running" (sidecar keeps it alive) | Critical |
| **A3** | Stale Progress | No commits for configurable threshold duration | Warning |
| **A4** | Repeated Approval Loop | Same agent approving multiple times in a row | Warning |
| **A5** | Post-Tess CI/Merge Failure | CI failing or merge conflict after Tess approval | Warning |
| **A7** | Pod Failure | Any CTO pod in Failed/Error/CrashLoopBackOff state | Warning/Critical |
| **A8** | Workflow Step Timeout | Step running longer than agent-specific threshold | Warning |
| **A9** | Stuck CodeRun | CodeRun in non-terminal state beyond threshold (10 min default) | Warning |
| **Completion** | Success Check | Validates successful workflow completion | Info |

### Alert Detection Methods (A2 Example)

The A2 Silent Failure alert uses **multi-method detection with priority-based deduplication**:

1. **Exit Code** (Priority 1) — Container terminated with non-zero exit
2. **Ready Status** (Priority 2) — Container `ready=false` while pod Running
3. **Pod Conditions** (Priority 3) — `ContainersReady=False` condition
4. **Restart Count** (Priority 4) — High restart count (≥3) indicates crash loop
5. **Terminated Duration** (Priority 5) — Container dead for extended period (>60s)

## CLI Commands

### Primary Workflow Commands

```bash
# Full E2E loop: start play, monitor all resources until completion
healer full --task-id 4 --config cto-config.json --self-healing

# Watch an existing play workflow with real-time kubectl streams
healer watch --task-id 4 --repository 5dlabs/cto-parallel-test

# Run/submit a play workflow via Argo CLI
healer run --task-id 4 --config cto-config.json
```

### Monitoring & Status Commands

```bash
# Get current status of a workflow (single check)
healer status --play-id play-task-4

# Get logs for a specific workflow step or pod
healer logs --play-id play-task-4 --step rex-step --tail 500

# Legacy polling-based monitor
healer loop --play-id play-task-4 --interval 10
```

### Alert & Remediation Commands

```bash
# Watch for platform alerts and spawn Factory on detection
healer alert-watch --namespace cto --prompts-dir monitor/prompts

# Test an alert flow manually (dry-run)
healer test-alert --alert a7 --pod-name test-pod --dry-run

# Spawn a remediation agent for a detected issue
healer spawn-remediation --alert a7 --task-id 123 --issue-number 456 --target-pod failed-pod
```

### Utility Commands

```bash
# Reset environment: clean cluster resources and reset test repo
healer reset --repo cto-parallel-test --org 5dlabs --force

# Fetch all logs for a pod (current, previous, events, describe)
healer fetch-logs --pod-name failed-pod-123 --namespace cto

# Memory commands (OpenMemory integration)
healer memory list --agent rex --limit 20
healer memory query --text "git authentication errors" --agent rex
healer memory stats --agent tess
```

## Core Modules

### `alerts/` — Alert Detection System

- **`mod.rs`** — `AlertRegistry` that evaluates all handlers against events
- **`types.rs`** — Core types: `Alert`, `AlertId`, `Severity`, `AlertContext`, `AlertHandler` trait
- **`a1_comment_order.rs`** — GitHub vs K8s comment sequence validation
- **`a2_silent_failure.rs`** — Multi-method silent container failure detection
- **`a3_stale_progress.rs`** — Stale commit detection with configurable threshold
- **`a4_approval_loop.rs`** — Repeated approval detection
- **`a5_post_tess_ci.rs`** — Post-QA CI/merge failure detection
- **`a7_pod_failure.rs`** — Pod failure/CrashLoopBackOff detection
- **`a8_step_timeout.rs`** — Agent-specific step timeout detection
- **`a9_stuck_coderun.rs`** — Stuck CodeRun phase transition detection

### `k8s.rs` — Kubernetes Types & Events

Defines event types from kubectl watches:
- `PodRunning`, `PodModified`, `PodSucceeded`, `PodFailed`
- `WorkflowPhaseChanged`
- `CodeRunChanged`
- `GitHubUpdate`

Excludes infrastructure pods from alerts:
```rust
const EXCLUDED_POD_PREFIXES: &[&str] = &[
    "healer", "cto-tools", "cto-controller", "vault-mcp-server",
    "openmemory", "event-cleaner", "workspace-pvc-cleaner",
    "tweakcn", "atlas-guardian", "atlas-batch-integration",
];
```

### `github.rs` — GitHub State Types

Types for PR state polling via `gh pr view --json`:
- `GitHubState` — PR number, comments, commits, checks, reviews, labels, mergeable status
- `Comment`, `Commit`, `Check`, `Review` — Individual PR components
- `CheckConclusion`, `ReviewState` — Enums for CI and review states

### `loki.rs` — Log Querying

Client for querying Grafana Loki for historical pod logs:

```rust
let client = LokiClient::with_defaults();

// Query logs for a specific pod
let logs = client.query_pod_logs("cto", "failed-pod", start, end, 1000).await?;

// Query logs for an entire workflow
let logs = client.query_workflow_logs("cto", "play-task-4", start, end, 0).await?;

// Query logs around a failure time
let logs = client.query_logs_around_failure("cto", "pod", failure_time, 5, 2).await?;
```

Utility functions:
- `format_logs_for_issue()` — Format logs for GitHub issues
- `extract_error_lines()` — Filter error/fatal/panic lines

### `dedup.rs` — Deduplication Engine

Prevents duplicate remediation CodeRuns and GitHub issues:

- **Workflow Family Extraction** — Groups pods from same workflow
  - `play-task-4-abc-step-123` → `play-task-4`
  - `atlas-conflict-monitor-xyz` → `atlas-conflict-monitor`
  
- **CodeRun Dedup** — `check_existing_remediation()` queries for active CodeRuns with matching labels
- **GitHub Issue Dedup** — `check_recent_alert_type_issue()` prevents spam within 30-minute window
- **Pod Exclusion** — Pods with `healer.platform/exclude=true` label are skipped

### `templates.rs` — Handlebars Template Engine

Renders remediation prompts from templates:

```rust
let engine = TemplateEngine::new("prompts")?;
let prompt = engine.render_alert("a7", &AlertContext {
    alert_id: "A7".into(),
    pod_name: "failed-pod".into(),
    namespace: "cto".into(),
    logs: "... error logs ...".into(),
    // ...
})?;
```

Custom helpers:
- `{{concat "prefix" variable "suffix"}}` — String concatenation
- `{{lowercase alert_id}}` — Case conversion

## Prompts Directory Structure

```
prompts/
├── alerts/                    # Alert-specific templates
│   ├── a1-comment-order.hbs
│   ├── a2-silent-failure.hbs
│   ├── a3-stale-progress.hbs
│   ├── a4-approval-loop.hbs
│   ├── a5-post-tess-ci.hbs
│   ├── a7-pod-failure.hbs
│   ├── a8-step-timeout.hbs
│   ├── a9-stuck-coderun.hbs
│   └── success-completion.hbs
├── expected/                  # Expected agent behaviors
│   ├── atlas.md
│   ├── blaze.md
│   ├── cipher.md
│   ├── cleo.md
│   ├── rex.md
│   └── tess.md
├── partials/                  # Shared template components
│   ├── _acceptance-criteria.hbs
│   ├── _base.hbs
│   ├── _issue-create.hbs
│   ├── _issue-folder.hbs
│   ├── _issue-update.hbs
│   ├── _merge-conflict-resolution.hbs
│   ├── _preamble.hbs
│   └── _spawn-remediation.hbs
└── test-scenarios/            # Test scenario definitions
    └── introduce-rex-git-bug.md
```

## Configuration

### `heal-config.json`

```json
{
  "coderun": {
    "namespace": "cto",
    "githubApp": "5DLabs-Rex",
    "model": "claude-opus-4-5-20251101",
    "repositoryUrl": "https://github.com/5dlabs/cto",
    "service": "heal",
    "runType": "implementation",
    "enableDocker": true,
    "remoteTools": "mcp_tools_github_*,mcp_tools_kubernetes_*,mcp_tools_argocd_*...",
    "cliConfig": {
      "cliType": "claude",
      "model": "claude-opus-4-5-20251101",
      "settings": { "template": "heal/claude" }
    }
  }
}
```

### Alert Configuration

```rust
AlertConfig {
    stale_progress_threshold_mins: 15,     // A3
    approval_loop_threshold: 2,             // A4
    stuck_coderun_threshold_mins: 10,       // A9
    step_timeouts: StepTimeouts {
        implementation_mins: 45,  // Rex/Blaze
        quality_mins: 15,         // Cleo
        testing_mins: 30,         // Tess
        security_mins: 15,        // Cipher
        integration_mins: 20,     // Atlas
        default_mins: 60,
    },
}
```

## Self-Healing Loop

When `--self-healing` is enabled, Healer runs a continuous feedback loop:

```
┌──────────────────────────────────────────────────────────────┐
│                     SELF-HEALING LOOP                        │
│                                                              │
│   1. Start Play Workflow                                     │
│           │                                                  │
│           ▼                                                  │
│   2. Monitor via kubectl --watch + GitHub polling            │
│           │                                                  │
│           ▼                                                  │
│   3. Alert Detected? ──No──▶ Continue monitoring             │
│           │                                                  │
│          Yes                                                 │
│           │                                                  │
│           ▼                                                  │
│   4. Dedup Check (existing CodeRun/Issue?)                   │
│           │                                                  │
│           ▼                                                  │
│   5. Fetch Logs (Loki + kubectl)                            │
│           │                                                  │
│           ▼                                                  │
│   6. Create GitHub Issue                                     │
│           │                                                  │
│           ▼                                                  │
│   7. Spawn Remediation CodeRun                               │
│           │                                                  │
│           ▼                                                  │
│   8. Monitor Remediation → Loop back to step 2               │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

## OpenMemory Integration

Healer can query OpenMemory for agent insights:

```bash
# List recent memories for an agent
healer memory list --agent rex --limit 20

# Semantic search across memories
healer memory query --text "git authentication errors" --agent rex

# Get statistics and health
healer memory stats --agent tess
```

## Dependencies

- **CLI**: `clap`, `colored`
- **Async**: `tokio`
- **HTTP**: `reqwest` (Loki queries)
- **Serialization**: `serde`, `serde_json`
- **Error Handling**: `anyhow`, `thiserror`
- **Time**: `chrono`
- **Templates**: `handlebars`
- **Logging**: `tracing`, `tracing-subscriber`
- **Notifications**: Internal `notify` crate

## Current Limitations

1. **Reactive Only** — Currently responds to problems after they occur, no proactive monitoring
2. **Single Namespace** — Watches one namespace at a time
3. **Limited Root Cause Analysis** — Detects symptoms, limited correlation between alerts
4. **No Learning** — Doesn't learn from past remediations to prevent future issues
5. **Manual Configuration** — Alert thresholds are static, not adaptive
6. **No Health Dashboard** — No persistent UI for platform health overview
7. **Resource Monitoring** — No CPU/memory/disk monitoring beyond pod status

