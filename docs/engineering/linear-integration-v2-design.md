# Linear Integration V2 Design

## Overview

This document outlines the enhanced Linear integration features:
1. **PR Merge → Project & Issues**: Automatically create Linear project and task issues when intake PR is merged
2. **Play Progress Updates**: Update Linear issue status and labels as agents work
3. **2-Way Agent Communication**: Sidecar approach for streaming agent logs to Linear and receiving user input

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Linear Integration                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌─────────────┐     ┌──────────────┐     ┌───────────────────────────┐   │
│   │   GitHub    │────▶│   Linear     │────▶│  Argo Workflows           │   │
│   │  Webhooks   │     │   Service    │     │  (Intake/Play)            │   │
│   └─────────────┘     └──────────────┘     └───────────────────────────┘   │
│         │                    │                         │                    │
│         │                    │                         ▼                    │
│         │                    │              ┌───────────────────────────┐   │
│         │                    │              │   Agent Pod               │   │
│         │                    │              │  ┌─────────┬─────────┐    │   │
│         │                    │              │  │  Main   │ Sidecar │    │   │
│         │                    │              │  │Container│Container│    │   │
│         │                    │              │  │ (Agent) │ (Comms) │    │   │
│         │                    │              │  └────┬────┴────┬────┘    │   │
│         │                    │              │       │         │         │   │
│         │                    │              │   stdout     log file     │   │
│         │                    │              │       │         │         │   │
│         │                    │              └───────┼─────────┼─────────┘   │
│         │                    │                      │         │             │
│         │                    ◀──────────────────────┴─────────┘             │
│         │                    │   Status updates & log streaming             │
│         │                    │                                              │
│         │                    ▼                                              │
│   ┌─────────────┐     ┌──────────────┐                                     │
│   │   GitHub    │◀────│    Linear    │                                     │
│   │    API      │     │     API      │                                     │
│   └─────────────┘     └──────────────┘                                     │
│                              │                                              │
│                              ▼                                              │
│                       Linear Workspace                                      │
│                    ┌────────────────────┐                                  │
│                    │  Project (Repo)    │                                  │
│                    │  ├── Task Issue 1  │                                  │
│                    │  ├── Task Issue 2  │                                  │
│                    │  └── Task Issue N  │                                  │
│                    └────────────────────┘                                  │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Feature 1: PR Merge → Linear Project & Issues

### Trigger
- GitHub webhook: `pull_request` event with `action: merged`
- Filter: PRs from intake workflow (branch pattern: `intake-*` or PR label)

### Flow
1. Receive GitHub PR merge webhook
2. Parse PR body for tasks.json reference or fetch from repo
3. Create Linear **Project** with:
   - Title: Repository name (e.g., "agent-sandbox")
   - Description: Link to PR, summary of tasks
   - Team: From original PRD issue or configured default
4. Create Linear **Issues** for each task:
   - Title: Task title
   - Description: Task description + subtasks as checklist
   - Labels: `task`, language labels (rust, typescript, etc.)
   - Project: Link to created project
   - Relations: Subtasks as sub-issues OR checklist items

### API Endpoints Needed
```
POST /webhooks/github - GitHub webhook receiver
```

### Linear API Calls
```graphql
# Create project
mutation CreateProject($input: ProjectCreateInput!) {
  projectCreate(input: $input) {
    project { id name }
  }
}

# Create issue
mutation CreateIssue($input: IssueCreateInput!) {
  issueCreate(input: $input) {
    issue { id identifier }
  }
}

# Link issue to project
mutation UpdateIssue($id: String!, $projectId: String!) {
  issueUpdate(id: $id, input: { projectId: $projectId }) {
    issue { id }
  }
}
```

### Configuration
```yaml
# Environment variables
GITHUB_WEBHOOK_SECRET: <secret>
LINEAR_DEFAULT_TEAM_ID: <team-id>
```

---

## Feature 2: Play Progress → Linear Status Updates

### Events to Track
| Event | Linear Update |
|-------|---------------|
| Play started | Issue → "In Progress", add "agent:working" label |
| Agent assigned | Update delegate field |
| Subtask complete | Check off subtask, update progress |
| PR created | Add attachment with PR link |
| PR merged | Issue → "Done" |
| Error | Add "agent:error" label, comment with error |

### Implementation Options

#### Option A: Workflow Callbacks (Current)
- Argo workflow steps call Linear service callbacks
- Pros: Explicit, controlled
- Cons: Only at step boundaries, not real-time

#### Option B: Status Sync Endpoint (Exists)
- `/status/linear-sync` endpoint receives updates from sidecar
- Pros: Can be called anytime
- Cons: Requires sidecar to manage state

#### Option C: Kubernetes Events
- Watch CodeRun status changes
- Pros: Native to K8s, reliable
- Cons: Coarse-grained

### Recommended: Hybrid Approach
1. Use callbacks for major state transitions
2. Use sidecar for real-time log streaming
3. Use K8s events as backup/reconciliation

### Label Schema
```
agent:pending      - Waiting for agent
agent:working      - Agent actively processing
agent:blocked      - Waiting for user input
agent:pr-created   - PR created, awaiting review
agent:complete     - Successfully completed
agent:error        - Error occurred
```

---

## Feature 3: 2-Way Agent Communication via Sidecar

### Requirements
1. **Keep stdout for model activity** - Don't interfere with Claude/agent output
2. **Stream logs to Linear** - Post agent progress to agent dialog
3. **2-way communication** - Accept user input via Claude's `--input` FIFO

