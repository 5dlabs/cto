# Healer: Unified Remediation Hub Design

## Overview

Healer becomes the **single remediation hub** for all issues across the platform. It acts as an intelligent dispatcher with comprehensive tooling access to make informed routing decisions and provide maximum context to specialist agents.

### Key Principle

- **Stitch** = Detection (finds issues, posts review comments)
- **Healer** = Remediation (analyzes, routes, tracks, enriches context)
- **Specialist Agents** = Execution (Rex, Blaze, Bolt, Cipher, Atlas)

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

## Success Metrics

| Metric | Target |
|--------|--------|
| Correct agent routing | >90% |
| Duplicate prevention | 100% |
| Time to fix (P50) | <15 min |
| Fix success rate | >70% |

---

## Open Questions

1. **Model selection per agent?** Should each agent use a different model, or all use the same?
2. **Escalation path?** If an agent fails to fix, should Healer escalate to a different agent or human?
3. **Retry limits?** How many remediation attempts per failure before giving up?
4. **Branch strategy?** Should fixes go to the failing branch or a new fix branch?

---

## Related Files

- `/infra/gitops/resources/sensors/ci-failure-remediation-sensor.yaml` - Current sensor
- `/crates/healer/src/play/remediate.rs` - Existing remediation engine
- `/infra/charts/controller/agent-templates/` - Agent prompt templates

