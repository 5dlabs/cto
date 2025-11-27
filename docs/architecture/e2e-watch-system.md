# E2E Watch System Architecture

## Overview

The E2E Watch System provides automated end-to-end testing and remediation for Play workflows. It consists of two agents working in coordination:

1. **Monitor Agent (Morgan)** - Submits Play, watches execution, evaluates against acceptance criteria, writes issue reports
2. **Remediation Agent (Rex)** - Reads issue reports, makes fixes, ensures PR merged and deployed before handing back

This system reuses existing infrastructure (Argo Workflows, CodeRun CRDs, shared PVCs) and existing GitHub Apps (Morgan, Rex) with role-specific prompts.

**Key Design Decisions:**

- **Single CLI**: Factory only (no multi-CLI complexity)
- **Models**: GLM for Monitor (lightweight observation), Opus 4.5 for Remediation (complex reasoning)
- **No max iterations**: Loop runs until acceptance criteria pass (infinite)
- **Polling-based**: No sensors needed - agents poll for state changes
- **Target repo**: `5dlabs/cto` (fixes to the platform itself)

## Goals

- **Automated E2E validation**: Run a Play and verify it completes successfully
- **Self-healing**: When failures occur, automatically remediate and retry
- **Full deployment cycle**: Remediation includes PR → CI → merge → ArgoCD sync → cluster verification
- **Template-driven**: Update expected behavior and remediation strategies without code changes
- **Observability**: Same monitoring/logging as other agents

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           E2E Watch Workflow                                     │
│                    (Argo WorkflowTemplate: watch-workflow-template)              │
│                         Entry: play-monitor e2e --task-id X                      │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│   ┌───────────────────────────────────────────────────────────────────────┐     │
│   │                         Shared PVC (/workspace/)                       │     │
│   │  ├── watch/                                                            │     │
│   │  │   ├── status.md              # Current iteration, state             │     │
│   │  │   ├── current-issue.md       # Active issue for remediation         │     │
│   │  │   ├── issue-history.md       # Log of all issues                    │     │
│   │  │   └── acceptance-criteria.md # Expected Play behavior               │     │
│   │  ├── play-artifacts/            # Logs from monitored Play             │     │
│   │  └── repo/                      # Cloned 5dlabs/cto repository         │     │
│   └───────────────────────────────────────────────────────────────────────┘     │
│                              │                                                   │
│                              │ (mounted by both agents)                          │
│                              │                                                   │
│   ┌──────────────────────────┴──────────────────────────┐                       │
│   │                                                      │                       │
│   ▼                                                      ▼                       │
│ ┌─────────────────────────────┐          ┌─────────────────────────────────┐    │
│ │     MONITOR AGENT           │          │      REMEDIATION AGENT          │    │
│ │     (Morgan / GLM)          │          │      (Rex / Opus 4.5)           │    │
│ ├─────────────────────────────┤          ├─────────────────────────────────┤    │
│ │ 1. Submit Play workflow     │          │ 1. Read issue from PVC          │    │
│ │    (argo submit)            │          │ 2. Clone repo, make fix         │    │
│ │ 2. Poll until complete      │          │ 3. cargo fmt/clippy/test        │    │
│ │    (argo get)               │          │ 4. Create branch, push PR       │    │
│ │ 3. Download all logs        │          │ 5. Poll: gh run list            │    │
│ │ 4. Evaluate vs criteria     │          │    (wait for CI to start)       │    │
│ │ 5. Write findings to PVC    │          │ 6. Poll: gh pr checks           │    │
│ │ 6. Exit 0=pass, 1=issues    │          │    (wait for checks to pass)    │    │
│ └──────────────┬──────────────┘          │ 7. Check bug-bot comments       │    │
│                │                          │ 8. gh pr merge --squash         │    │
│                │ Submits                  │ 9. Poll: argocd app get         │    │
│                ▼                          │    (wait for sync)              │    │
│   ┌───────────────────────────┐          │ 10. Poll: kubectl get pods      │    │
│   │       Play Workflow       │          │     (verify controller ready)   │    │
│   │  (Rex → Cleo → Tess →     │          │ 11. Exit 0                      │    │
│   │   Cipher → Atlas)         │          └──────────────┬──────────────────┘    │
│   └───────────────────────────┘                         │                       │
│                                                         │                       │
│   ┌─────────────────────────────────────────────────────┘                       │
│   │                                                                             │
│   │  Argo Workflow Loop Control:                                                │
│   │  ┌──────────────────────────────────────────────────────────────────────┐   │
│   │  │ Monitor exit 0 (success) ──────────────────────► Workflow Succeeds   │   │
│   │  │ Monitor exit 1 (issues)  ───► Remediation ───► Loop back to Monitor  │   │
│   │  └──────────────────────────────────────────────────────────────────────┘   │
│   │                                                                             │
└───┴─────────────────────────────────────────────────────────────────────────────┘
```

## Configuration Schema

### cto-config.json Structure

```json
{
  "defaults": {
    "watch": {
      "repository": "5dlabs/cto",
      "service": "cto-platform",
      "playTemplate": "play-workflow-template",
      
      "monitor": {
        "agent": "5DLabs-Morgan",
        "cli": "factory",
        "model": "glm-4-plus",
        "tools": [
          "kubernetes_listResources",
          "kubernetes_getResource",
          "kubernetes_getPodsLogs",
          "kubernetes_getEvents",
          "github_get_pull_request",
          "github_get_pull_request_status"
        ]
      },
      
      "remediation": {
        "agent": "5DLabs-Rex",
        "cli": "factory",
        "model": "claude-opus-4-5-20251101",
        "tools": [
          "brave_search_brave_web_search",
          "context7_resolve_library_id",
          "context7_get_library_docs",
          "github_create_pull_request",
          "github_push_files",
          "github_create_branch",
          "github_get_file_contents",
          "github_create_or_update_file"
        ]
      }
    }
  }
}
```

### Key Configuration Points

| Field | Description |
|-------|-------------|
| `repository` | Target repository for fixes (`5dlabs/cto`) |
| `monitor.agent` | GitHub App for monitor role (`5DLabs-Morgan`) |
| `monitor.model` | Lightweight model for observation (`glm-4-plus`) |
| `remediation.agent` | GitHub App for remediation role (`5DLabs-Rex`) |
| `remediation.model` | Heavy reasoning model (`claude-opus-4-5-20251101`) |

Note: No `maxIterations` - the loop runs until acceptance criteria pass.

## Agent Templates

### Directory Structure

Templates follow the existing pattern in `infra/charts/controller/agent-templates/`:

```
infra/charts/controller/agent-templates/
└── watch/
    └── factory/
        ├── container-monitor.sh.hbs          # Monitor agent container script
        ├── container-remediation.sh.hbs      # Remediation agent container script
        ├── agents-monitor.md.hbs             # Monitor system prompt (CLAUDE.md)
        └── agents-remediation.md.hbs         # Remediation system prompt
