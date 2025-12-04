# CI Remediation Hub Design

## Overview

Healer becomes the intelligent triage layer for all CI failures, routing remediation to the most appropriate specialist agent based on failure type.

## Current State

The `ci-remediation-sensor` directly creates `CodeRun` resources with Atlas whenever a GitHub Actions workflow fails:

```
GitHub CI Failure → Argo Sensor → CodeRun (Atlas)
```

**Problems:**
- No intelligent routing - Atlas handles everything
- No deduplication - multiple failures can spawn duplicate fixes
- No tracking - remediation attempts aren't correlated
- No learning - insights from fixes aren't captured

## Proposed State

Healer receives all CI failure events, analyzes them, and routes to the appropriate specialist:

```
GitHub CI Failure → Argo Sensor → Healer → CodeRun (Rex/Blaze/Bolt/Atlas)
```

---

## Agent Specializations

| Agent | Domain | GitHub App | Use Cases |
|-------|--------|------------|-----------|
| **Rex** | Rust | 5DLabs-Rex | Clippy errors, test failures, build errors, Cargo issues |
| **Blaze** | Frontend | 5DLabs-Blaze | JavaScript, TypeScript, npm/pnpm, React, CSS |
| **Bolt** | Infrastructure | 5DLabs-Bolt | Docker builds, Helm charts, K8s manifests, GitOps, YAML |
| **Atlas** | GitHub/General | 5DLabs-Atlas | GitHub API issues, permissions, workflow syntax, fallback |

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

### Security Events (Future)

| Event | Trigger | Use Case | Agent |
|-------|---------|----------|-------|
| **Dependabot alerts** | `created`, `reopened` | Auto-fix dependency vulnerabilities | Bolt |
| **Code scanning alerts** | `created` | Security issue remediation | Atlas |
| **Secret scanning alerts** | `created` | Rotate/revoke leaked secrets | Atlas |
| **Repository vulnerability alerts** | `created` | Dependency security fixes | Bolt |

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
│                        CI Failure Event                             │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
                    ┌─────────────────────────┐
                    │   Analyze Failure Type   │
                    └─────────────────────────┘
                                  │
        ┌─────────────┬───────────┼───────────┬─────────────┐
        ▼             ▼           ▼           ▼             ▼
   ┌─────────┐  ┌──────────┐  ┌────────┐  ┌────────┐  ┌──────────┐
   │  Rust   │  │ Frontend │  │ Infra  │  │ GitHub │  │ Unknown  │
   │ Clippy  │  │   npm    │  │ Docker │  │  API   │  │          │
   │  Test   │  │   pnpm   │  │  Helm  │  │ Perms  │  │          │
   │  Build  │  │   tsx    │  │  K8s   │  │ Syntax │  │          │
   └────┬────┘  └────┬─────┘  └───┬────┘  └───┬────┘  └────┬─────┘
        │            │            │           │            │
        ▼            ▼            ▼           ▼            ▼
   ┌─────────┐  ┌──────────┐  ┌────────┐  ┌────────┐  ┌──────────┐
   │   Rex   │  │  Blaze   │  │  Bolt  │  │ Atlas  │  │  Atlas   │
   └─────────┘  └──────────┘  └────────┘  └────────┘  └──────────┘
```

### Detailed Routing Matrix

| Workflow Pattern | Log Pattern | Changed Files | → Agent |
|------------------|-------------|---------------|---------|
| `*-ci`, `controller-*`, `healer-*` | `clippy`, `cargo test`, `rustc` | `*.rs`, `Cargo.toml` | Rex |
| `frontend-*`, `ui-*` | `npm`, `pnpm`, `tsc`, `eslint` | `*.ts`, `*.tsx`, `*.js`, `package.json` | Blaze |
| `infrastructure-*`, `docker-*`, `helm-*` | `docker build`, `helm`, `kubectl` | `Dockerfile`, `*.yaml`, `Chart.yaml` | Bolt |
| `*-release`, `deploy-*` | `push`, `ghcr.io`, `permission` | `.github/workflows/*` | Atlas |
| (fallback) | (any) | (any) | Atlas |

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
│  │  prompts/ci/rust-fix.hbs                                     │ │
│  │  prompts/ci/frontend-fix.hbs                                 │ │
│  │  prompts/ci/infra-fix.hbs                                    │ │
│  │  prompts/ci/github-fix.hbs                                   │ │
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