### Sidecar Container Design

```yaml
# Sidecar container spec
name: linear-comms
image: linear-sidecar:local
env:
  - name: LINEAR_SESSION_ID
    value: "$(SESSION_ID)"
  - name: LINEAR_OAUTH_TOKEN
    valueFrom:
      secretKeyRef:
        name: linear-secrets
        key: LINEAR_OAUTH_TOKEN
  - name: LOG_FILE_PATH
    value: /workspace/agent.log
  - name: INPUT_FIFO_PATH
    value: /workspace/agent-input.jsonl
volumeMounts:
  - name: workspace
    mountPath: /workspace
```

### Log Streaming
```
Main Container                    Sidecar
┌─────────────┐                  ┌─────────────┐
│   Claude    │                  │   Linear    │
│   Agent     │                  │   Comms     │
│             │                  │             │
│  stdout ───────▶ terminal      │             │
│             │                  │             │
│  tee ──────────▶ agent.log ──────▶ tail -f   │
│             │                  │      │      │
│             │                  │      ▼      │
│             │                  │ POST to     │
│             │                  │ Linear API  │
└─────────────┘                  └─────────────┘
```

### Input Handling (Claude's --input flag)
```bash
# Main container runs Claude with input FIFO
claude --input /workspace/agent-input.jsonl ...

# Sidecar writes to FIFO when user sends message in Linear
echo '{"type":"user","content":"Please also add tests"}' >> /workspace/agent-input.jsonl
```

### Sidecar Script (Rust or Shell)
```rust
// Pseudocode for sidecar
async fn main() {
    let session_id = env::var("LINEAR_SESSION_ID")?;
    let log_path = env::var("LOG_FILE_PATH")?;
    let input_path = env::var("INPUT_FIFO_PATH")?;
    
    // Task 1: Stream logs to Linear
    tokio::spawn(async move {
        let mut reader = tail_file(log_path).await;
        let mut buffer = String::new();
        let mut last_post = Instant::now();
        
        while let Some(line) = reader.next().await {
            buffer.push_str(&line);
            
            // Batch and post every 5 seconds or on key events
            if last_post.elapsed() > Duration::from_secs(5) 
               || line.contains("✓") || line.contains("ERROR") {
                post_to_linear(&session_id, &buffer).await;
                buffer.clear();
                last_post = Instant::now();
            }
        }
    });
    
    // Task 2: Listen for Linear webhook/polling for user messages
    tokio::spawn(async move {
        loop {
            if let Some(msg) = poll_linear_for_input(&session_id).await {
                write_to_fifo(&input_path, &msg).await;
            }
            sleep(Duration::from_secs(2)).await;
        }
    });
    
    // Keep running
    tokio::signal::ctrl_c().await?;
}
```

### Linear API for Agent Activities
```graphql
# Emit thought (log line)
mutation EmitThought($sessionId: String!, $body: String!) {
  agentSessionEmitActivity(
    sessionId: $sessionId
    input: { thought: { body: $body } }
  ) { success }
}

# Emit action (completed step)
mutation EmitAction($sessionId: String!, $action: String!, $parameter: String!) {
  agentSessionEmitActivity(
    sessionId: $sessionId
    input: { action: { action: $action, parameter: $parameter } }
  ) { success }
}
```

---

## Implementation Plan

### Phase 1: GitHub Webhook → Project & Issues (Week 1)
- [ ] Add `/webhooks/github` endpoint to linear service
- [ ] Implement PR merge detection
- [ ] Create Linear project from PR
- [ ] Create Linear issues from tasks.json
- [ ] Link issues to parent PRD issue

### Phase 2: Play Progress Updates (Week 1-2)
- [ ] Define label schema for agent status
- [ ] Update play workflow to emit status at key points
- [ ] Implement label updates in linear service
- [ ] Add PR link attachments when PRs are created

### Phase 3: Sidecar for 2-Way Comms (Week 2-3)
- [ ] Create linear-sidecar container image
- [ ] Implement log streaming (tail → Linear)
- [ ] Implement input FIFO writing
- [ ] Update play workflow template to include sidecar
- [ ] Test 2-way communication flow

### Phase 4: Polish & Testing (Week 3)
- [ ] Error handling and retry logic
- [ ] Rate limiting for Linear API
- [ ] End-to-end testing
- [ ] Documentation

---

## Open Questions

1. **Batching logs**: How often should we post to Linear? Every N seconds? On key events?
2. **Log format**: Should we filter/format logs before posting? (e.g., strip ANSI codes)
3. **Input latency**: How quickly does user input need to reach the agent?
4. **Session timeout**: What happens if agent session expires?
5. **FluentD**: Should we use FluentD instead of a custom sidecar? Pros: mature, cons: config complexity

---

## Alternatives Considered

### FluentD for Log Shipping
- Pro: Battle-tested, flexible output plugins
- Con: Need custom Linear output plugin, heavier weight
- Decision: Start with simple sidecar, consider FluentD if scaling issues

### Kubernetes Events
- Pro: Native, reliable
- Con: Too coarse for real-time updates
- Decision: Use as backup/reconciliation only

### Linear GraphQL Subscriptions
- Pro: Real-time updates from Linear
- Con: Linear doesn't expose subscriptions for agents
- Decision: Use polling for now, switch if Linear adds support