```

### Monitor Agent Behavior

The Monitor Agent (Morgan/GLM) performs these steps:

1. **Submit Play**: `argo submit --from workflowtemplate/play-workflow-template`
2. **Poll Status**: `argo get <workflow> -o json` until phase is Succeeded or Failed
3. **Harvest Logs**: Download logs from all stages
4. **Evaluate**: Compare results against acceptance criteria
5. **Report**: Write findings to `/workspace/watch/current-issue.md`
6. **Exit**: 0 if all criteria pass, 1 if issues found

### Remediation Agent Behavior

The Remediation Agent (Rex/Opus 4.5) performs the full fix-to-deployment cycle:

**Fix Phase:**
```bash
# Read issue, clone repo, make fix
git clone https://github.com/5dlabs/cto /workspace/repo
cd /workspace/repo
# Make targeted code changes
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

**PR Phase:**
```bash
# Create branch and PR
git checkout -b fix/watch-iteration-N-description
git add -A && git commit -m "fix: ..."
git push origin fix/watch-iteration-N-description
gh pr create --title "..." --body "..."
```

**GitHub Polling Flow:**
```bash
# 1. Wait for CI to start (at least one workflow run appears)
while true; do
  RUNS=$(gh run list --branch <branch> --repo 5dlabs/cto --json status)
  if [ "$(echo $RUNS | jq length)" -gt 0 ]; then break; fi
  sleep 10
done

# 2. Wait for all checks to complete (no pending)
while true; do
  CHECKS=$(gh pr checks <pr-number> --repo 5dlabs/cto)
  if ! echo "$CHECKS" | grep -q "pending"; then break; fi
  sleep 30
done

# 3. Check for bug-bot comments
COMMENTS=$(gh api repos/5dlabs/cto/issues/<pr-number>/comments)
# If bug-bot issues found → fix → push → back to step 1

# 4. Check merge status
MERGEABLE=$(gh pr view <pr-number> --repo 5dlabs/cto --json mergeable -q '.mergeable')
# If CONFLICTING → resolve → push → back to step 1

# 5. Merge PR
gh pr merge <pr-number> --repo 5dlabs/cto --squash
```

