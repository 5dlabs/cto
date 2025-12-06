# Linear Integration - Implementation Status

## Overview

This document tracks the implementation status of the Linear integration for the CTO platform. The integration enables:
- Submitting PRDs through Linear's UI to trigger intake workflows
- Initiating play workflows by delegating task issues to the CTO agent
- Real-time status updates in Linear via Agent Activities

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│  Linear UI                                                      │
│  - PRD Issues (labeled: prd, intake)                           │
│  - Task Issues (labeled: task, cto-task)                       │
│  - Agent delegation (@CTO-Agent)                               │
└───────────────────────────┬─────────────────────────────────────┘
                            │ Webhook
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│  CloudFlare Tunnel                                              │
│  linear-webhooks.5dlabs.ai → TunnelBinding                     │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│  crates/linear/ (Standalone Service)                           │
│  - Webhook receiver with signature verification                 │
│  - Intake handler → Argo intake workflow                       │
│  - Play handler → Argo play workflow                           │
│  - Callback endpoints for workflow completion                   │
│  - Agent Activity emission (thought, action, response, error)  │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│  Argo Workflows                                                 │
│  - project-intake: PRD → tasks.json → Linear sub-issues        │
│  - play-workflow: Task implementation with agent orchestration │
└─────────────────────────────────────────────────────────────────┘
```

## Implementation Status

### ✅ PR1: Foundation (COMPLETE)

| Component | Status | Description |
|-----------|--------|-------------|
| `crates/linear/src/client.rs` | ✅ | GraphQL client for Linear API |
| `crates/linear/src/models.rs` | ✅ | Type definitions for Linear entities |
| `crates/linear/src/webhooks.rs` | ✅ | Webhook signature verification |
| `crates/linear/src/activities.rs` | ✅ | Agent Activity emission |
| `infra/vault/secrets/linear.yaml` | ✅ | Vault secret definition |

**Linear API Client capabilities:**
- Issue CRUD (create, read, update)
- Issue relations (blocked-by dependencies)
- Team workflow states
- Label management (get/create)
- Agent Activity emission (thought, action, response, error)
- Document retrieval

### ✅ PR2: Intake Integration (COMPLETE)

| Component | Status | Description |
|-----------|--------|-------------|
| `crates/linear/src/handlers/intake.rs` | ✅ | PRD extraction and workflow trigger |
| `crates/linear/src/handlers/callbacks.rs` | ✅ | Workflow completion handling |
| `crates/linear/src/server.rs` | ✅ | Webhook routing |
| Task issue creation | ✅ | Creates sub-issues from tasks.json |
| Dependency mapping | ✅ | Creates blocked-by relations |

**Intake workflow:**
1. User delegates PRD issue (labeled `prd` or `intake`) to @CTO-Agent
2. Linear sends `AgentSessionEvent.created` webhook
3. Service emits "Processing PRD..." thought
4. Service creates ConfigMap with PRD content
5. Service submits `project-intake` Argo workflow
6. Workflow completes → calls `/callbacks/intake-complete`
7. Service creates Linear sub-issues for each task
8. Service emits completion response with summary

### ✅ PR3: Play Integration (COMPLETE)

| Component | Status | Description |
|-----------|--------|-------------|
| `crates/linear/src/handlers/play.rs` | ✅ | Task extraction and play trigger |
| Play workflow trigger | ✅ | Submits play workflow for task |
| Stop signal handling | ✅ | Cancels workflows on stop signal |
| `crates/linear/src/bin/linear.rs` | ✅ | Standalone service binary |
| `crates/linear/src/config.rs` | ✅ | Service configuration |

**Play workflow:**
1. User delegates task issue (labeled `task` or `cto-task`) to @CTO-Agent
2. Linear sends `AgentSessionEvent.created` webhook
3. Service emits "Starting task implementation..." thought
4. Service extracts task ID from issue title
5. Service submits `play-workflow` Argo workflow
6. Service emits action activity with workflow name

**Stop signal:**
1. User sends stop signal in Linear
2. Linear sends `AgentSessionEvent.prompted` with stop signal
3. Service finds workflows by `linear-session` label
4. Service stops running workflows
5. Service emits "Workflow cancelled" response

### ✅ PR3: Infrastructure (COMPLETE)

| Component | Status | Description |
|-----------|--------|-------------|
| `infra/charts/linear/` | ✅ | Helm chart for Kubernetes deployment |
| `infra/gitops/resources/cloudflare-tunnel/linear-binding.yaml` | ✅ | CloudFlare tunnel configuration |
| Linear webhook registration | 🔲 | Configure in Linear settings (manual step)

### 🔲 PR3: Morgan PM Status Sync (PENDING)

| Component | Status | Description |
|-----------|--------|-------------|
| Morgan PM Linear output | 🔲 | Sync task status to Linear |
| Issue state transitions | 🔲 | Update Linear issue states |

## Remaining Work

### 1. Morgan PM Integration

Morgan PM currently syncs status to GitHub Projects. Extension needed to also sync to Linear:

```rust
// In Morgan PM daemon
async fn sync_status_to_linear(
    linear_client: &LinearClient,
    issue_id: &str,
    status: TaskStatus,
) -> Result<()> {
    let state_id = linear_client
        .get_team_workflow_states(team_id)
        .await?
        .into_iter()
        .find(|s| s.state_type == status.to_linear_state_type())
        .map(|s| s.id);
    
    if let Some(state_id) = state_id {
        linear_client.update_issue(IssueUpdateInput {
            id: issue_id.to_string(),
            state_id: Some(state_id),
            ..Default::default()
        }).await?;
    }
    Ok(())
}
```

### 2. Linear Webhook Configuration

In Linear workspace settings:
1. Go to Settings → Integrations → Webhooks
2. Add webhook URL: `https://linear-webhooks.5dlabs.ai/webhooks/linear`
3. Set webhook secret (same as `LINEAR_WEBHOOK_SECRET`)
4. Enable events: `AgentSessionEvent`

## Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `LINEAR_ENABLED` | Enable Linear integration | Yes |
| `LINEAR_OAUTH_TOKEN` | Linear API token | Yes |
| `LINEAR_WEBHOOK_SECRET` | Webhook signing secret | Yes |
| `LINEAR_PORT` | HTTP server port (default: 8081) | No |
| `LINEAR_MAX_TIMESTAMP_AGE_MS` | Max webhook age (default: 60000) | No |
| `NAMESPACE` | Kubernetes namespace (default: cto) | No |
| `DEFAULT_REPOSITORY` | Default GitHub repo for play | No |

## Testing Checklist

### Intake Flow
- [ ] Delegate PRD issue to @CTO-Agent
- [ ] Verify "Processing PRD..." thought appears
- [ ] Wait for intake workflow completion
- [ ] Verify task sub-issues created
- [ ] Verify dependency relations created
- [ ] Verify completion summary appears

### Play Flow
- [ ] Delegate task issue to @CTO-Agent
- [ ] Verify "Starting task implementation..." thought appears
- [ ] Verify play workflow starts
- [ ] Test stop signal cancels workflow
- [ ] Verify "Workflow cancelled" response

### Error Handling
- [ ] Test with missing labels → helpful guidance response
- [ ] Test with invalid PRD → error activity
- [ ] Test workflow failure → error propagation

## Files Changed

### New Files
- `crates/linear/src/bin/linear.rs`
- `crates/linear/src/config.rs`
- `crates/linear/src/server.rs`
- `crates/linear/src/handlers/mod.rs`
- `crates/linear/src/handlers/intake.rs`
- `crates/linear/src/handlers/play.rs`
- `crates/linear/src/handlers/callbacks.rs`
- `infra/vault/secrets/linear.yaml`
- `infra/charts/linear/Chart.yaml`
- `infra/charts/linear/values.yaml`
- `infra/charts/linear/templates/_helpers.tpl`
- `infra/charts/linear/templates/deployment.yaml`
- `infra/charts/linear/templates/service.yaml`
- `infra/charts/linear/templates/serviceaccount.yaml`
- `infra/charts/linear/templates/rbac.yaml`
- `infra/gitops/resources/cloudflare-tunnel/linear-binding.yaml`

### Modified Files
- `crates/linear/Cargo.toml` - Added server dependencies
- `crates/linear/src/lib.rs` - Export new modules

### Deleted Files (moved to linear crate)
- `crates/controller/src/webhooks/mod.rs`
- `crates/controller/src/webhooks/linear.rs`
- `crates/controller/src/webhooks/handlers/mod.rs`
- `crates/controller/src/webhooks/handlers/intake.rs`
- `crates/controller/src/webhooks/handlers/play.rs`
- `crates/controller/src/webhooks/handlers/callbacks.rs`

### Controller Cleanup
- `crates/controller/Cargo.toml` - Removed linear dependency
- `crates/controller/src/lib.rs` - Removed webhooks module
- `crates/controller/src/bin/agent_controller.rs` - Removed Linear routes

