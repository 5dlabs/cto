# Linear-CTO Integration Workflow

> Complete workflow documentation for Linear integration with the CTO platform, from PRD intake through Play workflow execution.

## Overview

This document maps the complete integration between Linear (project management) and CTO (Cognitive Task Orchestrator) for AI-driven development workflows. The integration uses Linear's Agent API to provide real-time visibility into agent work, while leveraging the CTO platform for orchestration and execution.

## Credential Architecture

### Linear API (Simplified)
- **Single workspace API key** for all Linear operations
- Used for: Project creation, issue management, agent activities, status updates
- Stored in: OpenBao at `cto/linear`
- Environment variable: `LINEAR_API_KEY`

### GitHub Apps (Per-Agent)
- **Each agent has its own GitHub App** for git operations
- Used for: Repository access, PR creation, code commits, branch management
- Stored in: OpenBao at `github-app-{agent}`
- Examples: `github-app-5dlabs-rex`, `github-app-5dlabs-blaze`, etc.

### Key Distinction
| Credential Type | Scope | Purpose |
|----------------|-------|---------|
| Linear API Key | Workspace-wide | All Linear API operations |
| GitHub Apps | Per-agent | Git operations during Play phases |

---

## Phase 1: Intake via MCP Tool

### Trigger
User calls `intake()` MCP tool from their workstation with PRD and architecture documents.

### Actions

1. **Create Linear Project**
   - Uses workspace API key
   - Sets up project with appropriate views
   - Configures project status workflow

2. **Create Initial PRD Issue**
   - Title: Project name from PRD
   - Description: Full PRD content
   - Attachments: Architecture docs, supporting materials
   - Labels: `prd`, `intake`
   - Delegate: Morgan (awaiting assignment)

3. **Set Up Project Views**
   - Task board (by status)
   - Agent assignment view
   - Timeline/milestone view

### Potential Template Usage
- **Project Template**: Pre-configure project with standard views, status workflow, and issue templates
- **Issue Template**: Standard PRD issue format with required sections

---

## Phase 2: Docs/Intake Processing (Linear-Triggered)

### Trigger
User assigns the PRD issue to Morgan in Linear with the `prd` tag.

### Webhook Flow

```
Linear (AgentSessionEvent: created)
    └─→ PM Server (webhook receiver)
          └─→ Create Argo Workflow (intake-template)
                └─→ Morgan Agent Pod
                      ├─→ Parse PRD
                      ├─→ Generate tasks.json
                      ├─→ Create task documentation
                      └─→ Emit activities to Linear
```

### Agent Activities (Linear UI)

| Activity Type | Usage |
|--------------|-------|
| `thought` | Initial acknowledgment, parsing progress |
| `action` | Task generation, file creation |
| `response` | Completion with summary |
| `error` | Failures with details |

### Plan Updates
Morgan updates the session plan as a checklist:

```json
[
  { "content": "Parse PRD document", "status": "completed" },
  { "content": "Extract requirements", "status": "completed" },
  { "content": "Generate task breakdown", "status": "inProgress" },
  { "content": "Create task documentation", "status": "pending" },
  { "content": "Create Linear issues", "status": "pending" }
]
```

### Output
- `tasks.json` with all tasks
- Task documentation in docs repository
- Linear issues created for each task

---

## Phase 3: Play Workflow Execution

### Trigger
MCP `play()` call or automatic progression after intake.

### Task-Based Agent Routing

During **intake**, Morgan analyzes each task and assigns an `agentHint` based on:
1. **Explicit mention** in task title/description (e.g., "Rex: Create API endpoint")
2. **Dependency inheritance** - if all dependencies are handled by the same agent
3. **Keyword inference** - based on technology keywords (rust → Rex, react → Blaze)