**Deployment Polling Flow:**
```bash
# 6. Wait for ArgoCD sync
while true; do
  SYNC=$(argocd app get controller -o json | jq -r '.status.sync.status')
  HEALTH=$(argocd app get controller -o json | jq -r '.status.health.status')
  if [ "$SYNC" = "Synced" ] && [ "$HEALTH" = "Healthy" ]; then break; fi
  sleep 30
done

# 7. Verify controller pod is ready with new code
while true; do
  READY=$(kubectl get pods -n cto -l app=agent-controller -o json | \
    jq -r '.items[0].status.conditions[] | select(.type=="Ready") | .status')
  if [ "$READY" = "True" ]; then break; fi
  sleep 10
done
```

**Exit 0** → Argo workflow loops back to Monitor for fresh Play

## Shared PVC Structure

Both agents mount the same PVC at `/workspace/`:

```
/workspace/
├── watch/
│   ├── status.md                 # Current state (phase, iteration)
│   ├── current-issue.md          # Active issue (written by Monitor)
│   ├── issue-history.md          # Append-only log of all issues
│   └── acceptance-criteria.md    # Expected Play behavior
│
├── play-artifacts/               # Captured from Play
│   └── logs/
│       └── <stage>-<pod>.log
│
└── repo/                         # Cloned 5dlabs/cto
    └── <repository contents>
```

### status.md Format

```markdown
# E2E Watch Status

## Current State
- **Phase**: monitoring | remediating | succeeded
- **Iteration**: 3
- **Started**: 2024-01-15T10:00:00Z
- **Last Update**: 2024-01-15T10:35:00Z

## Play Workflow
- **Name**: play-task-42-abc123
- **Status**: Running | Succeeded | Failed
- **Current Stage**: code-quality

## History
| Iteration | Stage Failed | Issue | Fix Applied | Duration |
|-----------|--------------|-------|-------------|----------|
| 1 | implementation | compile error | added import | 5m |
| 2 | code-quality | clippy warning | applied suggestion | 3m |
| 3 | (in progress) | - | - | - |
```

### current-issue.md Format

```markdown
# E2E Issue Detected

## Metadata
- **Task ID**: 42
- **Iteration**: 3
- **Timestamp**: 2024-01-15T10:30:00Z

## Failure Context
- **Stage**: code-quality
- **Failed Step**: cleo-review
- **Pod**: play-42-cleo-xyz
- **Exit Code**: 1

## Error Summary
Clippy found unused import in controller/src/main.rs

## Relevant Logs
```
error: unused import: `std::collections::HashMap`
 --> controller/src/main.rs:5:5
```

## Acceptance Criteria Status
- [x] Implementation completed (PR created)
- [ ] Quality checks passed ← FAILED HERE
- [ ] Tests passed
- [ ] PR merged

## Suggested Remediation
Remove unused import on line 5 of controller/src/main.rs
```

## CLI Interface

### play-monitor e2e Command

```rust
#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...
    
    /// Start E2E watch: monitor Play, remediate failures, loop until success
    E2e {
        /// Task ID for the Play
        #[arg(long)]
        task_id: String,
        
        /// Path to cto-config.json
        #[arg(long, default_value = "cto-config.json")]
        config: String,
        
        /// Target repository (default: 5dlabs/cto)
        #[arg(long, default_value = "5dlabs/cto")]
        repository: String,
        
        /// Dry run - show what would be submitted
        #[arg(long)]
        dry_run: bool,
    },
}
```

### Command Flow

```bash
play-monitor e2e --task-id 42 --config cto-config.json
```

1. Parse `cto-config.json`, extract `defaults.watch` section
2. Validate configuration (agents exist, etc.)
3. Submit `watch-workflow-template` to Argo
4. Stream workflow status events
5. Output JSON events for each phase change

## Binary Distribution

### Cargo Dist Configuration

The `play-monitor` binary is distributed via GitHub releases using Cargo Dist.

Create `monitor/dist-workspace.toml`:
```toml
[workspace]
members = ["cargo:."]

[dist]
cargo-dist-version = "0.28.2"
ci = "github"
installers = ["shell"]
targets = ["aarch64-apple-darwin", "aarch64-unknown-linux-gnu", "x86_64-apple-darwin", "x86_64-unknown-linux-gnu"]
install-path = "CARGO_HOME"
hosting = "github"
install-updater = false
allow-dirty = ["ci"]
```

### Runtime Image Integration

Add to `infra/images/runtime/Dockerfile`:
```dockerfile
# Install play-monitor from CTO monorepo release
ARG PLAY_MONITOR_VERSION=skip
RUN if [ "$PLAY_MONITOR_VERSION" != "skip" ] && [ -n "$PLAY_MONITOR_VERSION" ]; then \
      echo "📦 Installing play-monitor v${PLAY_MONITOR_VERSION}..." && \
      curl --proto '=https' --tlsv1.2 -LsSf \
        "https://github.com/5dlabs/cto/releases/download/play-monitor-v${PLAY_MONITOR_VERSION}/play-monitor-installer.sh" | \
        sh -s -- --yes && \
      mv ~/.cargo/bin/play-monitor /usr/local/bin/play-monitor; \
    fi
```

