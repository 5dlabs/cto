# Healer: Unified Remediation Hub Design

## Overview

Healer becomes the **single remediation hub** for all issues across the platform. It acts as an intelligent dispatcher with comprehensive tooling access to make informed routing decisions and provide maximum context to specialist agents.

### Key Principle

- **Stitch** = Detection (finds issues, posts review comments)
- **Healer** = Remediation (analyzes, routes, tracks, enriches context)
- **Specialist Agents** = Execution (Rex, Blaze, Bolt, Cipher, Atlas)

---

## Scope & Phasing

Healer operates across two distinct domains with different timelines:

### Domain 1: CTO Platform CI (Current Focus)

| Attribute | Value |
|-----------|-------|
| **Repository** | `5dlabs/cto` |
| **Scope** | CI remediation for the CTO platform itself |
| **Failures** | Rust builds, Docker images, Helm charts, GitHub Actions |
| **Status** | Implementing now |

This phase focuses on getting the CTO platform's own CI green without human intervention.

### Domain 2: CTO-Managed Applications (Future)

| Attribute | Value |
|-----------|-------|
| **Repository** | `5dlabs/cto-apps` |
| **Scope** | Deployed applications created by CTO |
| **Failures** | App health, sync failures, resource issues, runtime errors |
| **Status** | After CTO platform is stable |