**Task 1 is always forced to Bolt** (initial infrastructure provisioning), but additional infrastructure tasks can also be assigned to Bolt throughout the project (e.g., adding a new database, configuring a cache layer).

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           INTAKE (Morgan)                                    │
│  Creates tasks.json with agentHint, language, framework for each task       │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           PLAY WORKFLOW                                      │
│                                                                              │
│   For EACH TASK:                                                            │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │ 1. IMPLEMENTATION AGENT (based on task.agentHint)                   │   │
│   │    ├─ bolt (infrastructure - Task 1 + any additional infra tasks)  │   │
│   │    ├─ rex/grizz/nova (backend tasks)                                │   │
│   │    └─ blaze/tap/spark (frontend tasks)                              │   │
│   │                                                                      │   │
│   │ 2. SUPPORT AGENTS (sequential, run for EVERY task)                  │   │
│   │    ├─ Cleo (quality) ──── receives language/framework context       │   │
│   │    ├─ Cipher (security) ─ receives language/framework context       │   │
│   │    ├─ Tess (testing) ──── receives language/framework context       │   │
│   │    └─ Atlas (integration) final merge gate                          │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Implementation Agents (Task-Routed)

| Agent | Language/Platform | Stack | Notes |
|-------|------------------|-------|-------|
| **Bolt** | Infrastructure | K8s operators, databases, caches | Task 1 forced; can handle additional infra tasks |
| **Rex** | Rust | axum, tokio, sqlx | |
| **Grizz** | Go | chi, grpc, pgx | |
| **Nova** | Node.js/Bun | Elysia, Effect, Drizzle | |
| **Blaze** | React/Web | Next.js, shadcn/ui, TailwindCSS | |
| **Tap** | Mobile | Expo, React Native | |
| **Spark** | Desktop | Electron | |

### Support Agents (Run for Every Task)

| Agent | Role | Context Received |
|-------|------|------------------|
| **Cleo** | Quality Review | `task-language`, `task-framework` for language-specific checks |
| **Cipher** | Security Scan | `task-language`, `task-framework` for language-specific vulnerabilities |
| **Tess** | Testing | `task-language`, `task-framework` for language-specific test tools |
| **Atlas** | Integration | Final merge gate, CI verification |

### Per-Task Execution Flow

1. **Issue Assignment**
   - Task issue assigned to appropriate agent (based on `agentHint`)
   - Agent set as `delegate` (human remains `assignee`)
   - Status moved to first "started" state

2. **Agent Session Created**
   - Linear creates `AgentSession` automatically
   - Webhook (`AgentSessionEvent: created`) sent to PM server

3. **Agent Execution**
   - CodeRun created with:
     - Agent's GitHub App credentials (for git)
     - Linear session ID (for status updates)
   - Linear sidecar runs alongside agent:
     - Streams logs to Linear agent dialog (`emit_thought`)
     - Polls for user input from Linear
     - Updates plan checklist
     - Tracks artifacts

4. **Agent Activities**
   - `thought`: Reasoning, analysis, decisions
   - `action`: Tool calls (edit_file, run_command, etc.)
   - `action` with result: Completed operations
   - `elicitation`: Requests for user clarification
   - `response`: Task completion

5. **User Interaction**
   - User can send input via Linear comment
   - Triggers `AgentSessionEvent: prompted` webhook
   - Input forwarded to agent via FIFO

6. **Support Agent Phases**
   - After implementation completes, support agents run sequentially
   - Each receives `task-language` and `task-framework` for context
   - Cleo → Cipher → Tess → Atlas

7. **Completion**
   - PR created with implementation agent's GitHub App
   - Support agents review and approve
   - Atlas merges after all checks pass

---

## Linear Agent API Integration

### Current Implementation (status-sync.rs)

| Feature | Status | Notes |
|---------|--------|-------|
| `emit_thought()` | ✅ Implemented | Thoughts, progress updates |
| `emit_ephemeral_thought()` | ✅ Implemented | Transient status messages |
| `emit_action()` | ✅ Implemented | Tool invocations |
| `emit_action_complete()` | ✅ Implemented | Tool results |
| `emit_error()` | ✅ Implemented | Error reporting |
| `emit_response()` | ✅ Implemented | Final completion |
| `update_plan()` | ✅ Implemented | Checklist updates |
| `set_external_url()` | ✅ Implemented | Link to Argo workflow |
| `get_session_activities()` | ✅ Implemented | Poll for user input |
| User input polling | ✅ Implemented | Forward to agent FIFO |
| Whip cracking | ✅ Implemented | Progress nudges |
| Artifact trail | ✅ Implemented | File tracking |