## Implementation Phases

### Phase 1: Binary Distribution
1. Add `dist-workspace.toml` to monitor crate
2. Set up GitHub Actions release workflow
3. Add play-monitor to runtime Dockerfile

### Phase 2: Agent Templates
1. Create `watch/factory/` directory structure
2. Implement `container-monitor.sh.hbs`
3. Implement `container-remediation.sh.hbs`
4. Create basic agent prompts (to be refined)

### Phase 3: Workflow Template
1. Create `watch-workflow-template.yaml`
2. Implement shared PVC setup
3. Implement infinite monitor→remediate loop
4. Wire up CodeRun creation for both agents

### Phase 4: CLI Integration
1. Add `e2e` subcommand to play-monitor
2. Parse watch config section
3. Implement workflow submission
4. Add status streaming

### Phase 5: Testing
1. Manual E2E test with simple failure scenarios
2. Verify monitor→remediation handoff
3. Verify full deployment cycle (PR → merge → ArgoCD → pod)

## Success Criteria

- [ ] `play-monitor e2e --task-id X` submits watch workflow
- [ ] Monitor agent submits Play and evaluates completion
- [ ] Remediation agent fixes issues and ensures deployment
- [ ] Loop continues until acceptance criteria pass
- [ ] Status visible in Argo UI and via CLI

## Helper Scripts

The Remediation Agent uses predefined helper scripts to minimize token usage and ensure consistent behavior. These scripts are mounted via ConfigMap.

### Library Scripts (`/workspace/scripts/lib/`)

| Script | Purpose |
|--------|---------|
| `common.sh` | Logging, retry logic, polling utilities |
| `github.sh` | PR operations, CI polling, bug-bot queries |
| `argocd.sh` | ArgoCD sync status, health checks |
| `kubernetes.sh` | Pod readiness, deployment status |
| `git.sh` | Clone, branch, commit, push operations |

### Action Scripts (`/workspace/scripts/actions/`)

| Script | Purpose |
|--------|---------|
| `poll-ci.sh` | Wait for all PR checks to pass |
| `check-bugbot.sh` | Query and parse bug-bot comments |
| `merge-pr.sh` | Verify and merge PR |
| `poll-deploy.sh` | Wait for ArgoCD sync + pod readiness |
| `run-validation.sh` | Run cargo fmt, clippy, test |
| `create-fix-pr.sh` | Branch, commit, push, create PR |
| `full-remediation-flow.sh` | Complete fix-to-deployment cycle |

### Usage in Remediation Agent

```bash
# Source library
source /workspace/scripts/lib/common.sh
source /workspace/scripts/lib/github.sh

# Use action scripts
/workspace/scripts/actions/poll-ci.sh --pr-number 123 --timeout 1800
/workspace/scripts/actions/check-bugbot.sh --pr-number 123 --wait
/workspace/scripts/actions/merge-pr.sh --pr-number 123
/workspace/scripts/actions/poll-deploy.sh --app controller --namespace cto
```

## Future Enhancements

### TODO: Context7 MCP Server Integration

Add Context7 MCP server to Watch agents for enhanced codebase understanding:

```json
{
  "tools": [
    {
      "name": "context7",
      "type": "mcp",
      "config": {
        "server": "context7-server",
        "namespace": "cto"
      }
    }
  ]
}
```

This will allow agents to query documentation and examples when fixing issues.
*Waiting for tools configuration changes to be ready.*

### Other Planned Improvements

- **Acceptance criteria templates**: Domain-specific criteria for different workflow types
- **Metrics/observability**: Prometheus metrics for Watch iterations, success rates
- **Slack notifications**: Alert on Watch failures or prolonged loops
- **Cost tracking**: Track token usage per Watch run

## References

- Existing Play workflow: `infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml`
- Factory container pattern: `infra/charts/controller/agent-templates/code/factory/container-base.sh.hbs`
- Tools Cargo Dist config: `tools/dist-workspace.toml`
- Runtime Dockerfile: `infra/images/runtime/Dockerfile`
- Current monitor CLI: `monitor/src/main.rs`
- Watch scripts ConfigMap: `infra/charts/controller/templates/configmaps/watch-scripts-configmap.yaml`
- Helper scripts: `infra/charts/controller/agent-templates/watch/scripts/`