The [`cto-apps`](https://github.com/5dlabs/cto-apps) repo uses an App-of-Apps pattern:

```
cto-apps/
├── app-of-apps.yaml          # Main ArgoCD app watching this repo
├── preview/                   # Preview deployments (task-{id}-preview)
├── production/                # Production deployments (task-{id}-prod)
└── templates/                 # Templates for Bolt to use
```

**Domain 2 will require:**
- ArgoCD application health monitoring (not just CI)
- Runtime log analysis from deployed pods
- Multi-repository awareness (cto vs cto-apps)
- Bolt integration for deployment fixes

**Note:** This design document focuses on Domain 1. Domain 2 will be designed separately once the CTO platform reaches stability.

---

## Current State vs Desired State

### Current State Overview

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                            CURRENT STATE                                      │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│  DETECTION                    REMEDIATION                   EXECUTION         │
│  ─────────                    ───────────                   ─────────         │
│                                                                               │
│  ┌─────────┐                 ┌──────────────────┐          ┌─────────┐       │
│  │ Stitch  │──PR Review───▶  │ templates/       │────────▶ │   Rex   │       │
│  │ (Review)│                 │ remediate/       │          │(direct) │       │
│  └─────────┘                 │ *.hbs            │          └─────────┘       │
│                              └──────────────────┘                             │
│                                                                               │
│  ┌─────────┐                 ┌──────────────────┐          ┌─────────┐       │
│  │ CI      │──Workflow───▶   │ ci-remediation-  │────────▶ │  Atlas  │       │
│  │ Failures│  Failure        │ sensor.yaml      │          │(direct) │       │
│  └─────────┘                 │ (creates CodeRun)│          └─────────┘       │
│                              └──────────────────┘                             │
│                                                                               │
│  ┌─────────┐                 ┌──────────────────┐          ┌─────────┐       │
│  │ Bugbot  │──Comment────▶   │ Atlas PR Guardian│────────▶ │  Atlas  │       │
│  │(Cursor) │                 │ (values.yaml)    │          │(direct) │       │
│  └─────────┘                 └──────────────────┘                             │
│                                                                               │
│  ┌─────────┐                                               ┌─────────┐       │
│  │Security │──Manual────────────────────────────────────▶  │ Manual  │       │
│  │ Alerts  │                                               │  Fixes  │       │
│  └─────────┘                                               └─────────┘       │
│                                                                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Current State Components

| Component | Location | Purpose | Issues |
|-----------|----------|---------|--------|
| **Stitch (Review)** | `templates/review/` | PR code review & bug detection | ✅ KEEP - Works well |
| **Rex Remediation** | `templates/remediate/` | Fixes Stitch findings | ❌ REMOVE - Healer takes over |
| **CI Sensor** | `infra/gitops/resources/sensors/ci-failure-remediation-sensor.yaml` | Creates Atlas CodeRun on CI failure | ❌ MODIFY - Route through Healer |
| **Atlas PR Guardian** | `infra/charts/controller/values.yaml` (atlas.guardianMode) | Bugbot resolution, merge conflicts | ❌ ABSORB into Healer routing |
| **Bugbot** | External (Cursor) | Code review comments | ❌ REPLACE with Stitch |
| **Security Remediation** | None | Manual process | ❌ ADD - Healer routes to Cipher |

### Current Flow Problems

1. **No intelligent routing** - Atlas handles all CI failures regardless of type
2. **No central tracking** - Remediation attempts are scattered, no correlation
3. **No deduplication** - Multiple failures can spawn duplicate fix attempts
4. **No learning** - Insights from fixes aren't captured or used
5. **Limited context** - Agents get minimal diagnostic information
6. **Multiple entry points** - Different templates for different sources
7. **External dependency** - Bugbot is outside our control

---

### Desired State Overview

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                            DESIRED STATE                                      │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│  DETECTION              INTELLIGENT HUB                EXECUTION              │
│  ─────────              ───────────────                ─────────              │
│                                                                               │
│  ┌─────────┐                                          ┌─────────┐            │
│  │ Stitch  │────┐                                ┌───▶│   Rex   │            │
│  │ (Review)│    │                                │    │ (Rust)  │            │
│  └─────────┘    │                                │    └─────────┘            │
│                 │                                │                            │
│  ┌─────────┐    │    ┌──────────────────────┐   │    ┌─────────┐            │
│  │   CI    │────┼───▶│                      │───┼───▶│  Blaze  │            │
│  │ Failures│    │    │        HEALER        │   │    │(Frontend)│           │
│  └─────────┘    │    │                      │   │    └─────────┘            │
│                 │    │  ┌────────────────┐  │   │                            │
│  ┌─────────┐    │    │  │   TOOLING      │  │   │    ┌─────────┐            │
│  │ Security│────┼───▶│  │ • ArgoCD API   │  │───┼───▶│  Bolt   │            │
│  │ Alerts  │    │    │  │ • Prometheus   │  │   │    │(Infra)  │            │
│  └─────────┘    │    │  │ • Loki Logs    │  │   │    └─────────┘            │
│                 │    │  │ • GitHub API   │  │   │                            │
│  ┌─────────┐    │    │  │ • Kubernetes   │  │   │    ┌─────────┐            │
│  │  Play   │────┘    │  └────────────────┘  │───┼───▶│ Cipher  │            │
│  │Workflow │         │                      │   │    │(Security)│           │
│  └─────────┘         │  • Smart Routing     │   │    └─────────┘            │
│                      │  • Deduplication     │   │                            │
│                      │  • Context Enrichment│   │    ┌─────────┐            │
│                      │  • Tracking/Insights │   └───▶│  Atlas  │            │
│                      │                      │        │(Git/GH) │            │
│                      └──────────────────────┘        └─────────┘            │
│                                                                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Desired State Components

| Component | Location | Purpose | Status |
|-----------|----------|---------|--------|
| **Stitch (Review)** | `templates/review/` | PR code review & bug detection | ✅ KEEP unchanged |
| **Healer Hub** | `crates/healer/src/ci/` | Central remediation router | 🆕 NEW |
| **Healer HTTP API** | `crates/healer/src/server.rs` | Receive events from sensors | 🆕 NEW |
| **CI Sensor** | Modified to call Healer API | Routes through Healer | 🔄 MODIFY |
| **Agent Prompts** | `crates/healer/prompts/ci/` | Agent-specific fix prompts | 🆕 NEW |

### Files to Remove

| Path | Reason |
|------|--------|
| `templates/remediate/claude/` | Healer takes over routing |
| `templates/remediate/factory/` | Healer takes over routing |
| `infra/charts/controller/agent-templates/remediate/` | Duplicated in Healer |

### Files to Modify

| Path | Change |
|------|--------|
| `infra/gitops/resources/sensors/ci-failure-remediation-sensor.yaml` | Call Healer HTTP API instead of creating CodeRun directly |
| `infra/charts/controller/values.yaml` | Remove Atlas `guardianMode` (Healer handles) |

---

## Healer Comprehensive Tooling

Healer needs access to the full platform observability stack to make intelligent routing decisions and enrich the context provided to specialist agents.

### Tool Categories

```
┌────────────────────────────────────────────────────────────────────────────┐
│                         HEALER TOOLING STACK                                │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        GitOps & Deployment                           │   │
│  │  • ArgoCD API: App status, sync state, health, resource tree        │   │
│  │  • Argo Workflows: Workflow status, logs, retry                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        Telemetry & Logs                              │   │
│  │  • Prometheus: Metrics, error rates, resource usage                 │   │
│  │  • Loki: Application logs, workflow logs, container logs            │   │
│  │  • Grafana: Dashboard queries, alert state                          │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        GitHub & Code                                 │   │
│  │  • GitHub CLI: PR state, CI checks, workflow logs, file diffs       │   │
│  │  • GitHub API: Reviews, comments, labels, commit history            │   │
│  │  • Code Analysis: Changed files, file types, patterns               │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        Kubernetes                                    │   │
│  │  • kubectl: Pod state, events, logs, ConfigMaps, CodeRuns           │   │
│  │  • K8s API: Watch resources, list deployments, check health         │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└────────────────────────────────────────────────────────────────────────────┘
```

### Tool Usage by Phase

#### Phase 1: Event Receipt & Validation

```rust
// When Healer receives a CI failure event
async fn handle_ci_failure(&self, event: CiFailureEvent) -> Result<RemediationResponse> {
    // 1. Validate event
    if !self.should_process(&event) {
        return Ok(RemediationResponse::skipped("already handled"));
    }
    
    // 2. Check for existing remediation (deduplication)
    if self.has_active_coderun_for(&event.workflow_run_id).await? {
        return Ok(RemediationResponse::skipped("existing remediation"));
    }
    
    // 3. Gather comprehensive context
    let context = self.gather_context(&event).await?;
    
    // 4. Route to appropriate agent
    let agent = self.route_to_agent(&context);
    
    // 5. Spawn enriched CodeRun
    let coderun = self.spawn_coderun(agent, &context).await?;
    
    Ok(RemediationResponse::accepted(coderun))
}
```

#### Phase 2: Context Gathering (The Intelligence)

```rust
async fn gather_context(&self, event: &CiFailureEvent) -> Result<RemediationContext> {
    // Parallel fetch from all sources
    let (
        gh_workflow_logs,
        gh_pr_state,
        gh_changed_files,
        argocd_app_status,
        loki_recent_logs,
        k8s_pod_state,
        prometheus_error_rate,
    ) = tokio::try_join!(
        // GitHub: Workflow logs, PR state, changed files
        self.github.get_workflow_logs(&event.workflow_run_id),
        self.github.get_pr_for_branch(&event.branch),
        self.github.get_changed_files(&event.head_sha),
        
        // ArgoCD: Is there an app out of sync? Health issues?
        self.argocd.get_app_status("cto-controller"),
        
        // Loki: Recent error logs from related pods
        self.loki.query_errors(&event.branch, Duration::minutes(30)),
        
        // Kubernetes: Pod state, events
        self.k8s.get_related_pods(&event.workflow_name),
        
        // Prometheus: Error rate spike? Resource issues?
        self.prometheus.query_error_rate(&event.workflow_name),
    )?;
    
    Ok(RemediationContext {
        event: event.clone(),
        workflow_logs: gh_workflow_logs,
        pr: gh_pr_state,
        changed_files: gh_changed_files,
        argocd_status: argocd_app_status,
        recent_logs: loki_recent_logs,
        pod_state: k8s_pod_state,
        metrics: prometheus_error_rate,
        failure_type: self.classify_failure(&gh_workflow_logs),
    })
}
```

#### Phase 3: Intelligent Routing

```rust
fn route_to_agent(&self, ctx: &RemediationContext) -> Agent {
    // Priority order - first match wins
    
    // 1. Security events always go to Cipher
    if ctx.is_security_event() {
        return Agent::Cipher;
    }
    
    // 2. Rust failures go to Rex
    if ctx.failure_type.is_rust() || ctx.changed_files.mostly_rust() {
        return Agent::Rex;
    }
    
    // 3. Frontend failures go to Blaze
    if ctx.failure_type.is_frontend() || ctx.changed_files.mostly_frontend() {
        return Agent::Blaze;
    }
    
    // 4. Infrastructure failures go to Bolt
    if ctx.failure_type.is_infra() || ctx.changed_files.mostly_infra() {
        return Agent::Bolt;
    }
    
    // 5. ArgoCD sync issues go to Bolt (has ArgoCD tooling)
    if ctx.argocd_status.is_out_of_sync() || ctx.argocd_status.has_health_issues() {
        return Agent::Bolt;
    }
    
    // 6. Merge conflicts go to Atlas
    if ctx.failure_type.is_merge_conflict() {
        return Agent::Atlas;
    }
    
    // 7. Default: Atlas handles everything else
    Agent::Atlas
}
```

#### Phase 4: Enriched CodeRun Creation

```rust
fn spawn_coderun(&self, agent: Agent, ctx: &RemediationContext) -> Result<CodeRun> {
    // Build comprehensive prompt with all gathered context
    let prompt = self.template_engine.render(
        &format!("ci/{}-fix.hbs", agent.name()),
        &json!({
            // Basic failure info
            "workflow_name": ctx.event.workflow_name,
            "workflow_url": ctx.event.workflow_url,
            "branch": ctx.event.branch,
            "commit_sha": ctx.event.head_sha,
            "commit_message": ctx.event.commit_message,
            
            // GitHub context
            "workflow_logs": ctx.workflow_logs,
            "pr_number": ctx.pr.as_ref().map(|p| p.number),
            "pr_title": ctx.pr.as_ref().map(|p| &p.title),
            "changed_files": ctx.changed_files,
            "file_diff_summary": ctx.summarize_diff(),
            
            // ArgoCD context (for Bolt especially)
            "argocd_app_status": ctx.argocd_status.health,
            "argocd_sync_status": ctx.argocd_status.sync,
            "argocd_resources_unhealthy": ctx.argocd_status.unhealthy_resources(),
            
            // Telemetry context
            "recent_error_logs": ctx.recent_logs.take(50),
            "error_rate_spike": ctx.metrics.has_spike(),
            
            // Kubernetes context
            "related_pods": ctx.pod_state.names(),
            "pod_events": ctx.pod_state.recent_events(),
            
            // Classification
            "failure_type": ctx.failure_type.name(),
            "failure_category": ctx.failure_type.category(),
            "suggested_fix_approach": ctx.failure_type.fix_approach(),
        }),
    )?;

    // Create CodeRun with full context
    let coderun = CodeRun {
        metadata: ObjectMeta {
            generate_name: Some(format!("healer-ci-{}-", agent.name())),
            namespace: Some("cto".into()),
            labels: Some(btreemap! {
                "app.kubernetes.io/name" => "healer",
                "healer/agent" => agent.name(),
                "healer/failure-type" => ctx.failure_type.name(),
                "healer/workflow-run-id" => ctx.event.workflow_run_id.to_string(),
                "healer/branch" => &ctx.event.branch,
            }),
            ..Default::default()
        },
        spec: CodeRunSpec {
            github_app: agent.github_app(),
            cli: "Claude".into(),
            model: agent.model(),
            repository_url: "https://github.com/5dlabs/cto".into(),
            prompt: Some(prompt),
            env: vec![
                EnvVar::new("HEALER_TASK_ID", ctx.task_id()),
                EnvVar::new("FAILURE_TYPE", ctx.failure_type.name()),
                EnvVar::new("WORKFLOW_RUN_ID", ctx.event.workflow_run_id),
                EnvVar::new("PR_NUMBER", ctx.pr.as_ref().map(|p| p.number).unwrap_or(0)),
            ],
            ..Default::default()
        },
    };

    self.k8s.create_coderun(coderun).await
}
```

### Tool Access Configuration

```yaml
# Healer deployment with all required tool access
apiVersion: apps/v1
kind: Deployment
metadata:
  name: healer
  namespace: cto
spec:
  template:
    spec:
      serviceAccountName: healer  # Needs K8s permissions
      containers:
        - name: healer
          image: ghcr.io/5dlabs/healer:latest
          env:
            # GitHub
            - name: GITHUB_APP_ID
              valueFrom:
                secretKeyRef:
                  name: healer-github
                  key: app-id
            - name: GITHUB_PRIVATE_KEY
              valueFrom:
                secretKeyRef:
                  name: healer-github
                  key: private-key
            
            # ArgoCD
            - name: ARGOCD_SERVER
              value: "argocd-server.argocd.svc:443"
            - name: ARGOCD_TOKEN
              valueFrom:
                secretKeyRef:
                  name: healer-argocd
                  key: token
            
            # Prometheus
            - name: PROMETHEUS_URL
              value: "http://prometheus-server.observability.svc:80"
            
            # Loki
            - name: LOKI_URL
              value: "http://loki-gateway.observability.svc:80"
            
            # Grafana (optional - for dashboard queries)
            - name: GRAFANA_URL
              value: "http://grafana.observability.svc:80"
            - name: GRAFANA_TOKEN
              valueFrom:
                secretKeyRef:
                  name: healer-grafana
                  key: token
```

### MCP Tools Available to Healer

Healer should have access to these MCP server tools:

| MCP Server | Tools | Use Case |
|------------|-------|----------|
| **argocd** | `list_applications`, `get_application`, `sync_application`, `get_application_events` | Check deployment state, sync status |
| **prometheus** | `execute_query`, `execute_range_query`, `list_metrics` | Error rates, resource usage |
| **loki** | `query`, `label_names`, `label_values` | Application logs, error patterns |
| **grafana** | `search_dashboards`, `query_prometheus`, `query_loki_logs` | Dashboard data, alert state |
| **github** | Via `gh` CLI | PR state, workflow logs, changed files |

---

## Issue Sources

Healer receives issues from multiple detection sources:

```
┌─────────────────────────────────────────────────────────────────────┐
│                        DETECTION LAYER                              │
├─────────────┬─────────────┬─────────────┬─────────────┬────────────┤
│   Stitch    │  CI Checks  │  Security   │   Play      │  Manual    │
│  (Review)   │ (Workflows) │   Alerts    │  Workflow   │ Commands   │
└──────┬──────┴──────┬──────┴──────┬──────┴──────┬──────┴─────┬──────┘
       │             │             │             │            │
       └─────────────┴─────────────┴─────────────┴────────────┘
                                   │
                                   ▼
                    ┌─────────────────────────┐
                    │         HEALER          │
                    │   (Remediation Hub)     │
                    └─────────────────────────┘
                                   │
       ┌───────────┬───────────────┼───────────────┬───────────┐
       ▼           ▼               ▼               ▼           ▼
   ┌───────┐   ┌───────┐       ┌───────┐       ┌───────┐   ┌───────┐
   │  Rex  │   │ Blaze │       │ Bolt  │       │Cipher │   │ Atlas │
   │(Rust) │   │(Front)│       │(Infra)│       │ (Sec) │   │(Git)  │
   └───────┘   └───────┘       └───────┘       └───────┘   └───────┘
```

### Source Details

| Source | Trigger | Event Type | Notes |
|--------|---------|------------|-------|
| **Stitch** | PR review posted | `pull_request_review`, `check_run` action button | Our review bot - posts findings, creates check run with "Remediate" button |
| **CI Checks** | Workflow failure | `workflow_job`, `check_run` | GitHub Actions workflow failures |
| **Security** | Alert created | `dependabot_alert`, `code_scanning_alert`, `secret_scanning_alert` | GitHub Security features |
| **Play** | Stage failure | Internal event | Play workflow monitoring (existing Healer feature) |
| **Manual** | Comment command | `issue_comment` | `/healer fix`, `/healer retry` commands |

### Stitch → Healer Flow

```
1. PR opened/synchronized
2. Stitch reviews code, posts findings as PR review
3. Stitch creates check run with "Remediate with Rex" button
4. User clicks button → `check_run.requested_action` event
5. Healer receives event, analyzes findings
6. Healer routes to appropriate agent (usually Rex for Stitch findings)
7. Agent fixes issues, pushes to PR
```

---

## Agent Specializations

| Agent | Domain | GitHub App | Use Cases |
|-------|--------|------------|-----------|
| **Rex** | Rust | 5DLabs-Rex | Clippy errors, test failures, build errors, Cargo issues |
| **Blaze** | Frontend | 5DLabs-Blaze | JavaScript, TypeScript, npm/pnpm, React, CSS |
| **Bolt** | Infrastructure | 5DLabs-Bolt | Docker builds, Helm charts, K8s manifests, Argo CD, GitOps, YAML |
| **Cipher** | Security | 5DLabs-Cipher | Dependabot alerts, code scanning, secret scanning, vulnerability fixes |
| **Atlas** | GitHub/Git | 5DLabs-Atlas | Merge conflicts, GitHub API, permissions, workflow syntax, fallback |

### Agent Tooling

| Agent | MCP Tools | Capabilities |
|-------|-----------|--------------|
| **Rex** | Rust analyzer, Cargo | Code analysis, dependency management, test running |
| **Blaze** | npm/pnpm, ESLint, TypeScript | Package management, linting, type checking |
| **Bolt** | Docker, Helm, kubectl, **Argo CD** | Container builds, chart management, cluster ops, GitOps sync |
| **Cipher** | GitHub Security, Dependabot | Vulnerability remediation, dependency updates, secret rotation |
| **Atlas** | GitHub CLI, Git | PR management, merge conflict resolution, workflow editing |

---

## GitHub Webhook Events

Healer can respond to various GitHub webhook events. The events are categorized by priority and use case.

### Primary Events (CI Remediation)

| Event | Trigger | Use Case | Agent |
|-------|---------|----------|-------|
| **Workflow jobs** | `completed` with `conclusion: failure` | Primary CI failure signal - most detailed | Router decides |
| **Check runs** | `completed` with `conclusion: failure` | Individual check failures (lint, test, build) | Router decides |
| **Check suites** | `completed` with `conclusion: failure` | Aggregate check status | Router decides |
| **Statuses** | `failure` or `error` | Commit status from external CI | Router decides |

#### Workflow Jobs Event (Recommended Primary)

```json
{
  "action": "completed",
  "workflow_job": {
    "id": 12345,
    "name": "clippy",
    "conclusion": "failure",
    "workflow_name": "Controller CI",
    "steps": [
      { "name": "Run clippy", "conclusion": "failure" }
    ]
  },
  "repository": { "full_name": "5dlabs/cto" }
}
```

**Advantages over `workflow_run`:**
- Fires per job, not per workflow (more granular)
- Includes step-level failure details
- Faster feedback (don't wait for entire workflow)

### Secondary Events (Extended Remediation)

| Event | Trigger | Use Case | Agent |
|-------|---------|----------|-------|
| **Pull requests** | `opened`, `synchronize` | Pre-emptive checks, auto-fix on push | Router decides |
| **Push** | Any push to monitored branches | Detect problematic commits early | Router decides |
| **Issue comments** | `/healer fix`, `/healer retry` | Manual trigger via comment commands | Command parser |

### Security Events → Cipher

| Event | Trigger | Use Case | Agent |
|-------|---------|----------|-------|
| **Dependabot alerts** | `created`, `reopened` | Auto-fix dependency vulnerabilities | **Cipher** |
| **Code scanning alerts** | `created` | Security issue remediation | **Cipher** |
| **Secret scanning alerts** | `created` | Rotate/revoke leaked secrets | **Cipher** |
| **Repository vulnerability alerts** | `created` | Dependency security fixes | **Cipher** |

### Lifecycle Events (Tracking)

| Event | Trigger | Use Case | Agent |
|-------|---------|----------|-------|
| **Pull request reviews** | `submitted` | Track fix PR approval | - (tracking only) |
| **Check runs** | `completed` with `success` | Confirm fix worked | - (tracking only) |
| **Merge groups** | `checks_requested` | Merge queue monitoring | - (tracking only) |

---

## Event Payload Examples

### Workflow Job Failure

```json
{
  "action": "completed",
  "workflow_job": {
    "id": 29679449526,
    "run_id": 12216892003,
    "workflow_name": "Controller CI",
    "name": "lint-rust",
    "conclusion": "failure",
    "started_at": "2025-01-15T10:30:00Z",
    "completed_at": "2025-01-15T10:32:15Z",
    "steps": [
      {
        "name": "Run Clippy (pedantic)",
        "status": "completed",
        "conclusion": "failure",
        "number": 4
      }
    ],
    "labels": ["k8s-runner"],
    "runner_name": "k8s-runner-abc123"
  },
  "repository": {
    "full_name": "5dlabs/cto",
    "default_branch": "main"
  },
  "sender": {
    "login": "dependabot[bot]"
  }
}
```

### Check Run Failure

```json
{
  "action": "completed",
  "check_run": {
    "id": 12345678,
    "name": "clippy",
    "status": "completed",
    "conclusion": "failure",
    "output": {
      "title": "Clippy found 3 errors",
      "summary": "error: unused variable `x`\n..."
    },
    "check_suite": {
      "head_branch": "feat/new-feature",
      "head_sha": "abc123def456"
    }
  },
  "repository": {
    "full_name": "5dlabs/cto"
  }
}
```

### Pull Request Event (for `/healer` commands)

```json
{
  "action": "created",
  "comment": {
    "body": "/healer fix clippy",
    "user": { "login": "developer" }
  },
  "issue": {
    "number": 123,
    "pull_request": { "url": "..." }
  },
  "repository": {
    "full_name": "5dlabs/cto"
  }
}
```

---

## Event Filtering Strategy

### Argo Events Sensor Filters

```yaml
dependencies:
  - name: ci-failure
    eventSourceName: github
    eventName: org
    filters:
      data:
        # Workflow job completed events
        - path: headers.X-GitHub-Event
          type: string
          value: ["workflow_job"]
        - path: body.action
          type: string
          value: ["completed"]
        - path: body.workflow_job.conclusion
          type: string
          value: ["failure"]
        # Only our repository
        - path: body.repository.full_name
          type: string
          value: ["5dlabs/cto"]
      exprs:
        # Skip if commit message contains skip flag
        - expr: '!(body.workflow_job.head_commit.message contains "[skip-healer]")'
```

### Healer-Side Validation

Even with sensor filtering, Healer performs additional validation:

```rust
fn should_process(&self, event: &GitHubEvent) -> bool {
    // Verify event type
    if !matches!(event.event_type, EventType::WorkflowJob | EventType::CheckRun) {
        return false;
    }
    
    // Verify failure
    if event.conclusion != "failure" {
        return false;
    }
    
    // Skip bot-generated commits (prevent loops)
    if event.sender.ends_with("[bot]") && !self.config.process_bot_commits {
        return false;
    }
    
    // Skip if already being remediated
    if self.has_active_remediation(&event.run_id) {
        return false;
    }
    
    true
}
```

---

## Recommended Event Configuration

### Phase 1: CI Failures Only

Enable these events on the GitHub App:
- ✅ **Workflow jobs** - Primary CI signal
- ✅ **Check runs** - Granular check failures

### Phase 2: Extended Triggers

Add these events:
- ✅ **Issue comments** - For `/healer` commands
- ✅ **Pull requests** - For auto-fix on sync

### Phase 3: Security Remediation

Add these events:
- ✅ **Dependabot alerts** - Dependency fixes
- ✅ **Code scanning alerts** - Security fixes
- ✅ **Secret scanning alerts** - Secret rotation

---

## Failure Detection & Routing

### Detection Sources

1. **Workflow name** - Primary signal (e.g., `controller-ci`, `infrastructure-build`)
2. **Workflow path** - Which files triggered the workflow
3. **Log content** - Error patterns in failure logs
4. **Changed files** - File extensions in the failing commit

### Routing Rules

```
┌─────────────────────────────────────────────────────────────────────┐
│                        CI Failure / Security Event                  │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
                    ┌─────────────────────────┐
                    │   Healer Analyzes &     │
                    │   Routes to Specialist  │
                    └─────────────────────────┘
                                  │
        ┌───────────┬─────────────┼─────────────┬───────────┐
        ▼           ▼             ▼             ▼           ▼
   ┌─────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌─────────┐
   │  Rust   │ │ Frontend │ │  Infra   │ │ Security │ │  Git/   │
   │ Clippy  │ │   npm    │ │  Docker  │ │Dependabot│ │ GitHub  │
   │  Test   │ │   pnpm   │ │   Helm   │ │CodeScan  │ │ Merge   │
   │  Build  │ │   tsx    │ │  ArgoCD  │ │ Secrets  │ │Conflict │
   └────┬────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬────┘
        │           │            │            │            │
        ▼           ▼            ▼            ▼            ▼
   ┌─────────┐ ┌──────────┐ ┌────────┐  ┌──────────┐ ┌─────────┐
   │   Rex   │ │  Blaze   │ │  Bolt  │  │  Cipher  │ │  Atlas  │
   └─────────┘ └──────────┘ └────────┘  └──────────┘ │(default)│
                                                     └─────────┘
```

**Note:** Atlas serves as the default/fallback agent. Healer always routes to one of the five agents - there is no "unknown" category.

### Detailed Routing Matrix

Healer evaluates patterns in order. First match wins; Atlas is the default if no pattern matches.

| Priority | Workflow Pattern | Log/Event Pattern | Changed Files | → Agent |
|----------|------------------|-------------------|---------------|---------|
| 1 | `*-ci`, `controller-*`, `healer-*` | `clippy`, `cargo test`, `rustc` | `*.rs`, `Cargo.toml` | **Rex** |
| 2 | `frontend-*`, `ui-*` | `npm`, `pnpm`, `tsc`, `eslint` | `*.ts`, `*.tsx`, `*.js`, `package.json` | **Blaze** |
| 3 | `infrastructure-*`, `docker-*`, `helm-*` | `docker build`, `helm`, `kubectl` | `Dockerfile`, `*.yaml`, `Chart.yaml` | **Bolt** |
| 4 | `argocd-*`, `gitops-*`, `sync-*` | `argocd`, `sync failed`, `OutOfSync` | `infra/gitops/*`, `applications/*` | **Bolt** |
| 5 | (security event) | `dependabot_alert`, `code_scanning_alert`, `secret_scanning_alert` | (any) | **Cipher** |
| 6 | (security event) | `vulnerability`, `CVE-`, `security advisory` | `Cargo.lock`, `package-lock.json` | **Cipher** |
| 7 | (any) | `merge conflict`, `CONFLICT`, `cannot merge` | (any) | **Atlas** |
| 8 | `*-release`, `deploy-*` | `push`, `ghcr.io`, `permission` | `.github/workflows/*` | **Atlas** |
| — | *(default)* | *(no match)* | *(no match)* | **Atlas** |

---

## Architecture

### Components

```
┌──────────────────────────────────────────────────────────────────┐
│                           Healer                                  │
├──────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐   │
│  │  HTTP API   │───▶│  CI Router  │───▶│  CodeRun Spawner    │   │
│  │  /api/...   │    │             │    │  (with dedup)       │   │
│  └─────────────┘    └─────────────┘    └─────────────────────┘   │
│         │                  │                      │               │
│         │                  ▼                      ▼               │
│         │          ┌─────────────┐    ┌─────────────────────┐   │
│         │          │  Log Fetch  │    │  Insights Collector │   │
│         │          │  (gh CLI)   │    │                     │   │
│         │          └─────────────┘    └─────────────────────┘   │
│         │                                                         │
│         ▼                                                         │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │                    Prompt Templates                          │ │
│  │  prompts/ci/rust-fix.hbs      → Rex                          │ │
│  │  prompts/ci/frontend-fix.hbs  → Blaze                        │ │
│  │  prompts/ci/infra-fix.hbs     → Bolt                         │ │
│  │  prompts/ci/security-fix.hbs  → Cipher                       │ │
│  │  prompts/ci/github-fix.hbs    → Atlas                        │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                   │
└──────────────────────────────────────────────────────────────────┘
```

### HTTP API

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check |
| `/api/remediate/ci-failure` | POST | Receive CI failure webhook |
| `/api/status` | GET | Current remediation status |

### Request Payload (from Sensor)

```json
{
  "workflow_run": {
    "id": 12345678,
    "name": "Controller CI",
    "conclusion": "failure",
    "html_url": "https://github.com/5dlabs/cto/actions/runs/12345678",
    "head_branch": "feat/new-feature",
    "head_sha": "abc123def456",
    "head_commit": {
      "message": "feat: add new feature"
    }
  },
  "repository": {
    "full_name": "5dlabs/cto"
  }
}
```

### Response

```json
{
  "status": "accepted",
  "coderun_name": "healer-ci-rex-abc12345",
  "agent": "rex",
  "failure_type": "rust_clippy",
  "dedup_status": "new"  // or "existing", "skipped"
}
```

---

## Deduplication

Healer prevents duplicate remediation attempts:

1. **By workflow run ID** - Only one fix per workflow run
2. **By branch + time window** - Don't spam fixes on rapid commits
3. **By existing CodeRun** - Check for active remediation first

```rust
fn should_remediate(&self, failure: &CiFailure) -> bool {
    // Check if CodeRun already exists for this workflow run
    if self.has_existing_coderun(failure.workflow_run_id) {
        return false;
    }
    
    // Check time window (no duplicate fixes within 10 min)
    if self.recent_remediation_for_branch(&failure.branch, Duration::minutes(10)) {
        return false;
    }
    
    true
}
```

---

## Prompt Templates

### Rex (Rust)

```handlebars
# CI Rust Fix - Rex

You are Rex, the Rust specialist. A CI workflow has failed with Rust-related errors.

## Failure Details
- **Workflow**: {{workflow_name}}
- **Branch**: {{branch}}
- **Commit**: {{commit_sha}}

## Failure Logs
```
{{logs}}
```

## Instructions
1. Analyze the Rust compiler/clippy/test errors above
2. Identify the root cause in the code
3. Apply a minimal, targeted fix
4. Ensure `cargo clippy --all-targets -- -D warnings -W clippy::pedantic` passes
5. Ensure `cargo test` passes
6. Create a PR with clear description of the fix

Focus only on fixing the CI failure. Do not refactor unrelated code.
```

### Blaze (Frontend)

```handlebars
# CI Frontend Fix - Blaze

You are Blaze, the frontend specialist. A CI workflow has failed with JavaScript/TypeScript errors.

## Failure Details
- **Workflow**: {{workflow_name}}
- **Branch**: {{branch}}
- **Commit**: {{commit_sha}}

## Failure Logs
```
{{logs}}
```

## Instructions
1. Analyze the npm/TypeScript/ESLint errors above
2. Identify the root cause (missing deps, type errors, lint issues)
3. Apply a minimal, targeted fix
4. Ensure the build passes
5. Create a PR with clear description

Focus only on fixing the CI failure. Do not refactor unrelated code.
```

### Bolt (Infrastructure)

```handlebars
# CI Infrastructure Fix - Bolt

You are Bolt, the infrastructure specialist. A CI workflow has failed with Docker/Helm/K8s errors.

## Failure Details
- **Workflow**: {{workflow_name}}
- **Branch**: {{branch}}
- **Commit**: {{commit_sha}}

## Failure Logs
```
{{logs}}
```

## Instructions
1. Analyze the Docker/Helm/YAML errors above
2. Identify the root cause (Dockerfile issue, Helm values, K8s manifest)
3. Apply a minimal, targeted fix
4. Validate YAML syntax and Helm templates
5. Create a PR with clear description

Focus only on fixing the CI failure. Do not change application code.
```

### Cipher (Security)

```handlebars
# Security Fix - Cipher

You are Cipher, the security specialist. A security alert has been raised that requires remediation.

## Alert Details
- **Type**: {{alert_type}}
- **Severity**: {{severity}}
- **Package**: {{package_name}}
- **CVE**: {{cve_id}}
- **Branch**: {{branch}}

## Alert Description
```
{{alert_description}}
```

## Instructions
1. Analyze the security alert and identify the vulnerable component
2. Determine the appropriate fix:
   - **Dependency vulnerability**: Update to patched version in Cargo.toml/package.json
   - **Code scanning alert**: Fix the vulnerable code pattern
   - **Secret leak**: Rotate the secret and update references
3. Apply the minimal fix to resolve the vulnerability
4. Ensure the fix doesn't break existing functionality
5. Create a PR with clear description of the security impact

## Security Best Practices
- Always update to the latest patched version, not just minimum safe version
- Check for breaking changes in major version updates
- Verify no new vulnerabilities are introduced by the update
- Document the CVE and remediation in the commit message
```

### Atlas (GitHub/General)

```handlebars
# CI General Fix - Atlas

You are Atlas, handling a CI failure that doesn't fit specialist categories.

## Failure Details
- **Workflow**: {{workflow_name}}
- **Branch**: {{branch}}
- **Commit**: {{commit_sha}}

## Failure Logs
```
{{logs}}
```

## Instructions
1. Analyze the CI failure logs above
2. Identify the root cause
3. Apply a minimal, targeted fix
4. Create a PR with clear description

If this failure would be better handled by a specialist:
- Rust issues → mention Rex should handle this
- Frontend issues → mention Blaze should handle this
- Infrastructure issues → mention Bolt should handle this
```

---

## Sensor Changes

Update `ci-failure-remediation-sensor.yaml` to call Healer:

```yaml
triggers:
  - template:
      name: trigger-healer-ci-remediation
      conditions: "workflow-failure"
      http:
        url: http://healer.cto.svc:8080/api/remediate/ci-failure
        method: POST
        headers:
          Content-Type: application/json
        payload:
          - src:
              dependencyName: workflow-failure
              dataKey: body
            dest: body
      retryStrategy:
        steps: 3
        duration: "10s"
```

---

## Healer Deployment Changes

Add HTTP server capability:

```yaml
# In Healer deployment
spec:
  containers:
    - name: healer
      command: ["healer", "server"]
      ports:
        - containerPort: 8080
          name: http
      livenessProbe:
        httpGet:
          path: /health
          port: 8080
      readinessProbe:
        httpGet:
          path: /health
          port: 8080
```

Add Service:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: healer
  namespace: cto
spec:
  selector:
    app: healer
  ports:
    - port: 8080
      targetPort: http
```

---

## Implementation Phases

### Phase 1: Core Router (MVP)
- [ ] Add `ci/` module with router logic
- [ ] Add `healer server` command
- [ ] Implement `/api/remediate/ci-failure` endpoint
- [ ] Basic failure detection (workflow name only)
- [ ] Route to Rex/Blaze/Bolt/Atlas

### Phase 2: Enhanced Detection
- [ ] Fetch and analyze workflow logs via `gh` CLI
- [ ] Pattern matching on log content
- [ ] Changed files analysis

### Phase 3: Deduplication & Tracking
- [ ] Check for existing remediation CodeRuns
- [ ] Time-window deduplication
- [ ] Track remediation attempts and outcomes

### Phase 4: Learning & Insights
- [ ] Record success/failure by agent
- [ ] Identify routing improvements
- [ ] Track time-to-fix metrics

### Phase 5: Sensor Migration
- [ ] Update sensor to call Healer HTTP endpoint
- [ ] Remove direct CodeRun creation from sensor
- [ ] Deploy and validate end-to-end

---

## Configuration Defaults

### CLI & Model

For initial implementation, use consistent defaults:

| Setting | Value | Rationale |
|---------|-------|-----------|
| **CLI** | Factory | Our primary agent runtime |
| **Model** | `claude-opus-4-5-20250929` | Best reasoning until fine-tuned |
| **Max Attempts** | 3 | Avoid wasting credits while building |
| **Time Window** | 10 minutes | Prevent rapid retry spam |

```rust
pub struct RemediationConfig {
    pub cli: String,           // "Factory"
    pub model: String,         // "claude-opus-4-5-20250929"
    pub max_attempts: u32,     // 3
    pub time_window_mins: u32, // 10
}

impl Default for RemediationConfig {
    fn default() -> Self {
        Self {
            cli: "Factory".into(),
            model: "claude-opus-4-5-20250929".into(),
            max_attempts: 3,
            time_window_mins: 10,
        }
    }
}
```

### Branch Strategy

**Fixes go to the existing PR branch** - the goal is to get PRs green without human intervention.

```rust
fn determine_target_branch(&self, ctx: &RemediationContext) -> String {
    // If there's an existing PR for this branch, push to that PR's branch
    if let Some(pr) = &ctx.pr {
        return pr.head_ref.clone();
    }
    
    // If it's a push to main that failed, create a fix branch
    if ctx.event.branch == "main" {
        return format!("fix/{}-{}", ctx.failure_type.short_name(), &ctx.event.head_sha[..8]);
    }
    
    // Otherwise push to the failing branch
    ctx.event.branch.clone()
}
```

---

## Recursive Remediation Loop

Healer implements a **recursive remediation loop** where failed agent attempts trigger re-analysis with more context, not just blind retries.

### Loop Flow

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                        RECURSIVE REMEDIATION LOOP                             │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│   ┌─────────────┐                                                            │
│   │  CI Failure │                                                            │
│   │   Event     │                                                            │
│   └──────┬──────┘                                                            │
│          │                                                                   │
│          ▼                                                                   │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                           HEALER                                     │   │
│   │  1. Gather context (logs, PR, ArgoCD, metrics)                      │   │
│   │  2. Route to specialist agent                                        │   │
│   │  3. Spawn CodeRun with enriched prompt                               │   │
│   └──────────────────────────────────┬──────────────────────────────────┘   │
│                                      │                                       │
│                                      ▼                                       │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                        SPECIALIST AGENT                              │   │
│   │  (Rex / Blaze / Bolt / Cipher / Atlas)                              │   │
│   │  Attempts to fix the issue, pushes to PR branch                     │   │
│   └──────────────────────────────────┬──────────────────────────────────┘   │
│                                      │                                       │
│                    ┌─────────────────┴─────────────────┐                    │
│                    ▼                                   ▼                    │
│            ┌───────────────┐                   ┌───────────────┐            │
│            │   SUCCESS     │                   │   FAILURE     │            │
│            │ CI passes ✓   │                   │ Agent failed  │            │
│            └───────┬───────┘                   └───────┬───────┘            │
│                    │                                   │                    │
│                    ▼                                   ▼                    │
│            ┌───────────────┐                   ┌───────────────┐            │
│            │   Complete    │                   │  attempt < 3? │            │
│            │   Log insight │                   └───────┬───────┘            │
│            └───────────────┘                   ┌───────┴───────┐            │
│                                                ▼               ▼            │
│                                         ┌──────────┐   ┌──────────────┐    │
│                                         │   YES    │   │     NO       │    │
│                                         │ Re-enter │   │  ESCALATE    │    │
│                                         │  loop    │   │  to human    │    │
│                                         └────┬─────┘   └──────┬───────┘    │
│                                              │                │            │
│                                              │                ▼            │
│                                              │         ┌──────────────┐    │
│                                              │         │   Notify     │    │
│                                              │         │   (Discord)  │    │
│                                              │         └──────────────┘    │
│                                              │                             │
│   ┌──────────────────────────────────────────┴──────────────────────────┐  │
│   │                     RE-ANALYSIS PHASE                                │  │
│   │  • Fetch agent's failure output/logs                                │  │
│   │  • Query additional context (what did agent try? what went wrong?) │  │
│   │  • Augment prompt with "Previous attempt failed because..."         │  │
│   │  • Possibly route to different agent                                │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                      │                                      │
│                                      └──────────────────────────────────────┘
│                                         (loops back to HEALER)
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Implementation

```rust
pub struct RemediationAttempt {
    pub attempt_number: u32,
    pub agent: Agent,
    pub coderun_name: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub outcome: Option<AttemptOutcome>,
    pub failure_reason: Option<String>,
    pub agent_output: Option<String>,
}

pub enum AttemptOutcome {
    Success,           // CI passed after fix
    AgentFailed,       // Agent crashed or errored
    CiStillFailing,    // Agent pushed but CI still fails
    Timeout,           // Agent exceeded time limit
}

impl Healer {
    pub async fn handle_agent_completion(&self, coderun: &CodeRun) -> Result<()> {
        let task_id = coderun.labels.get("healer/task-id")?;
        let mut state = self.get_remediation_state(task_id).await?;
        
        // Record this attempt
        let outcome = self.determine_outcome(coderun).await?;
        state.record_attempt(outcome.clone(), coderun);
        
        match outcome {
            AttemptOutcome::Success => {
                // 🎉 Fixed! Log insight for learning
                self.insights.record_success(&state).await;
                tracing::info!(task_id, "Remediation succeeded on attempt {}", state.attempts.len());
            }
            
            _ if state.attempts.len() >= self.config.max_attempts => {
                // ❌ Max attempts reached - escalate to human
                self.escalate_to_human(&state).await?;
            }
            
            _ => {
                // 🔄 Re-enter loop with more context
                self.retry_with_more_context(&state).await?;
            }
        }
        
        self.save_remediation_state(&state).await
    }
    
    async fn retry_with_more_context(&self, state: &RemediationState) -> Result<()> {
        // Gather what the agent tried and why it failed
        let last_attempt = state.attempts.last().unwrap();
        let agent_logs = self.fetch_coderun_logs(&last_attempt.coderun_name).await?;
        let what_changed = self.github.get_commits_since(&state.original_sha).await?;
        
        // Re-gather context with new information
        let mut ctx = self.gather_context(&state.event).await?;
        ctx.previous_attempts = state.attempts.clone();
        ctx.agent_failure_output = Some(agent_logs);
        ctx.changes_made_so_far = what_changed;
        
        // Possibly route to a different agent if current one is struggling
        let agent = if state.attempts.len() >= 2 && state.same_agent_failed_twice() {
            self.try_different_agent(&ctx)
        } else {
            state.current_agent()
        };
        
        // Spawn new CodeRun with augmented context
        let prompt = self.build_retry_prompt(&ctx, &state);
        self.spawn_coderun(agent, &ctx, &prompt).await
    }
    
    async fn escalate_to_human(&self, state: &RemediationState) -> Result<()> {
        // Send notification via notify module
        let message = format!(
            "🚨 **CI Remediation Failed** - Human intervention needed\n\n\
             **Workflow**: {}\n\
             **Branch**: {}\n\
             **Attempts**: {} (max {})\n\
             **Last Agent**: {:?}\n\
             **PR**: {}\n\n\
             Agents tried but couldn't fix this. Please investigate.",
            state.event.workflow_name,
            state.event.branch,
            state.attempts.len(),
            self.config.max_attempts,
            state.current_agent(),
            state.pr_url().unwrap_or("(no PR)".into()),
        );
        
        self.notify.send(NotifyChannel::Discord, &message).await?;
        
        // Also comment on the PR if one exists
        if let Some(pr_number) = state.pr_number {
            self.github.comment_pr(
                pr_number,
                &format!(
                    "## ⚠️ Healer: Automatic remediation failed\n\n\
                     I tried {} times to fix the CI failures but wasn't successful.\n\n\
                     **What I tried**:\n{}\n\n\
                     A human needs to take a look at this one.\n\n\
                     ---\n*Escalated by Healer*",
                    state.attempts.len(),
                    state.summarize_attempts(),
                ),
            ).await?;
        }
        
        Ok(())
    }
}
```

### Augmented Retry Prompt

When retrying, the prompt includes previous attempt context:

```handlebars
# CI Rust Fix - Rex (Retry Attempt {{attempt_number}})

You are Rex, the Rust specialist. This is **retry attempt {{attempt_number}}** of {{max_attempts}}.

## ⚠️ Previous Attempt Failed

The previous attempt by {{previous_agent}} did not resolve the issue.

### What Was Tried
{{#each changes_made_so_far}}
- {{this.message}} ({{this.sha}})
{{/each}}

### Why It Failed
```
{{agent_failure_output}}
```

### Current CI Status
```
{{current_logs}}
```

## Instructions

1. **Analyze why the previous fix didn't work**
2. Review the error output above carefully
3. Apply a **different approach** - don't repeat the same fix
4. Ensure `cargo clippy --all-targets -- -D warnings -W clippy::pedantic` passes
5. Push directly to the PR branch: `{{target_branch}}`

**CRITICAL**: Do not repeat failed approaches. Try something new.
```

---

## Human Escalation via Notify

When max attempts are exhausted, Healer uses the `notify` module to alert humans:

```rust
// In crates/healer/src/notify.rs (existing module)

pub enum NotifyChannel {
    Discord,
    Slack,
    Email,
}

pub struct NotifyMessage {
    pub channel: NotifyChannel,
    pub title: String,
    pub body: String,
    pub severity: NotifySeverity,
    pub context: HashMap<String, String>,
}

pub enum NotifySeverity {
    Info,
    Warning,
    Critical,  // Used for escalations
}
```

### Discord Notification Example

```
🚨 **CI Remediation Failed** - Human intervention needed

**Workflow**: Controller CI
**Branch**: feat/new-feature
**PR**: #1234
**Attempts**: 3 (max 3)
**Agents Tried**: Rex (2x), Atlas (1x)

**Summary of attempts**:
1. Rex: Fixed clippy lint, but new test failure appeared
2. Rex: Fixed test, but introduced new clippy error  
3. Atlas: Tried general fix approach, still failing

**Current Failure**:
```
error[E0382]: borrow of moved value: `config`
```

Please investigate: https://github.com/5dlabs/cto/pull/1234
```

---

## Success Metrics

| Metric | Target |
|--------|--------|
| Correct agent routing | >90% |
| Duplicate prevention | 100% |
| Time to fix (P50) | <15 min |
| Fix success rate (attempt 1) | >50% |
| Fix success rate (within 3 attempts) | >80% |
| Human escalation rate | <10% |

---

## Implementation Phases

### Phase 1: Core Router (MVP)
- [ ] Add `ci/` module with router logic
- [ ] Add `healer server` command
- [ ] Implement `/api/remediate/ci-failure` endpoint
- [ ] Basic failure detection (workflow name only)
- [ ] Route to specialist agents (Rex/Blaze/Bolt/Cipher/Atlas)
- [ ] Default to Factory CLI + Opus 4.5

### Phase 2: Enhanced Detection & Context
- [ ] Fetch and analyze workflow logs via `gh` CLI
- [ ] Pattern matching on log content
- [ ] Changed files analysis
- [ ] Context enrichment from ArgoCD, Loki, K8s

### Phase 3: OpenMemory Integration
- [ ] Add Healer agent to OpenMemory configuration
- [ ] Implement `query_historical_context()` for routing decisions
- [ ] Query past remediations for similar failures
- [ ] Query agent success rates by failure type
- [ ] Enrich agent prompts with historical solutions

### Phase 4: Recursive Loop & Tracking
- [ ] Watch CodeRun completions
- [ ] Implement retry-with-more-context logic
- [ ] Track attempts per task
- [ ] Max 3 attempts default

### Phase 5: Memory Storage & Learning
- [ ] Record remediation outcomes to OpenMemory
- [ ] Store success patterns (agent, approach, time-to-fix)
- [ ] Store escalation events for learning
- [ ] Detect routing mismatches and improve
- [ ] Feed insights back to prompts

### Phase 6: Escalation & Notifications
- [ ] Human escalation after max attempts
- [ ] Discord notifications via notify module
- [ ] PR comments explaining what was tried

### Phase 7: Sensor Migration & Cleanup
- [ ] Update sensor to call Healer HTTP endpoint
- [ ] Remove direct CodeRun creation from sensor
- [ ] Remove `templates/remediate/` directory
- [ ] Deploy and validate end-to-end

### Phase 8: Advanced Coordination (Future)
- [ ] Evaluate Beads for multi-failure dependency tracking
- [ ] Evaluate Agent Mail for bi-directional agent communication
- [ ] Implement if justified by complexity needs

---

---

## OpenMemory Integration (Server-Side)

Healer uses OpenMemory for **server-side historical context**, enabling intelligent routing decisions based on past remediation patterns.

### Why Server-Side Memory?

| Component | Memory Use | Purpose |
|-----------|------------|---------|
| **Healer Server** | Long-term (3-4 days) | Historical patterns, routing improvements, failure correlations |
| **Agent CodeRuns** | Task-scoped | Immediate context for current fix attempt |

The server maintains a "big picture" view while agents focus on the immediate task.

### Memory Integration Points

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                     HEALER + OPENMEMORY INTEGRATION                           │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│   ┌─────────────────┐          ┌─────────────────────────────────────────┐   │
│   │   CI Failure    │          │           OpenMemory Server              │   │
│   │     Event       │          │           (cto-system:3000)              │   │
│   └────────┬────────┘          └─────────────────────────────────────────┘   │
│            │                              ▲                                   │
│            ▼                              │                                   │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                           HEALER SERVER                              │   │
│   │                                                                      │   │
│   │  1. QUERY: "What do I know about this type of failure?"             │   │
│   │     └── Past remediations for similar errors                        │   │
│   │     └── Success rates by agent for this failure type               │   │
│   │     └── Common root causes and fixes                                │   │
│   │                                                                      │   │
│   │  2. ROUTE: Use historical success rates to pick best agent          │   │
│   │                                                                      │   │
│   │  3. ENRICH: Include relevant memories in agent prompt               │   │
│   │                                                                      │   │
│   │  4. STORE: Record outcome (success/failure/escalation)              │   │
│   │     └── Agent used, time to fix, what worked                        │   │
│   │     └── For failures: what was tried, why it didn't work           │   │
│   │                                                                      │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Memory Queries

```rust
impl Healer {
    /// Query OpenMemory before routing a CI failure
    async fn query_historical_context(&self, ctx: &RemediationContext) -> HistoricalContext {
        let memory_client = OpenMemoryClient::new(&self.config.memory_url);
        
        // Query for similar failures
        let similar_failures = memory_client.query(&format!(
            "CI failure {} in workflow {} with error: {}",
            ctx.failure_type.name(),
            ctx.event.workflow_name,
            ctx.error_summary(),
        ), 10).await?;
        
        // Query for agent success rates on this failure type
        let agent_patterns = memory_client.query(&format!(
            "remediation success {} agent",
            ctx.failure_type.name(),
        ), 5).await?;
        
        // Query for known solutions
        let solutions = memory_client.query(&format!(
            "fix solution {} {}",
            ctx.failure_type.name(),
            ctx.workflow_logs.error_signature(),
        ), 5).await?;
        
        HistoricalContext {
            similar_failures,
            agent_success_patterns: agent_patterns,
            known_solutions: solutions,
        }
    }
}
```

### Memory Storage Events

```rust
/// Events that trigger memory storage
enum MemoryEvent {
    /// Remediation succeeded on first attempt
    FirstAttemptSuccess {
        failure_type: CiFailureType,
        agent: Agent,
        fix_approach: String,
        time_to_fix: Duration,
    },
    
    /// Remediation succeeded after retry
    RetrySuccess {
        failure_type: CiFailureType,
        attempts: Vec<AttemptSummary>,
        winning_approach: String,
    },
    
    /// Escalated to human - valuable learning signal
    Escalation {
        failure_type: CiFailureType,
        attempts: Vec<AttemptSummary>,
        why_agents_failed: String,
    },
    
    /// Agent routing was suboptimal
    RoutingMismatch {
        initial_agent: Agent,
        successful_agent: Agent,
        failure_type: CiFailureType,
    },
}

impl Healer {
    /// Store remediation outcome in OpenMemory
    async fn record_to_memory(&self, event: MemoryEvent) -> Result<()> {
        let memory_client = OpenMemoryClient::new(&self.config.memory_url);
        
        match event {
            MemoryEvent::FirstAttemptSuccess { failure_type, agent, fix_approach, time_to_fix } => {
                memory_client.add_memory(
                    &format!(
                        "Successful {} remediation by {} in {:?}: {}",
                        failure_type.name(), agent.name(), time_to_fix, fix_approach
                    ),
                    json!({
                        "type": "remediation_success",
                        "failure_type": failure_type.name(),
                        "agent": agent.name(),
                        "time_to_fix_secs": time_to_fix.as_secs(),
                        "first_attempt": true,
                    }),
                ).await?;
            }
            
            MemoryEvent::Escalation { failure_type, attempts, why_agents_failed } => {
                memory_client.add_memory(
                    &format!(
                        "ESCALATION: {} required human help after {} attempts. Reason: {}",
                        failure_type.name(), attempts.len(), why_agents_failed
                    ),
                    json!({
                        "type": "escalation",
                        "failure_type": failure_type.name(),
                        "attempts": attempts.len(),
                        "agents_tried": attempts.iter().map(|a| a.agent.name()).collect::<Vec<_>>(),
                    }),
                ).await?;
            }
            
            // ... other events
        }
        
        Ok(())
    }
}
```

### Configuration

Add Healer as an agent in OpenMemory:

```yaml
# In infra/charts/openmemory/values.yaml
agents:
  - name: healer
    namespace: cto
    queryLimit: 15          # More queries for routing decisions
    reinforcementMultiplier: 1.5  # Learn faster from outcomes
```

### Prompt Enrichment

When spawning a CodeRun, include relevant memories:

```handlebars
# CI Rust Fix - Rex

{{#if historical_context.known_solutions}}
## 📚 Historical Context

Similar issues have been fixed before:

{{#each historical_context.known_solutions}}
- **{{this.pattern}}**: {{this.solution}}
{{/each}}

Consider these approaches when analyzing the current failure.
{{/if}}

## Failure Details
...
```

---

## Agent Communication: Beads & Agent Mail Evaluation

Two emerging tools have been evaluated for potential integration:

### Beads (Graph-Based Task Planner)

**Repository**: https://github.com/steveyegge/beads

| Feature | Description | Healer Use Case |
|---------|-------------|-----------------|
| **Issue Graph** | Tracks tasks with dependencies | Track remediation attempts as a dependency graph |
| **Ready Work Detection** | Identifies unblocked tasks | Find which failures can be worked on in parallel |
| **Agent-Friendly JSON** | `bd export --json` | Easy integration with Healer's Rust code |
| **Git-Versioned** | JSONL records in repo | Audit trail of all remediation decisions |
| **Memory Decay** | Agent-driven compaction | Auto-clean old remediation records |

**Potential Integration**:
```rust
// Track remediation as Beads issues
struct RemediationIssue {
    id: String,                    // "healer/ci-123"
    failure_type: CiFailureType,
    status: BeadStatus,            // pending, in_progress, done, blocked
    depends_on: Vec<String>,       // Other issues this needs resolved first
    agent_assigned: Option<Agent>,
}

// Use Beads to find what can be worked on
let ready_issues = beads.find_ready();  // Issues with no blockers
```

**Verdict**: 🟡 **Useful for complex multi-failure scenarios** where remediations have dependencies (e.g., "fix Cargo.lock before fixing Clippy"). Not essential for MVP but valuable for Phase 2.

### Agent Mail (MCP Coordination Layer)

**Repository**: https://github.com/Dicklesworthstone/mcp_agent_mail

| Feature | Description | Healer Use Case |
|---------|-------------|-----------------|
| **Inbox/Outbox** | Structured messaging | Healer sends instructions, agents report status |
| **File Leases** | Avoid edit conflicts | Prevent multiple agents from editing same file |
| **Message History** | Searchable archive | Review past agent communications |
| **Web UI** | Human oversight | Monitor Healer-agent conversations |

**Potential Integration**:
```rust
// Healer sends a remediation request
agent_mail.send(AgentMessage {
    from: "healer",
    to: "rex",
    subject: "CI Failure: clippy lint",
    body: json!({
        "workflow_run_id": 12345,
        "failure_type": "rust_clippy",
        "priority": "high",
        "prompt": rendered_prompt,
    }),
    claims: vec!["/crates/healer/src/main.rs"],  // Reserve files
});

// Agent responds when done
let response = agent_mail.receive("healer").await;
match response.status {
    "fixed" => self.mark_resolved(response.task_id),
    "blocked" => self.escalate_or_retry(response),
}
```

**Verdict**: 🟢 **Good for bi-directional communication** if we want agents to report back mid-execution or request clarification. However, the current CodeRun + watch approach is simpler and works well. Consider for Phase 3 when we need richer agent coordination.

### Recommendation

| Phase | Communication Approach | Rationale |
|-------|------------------------|-----------|
| **Phase 1 (MVP)** | CodeRun CRD + K8s watch | Simple, proven, already implemented |
| **Phase 2** | Add Beads for task graph | Track multi-failure dependencies |
| **Phase 3** | Evaluate Agent Mail | If agents need mid-run communication |

For now, we continue with the CodeRun-based approach since:
1. It's already working in the play module
2. CodeRun labels provide good tracking
3. K8s events give completion notifications
4. Adding complexity should be justified by need

---

## Related Files

- `/infra/gitops/resources/sensors/ci-failure-remediation-sensor.yaml` - Current sensor
- `/crates/healer/src/play/remediate.rs` - Existing remediation engine
- `/crates/healer/src/notify.rs` - Notification module
- `/infra/charts/controller/agent-templates/` - Agent prompt templates
- `/infra/charts/openmemory/values.yaml` - OpenMemory configuration
- `/docs/openmemory-integration-guide.md` - Memory function reference