### Linear Signals Support

| Signal | Direction | Status | Notes |
|--------|-----------|--------|-------|
| `stop` | Human→Agent | ⚠️ Partial | Need to handle graceful shutdown |
| `auth` | Agent→Human | ❌ Not needed | We use workspace API key |
| `select` | Agent→Human | ❌ Not implemented | Could use for confirmations |

---

## Open Questions

### 1. Stop Signal Handling ✅ IMPLEMENTED
When a user clicks "Send stop request" in Linear:
- The `input_poll_task` in `status-sync.rs` detects `signal: "stop"` in polled activities
- Emits response: "🛑 Stopped as requested. No further changes were made."
- Sets shutdown flag to trigger graceful termination
- HTTP endpoint `/stop` also available for direct stop requests

### 2. Morgan Assignment Auto-Trigger ✅ IMPLEMENTED
When a PRD issue is assigned to Morgan with a "PRD" label:
- `handle_agent_session_created` in `agent_session.rs` detects Morgan + PRD tag
- Automatically extracts intake request from issue description  
- Submits intake workflow via Kubernetes API
- Emits progress thoughts to Linear agent dialog
- Returns `intake_triggered` status with workflow name

To trigger intake via Linear:
1. Create an issue with your PRD content in the description
2. Add the "PRD" label to the issue
3. Assign Morgan as delegate (or @mention Morgan)
4. Intake workflow starts automatically

### 3. Project Templates
Should we create a Linear Project Template for Play workflows?
- Pros: Consistent project structure, predefined views, status workflow
- Cons: Adds manual setup step in Linear

### 4. Issue Templates
Should we create Issue Templates for different task types?
- Implementation task template
- Bug fix template
- Documentation template

### 5. Resume Capability
Linear doesn't have explicit "resume" signal. Options:
- User sends new prompt to resume
- Agent checks for incomplete work on startup

---

## Appendix: Linear API Reference

### Key GraphQL Mutations

```graphql
# Emit agent activity
mutation AgentActivityCreate($input: AgentActivityCreateInput!) {
  agentActivityCreate(input: $input) {
    success
  }
}

# Update session plan
mutation AgentSessionUpdate($id: String!, $input: AgentSessionUpdateInput!) {
  agentSessionUpdate(id: $id, input: $input) {
    success
  }
}

# Create issue
mutation IssueCreate($input: IssueCreateInput!) {
  issueCreate(input: $input) {
    success
    issue { id identifier }
  }
}

# Create project
mutation ProjectCreate($input: ProjectCreateInput!) {
  projectCreate(input: $input) {
    success
    project { id name }
  }
}
```

### Webhook Events

| Event | Action | Description |
|-------|--------|-------------|
| AgentSessionEvent | created | Agent mentioned/delegated |
| AgentSessionEvent | prompted | User sent follow-up message |
| AppUserNotification | issueAssignedToYou | Issue delegated to agent |
| AppUserNotification | issueUnassignedFromYou | Agent removed from issue |

---

## References

- [Linear Agent Interaction Guidelines (AIG)](https://linear.app/developers/aig)
- [Linear Getting Started with Agents](https://linear.app/developers/agents)
- [Linear Developing Agent Interaction](https://linear.app/developers/agent-interaction)
- [Linear Agent Best Practices](https://linear.app/developers/agent-best-practices)
- [Linear Agent Signals](https://linear.app/developers/agent-signals)
- [Linear Project Templates](https://linear.app/docs/project-templates)
- [Linear Issue Templates](https://linear.app/docs/issue-templates)
