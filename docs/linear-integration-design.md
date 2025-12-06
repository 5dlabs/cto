# Linear Integration Design Document

> **Status:** Discovery / Ideation  
> **Author:** CTO Platform Team  
> **Created:** 2025-12-06  
> **Last Updated:** 2025-12-06

## Executive Summary

This document explores integrating the CTO Platform with [Linear](https://linear.app) as the primary user interface for project planning and task execution. Linear's new **Agent APIs** (currently in Developer Preview) provide a native way for AI agents to participate in workflows, making it an ideal frontend for our intake and play workflows.

### Key Value Propositions

1. **Eliminate custom UI development** — Linear provides kanban boards, roadmaps, mobile apps, and team collaboration out of the box
2. **Native agent experience** — Users see agent thinking, can interact mid-workflow, and control execution via Linear's UI
3. **Team-first collaboration** — Multiple stakeholders can review, comment, and redirect agent work naturally
4. **Reduced context switching** — Teams already using Linear don't need a separate tool for AI-assisted development

---

## Table of Contents

1. [Linear Agent Capabilities](#linear-agent-capabilities)
2. [Integration Options](#integration-options)
3. [Proposed Architecture](#proposed-architecture)
4. [Intake Workflow via Linear](#intake-workflow-via-linear)
5. [Play Workflow via Linear](#play-workflow-via-linear)
6. [Data Model Mapping](#data-model-mapping)
7. [Implementation Phases](#implementation-phases)
8. [Open Questions](#open-questions)
9. [Appendix: Linear API Reference](#appendix-linear-api-reference)

---

## Linear Agent Capabilities

Linear's Agent APIs ([documentation](https://linear.app/developers/agents)) enable applications to act as workspace members with their own identity. Key features:

### Agent Sessions

An `AgentSession` tracks the lifecycle of an agent's work on a task:

| State | Description |
|-------|-------------|
| `pending` | Session created, awaiting agent response |
| `active` | Agent is working |
| `awaitingInput` | Agent needs user input (elicitation) |
| `error` | Agent encountered an error |
| `complete` | Work finished |

Sessions are created automatically when:
- A user **@mentions** the agent in an issue or document
- A user **delegates** (assigns) an issue to the agent

### Agent Activities

Agents communicate via typed activities:

| Type | Purpose | Example |
|------|---------|---------|
| `thought` | Internal reasoning, status updates | "Analyzing PRD requirements..." |
| `action` | Tool invocations with optional results | "Running tests...", result: "24/24 passed" |
| `elicitation` | Request user input | "Which repository should I use?" |
| `response` | Final output or completion | "✅ PR ready for review" |
| `error` | Report failures | "Build failed: missing dependency" |

### Signals

Optional metadata that controls agent behavior:

| Signal | Direction | Purpose |
|--------|-----------|---------|
| `stop` | Human → Agent | Halt work immediately |
| `auth` | Agent → Human | Request account linking |
| `select` | Agent → Human | Present options for selection |

### Webhooks

Agents receive webhooks for:
- `AgentSessionEvent.created` — New mention or delegation
- `AgentSessionEvent.prompted` — User sent follow-up message
- `AppUserNotification` — Issue changes, reactions, assignments
- `PermissionChange` — Team access changes

**Response requirements:**
- First activity must be sent within **10 seconds** of receiving webhook
- Webhook handler must respond within **5 seconds**
- Sessions become stale after **30 minutes** of inactivity

---

## Integration Options

### Hybrid Approach (Recommended)

Use **Issues** for the primary PRD and tasks, and **Documents** for supporting design documentation on complex projects.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                       LINEAR WORKSPACE                                   │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                     PROJECT: TeamSync API                        │    │
│  │                                                                  │    │
│  │  ISSUES (Actionable)              DOCUMENTS (Reference)          │    │
│  │  ──────────────────               ────────────────────           │    │
│  │                                                                  │    │
│  │  📋 PRD: TeamSync API             📄 Architecture Overview       │    │
│  │     └─ Primary requirements          └─ System design diagrams   │    │
│  │     └─ @CTO-Agent delegated          └─ Tech stack decisions     │    │
│  │                                                                  │    │
│  │  ✅ Task 1: Setup foundation      📄 API Specification           │    │
│  │  ✅ Task 2: Auth system              └─ Endpoint definitions     │    │
│  │  🔄 Task 3: Team management          └─ Request/response schemas │    │
│  │  ⏳ Task 4: Task board                                           │    │
│  │  ⏳ Task 5: Notifications         📄 Database Schema             │    │
│  │  ⏳ Task 6: Dashboard                └─ ERD diagrams             │    │
│  │                                      └─ Migration strategy       │    │
│  │                                                                  │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

#### When to Use Each

| Content Type | Linear Primitive | Rationale |
|--------------|------------------|-----------|
| PRD (requirements) | **Issue** | Actionable, triggers intake, status tracking |
| Tasks | **Issue** (sub-issues) | Actionable, triggers play, PR linking |
| Architecture docs | **Document** | Long-form, diagrams, reference material |
| API specifications | **Document** | Detailed schemas, examples |
| Database design | **Document** | ERDs, migration plans |
| ADRs (Architecture Decision Records) | **Document** | Historical context, rationale |
| Runbooks | **Document** | Operational procedures |

#### Linking Issues and Documents

Documents can be linked to issues via:
1. **Issue attachments** — Attach document URL to issue
2. **Issue description** — Reference document in markdown: `See [Architecture](doc-url)`
3. **Document mentions** — @mention issues within documents

### Issue-Based PRD (Primary)

The issue description holds the core PRD content:

**Pros:**
- Natural fit — issues are the core Linear primitive
- Rich metadata — labels, custom fields, projects
- Status tracking and delegation built-in
- Mobile app support
- Commenting and collaboration

**Trigger mechanism:**
1. User creates issue with PRD in description
2. User adds label `prd` or `intake-ready`
3. User delegates issue to CTO Agent
4. Agent receives `AgentSessionEvent.created` webhook

### Documents for Supporting Content

Linear Documents are used for detailed design documentation:

**Pros:**
- Better for lengthy technical content
- Native document editing with formatting
- Can be linked to multiple issues
- Suitable for diagrams and detailed specs

**Use cases:**
- Architecture documents referenced during intake
- API specs that inform task generation
- Database schemas for implementation context
- Design decisions that span multiple tasks

---

## Proposed Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                            LINEAR WORKSPACE                              │
│                                                                          │
│  ┌────────────────┐    ┌────────────────┐    ┌────────────────┐        │
│  │  PRD Issue     │    │  Task Issue    │    │  Task Issue    │        │
│  │  ────────────  │    │  ────────────  │    │  ────────────  │        │
│  │  Delegate: @CTO│    │  Delegate: @CTO│    │  Status: Done  │        │
│  │  Label: prd    │    │  Label: task   │    │  PR: #42       │        │
│  │  Project: Acme │    │  Blocks: #3    │    │                │        │
│  │  📎 Arch Doc   │    │                │    │                │        │
│  │  📎 API Spec   │    │                │    │                │        │
│  └───────┬────────┘    └───────┬────────┘    └────────────────┘        │
│          │                     │                                        │
│          │                     │             ┌────────────────┐        │
│          │                     │             │  📄 Documents   │        │
│          │                     │             │  ────────────  │        │
│          │ AgentSessionEvent   │             │  Architecture  │        │
│          │ (created)           │             │  API Spec      │        │
│          │    +                │             │  DB Schema     │        │
│          │ Linked docs fetched │             │                │        │
│          │                     │             └────────────────┘        │
└──────────┼─────────────────────┼────────────────────────────────────────┘
           │                     │
           ▼                     ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                        WEBHOOK INGRESS                                   │
│                   (Cloudflare Tunnel / K8s)                             │
│                                                                          │
│  - Signature verification (HMAC-SHA256)                                 │
│  - Timestamp validation (< 60s)                                         │
│  - Rate limiting                                                        │
└─────────────────────────────────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                        CTO LINEAR CONTROLLER                             │
│                                                                          │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐      │
│  │  Session Router  │  │  Activity        │  │  Linear API      │      │
│  │  ──────────────  │  │  Emitter         │  │  Client          │      │
│  │                  │  │  ──────────────  │  │  ──────────────  │      │
│  │  - Parse webhook │  │  - thought()     │  │  - Create issues │      │
│  │  - Route to      │  │  - action()      │  │  - Update status │      │
│  │    handler       │  │  - elicitation() │  │  - Add comments  │      │
│  │  - Handle stop   │  │  - response()    │  │  - Query tasks   │      │
│  │    signal        │  │  - error()       │  │                  │      │
│  └────────┬─────────┘  └────────┬─────────┘  └──────────────────┘      │
│           │                     │                                       │
│           ▼                     │                                       │
│  ┌──────────────────────────────┼───────────────────────────────┐      │
│  │  Workflow Handlers           │                                │      │
│  │  ─────────────────           │                                │      │
│  │                              │                                │      │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐           │      │
│  │  │   Intake    │  │    Play     │  │   Status    │           │      │
│  │  │   Handler   │  │   Handler   │  │   Handler   │           │      │
│  │  │             │  │             │  │             │           │      │
│  │  │ - Parse PRD │  │ - Get task  │  │ - Sync      │           │      │
│  │  │ - Trigger   │  │ - Trigger   │  │   workflow  │           │      │
│  │  │   workflow  │──│   workflow  │──│   state to  │───────────┼──────│
│  │  │ - Create    │  │ - Stream    │  │   Linear    │           │      │
│  │  │   issues    │  │   progress  │  │             │           │      │
│  │  └──────┬──────┘  └──────┬──────┘  └─────────────┘           │      │
│  └─────────┼────────────────┼───────────────────────────────────┘      │
└────────────┼────────────────┼───────────────────────────────────────────┘
             │                │
             ▼                ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                          ARGO WORKFLOWS                                  │
│                                                                          │
│  ┌─────────────────────────┐    ┌─────────────────────────────────┐    │
│  │  intake-workflow        │    │  play-workflow                   │    │
│  │  ─────────────────      │    │  ─────────────────               │    │
│  │                         │    │                                   │    │
│  │  1. Parse PRD           │    │  1. Rex (implementation)         │    │
│  │  2. Generate tasks      │    │  2. Cleo (quality review)        │    │
│  │  3. Analyze complexity  │    │  3. Tess (testing)               │    │
│  │  4. Create docs         │    │  4. Atlas (security scan)        │    │
│  │  5. Submit PR           │    │  5. Bolt (deployment)            │    │
│  │                         │    │                                   │    │
│  │  Output: tasks.json     │    │  Output: PR merged               │    │
│  └─────────────────────────┘    └─────────────────────────────────┘    │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### The Core Flow

```
┌──────────────────────────────────────────────────────────────────────────┐
│                          THE SIMPLE FLOW                                  │
│                                                                           │
│  1. USER creates PRD Issue in Linear                                      │
│  2. USER delegates issue to @CTO-Agent                                    │
│                                                                           │
│  3. LINEAR sends webhook (POST) ─────────────────────────────────────┐   │
│                                                                       │   │
│                                                                       ▼   │
│     ┌─────────────────────────────────────────────────────────────────┐  │
│     │  CTO WEBHOOK ENDPOINT                                           │  │
│     │  POST https://cto.example.com/webhooks/linear                   │  │
│     │                                                                  │  │
│     │  Body: { action: "created", agentSession: { issue: {...} } }    │  │
│     └──────────────────────────────┬──────────────────────────────────┘  │
│                                    │                                      │
│  4. CTO extracts PRD from issue    │                                      │
│     description + linked docs      │                                      │
│                                    ▼                                      │
│     ┌─────────────────────────────────────────────────────────────────┐  │
│     │  INTAKE WORKFLOW                                                 │  │
│     │  - Parse PRD                                                     │  │
│     │  - Generate tasks.json                                           │  │
│     │  - Create agent prompts                                          │  │
│     └──────────────────────────────┬──────────────────────────────────┘  │
│                                    │                                      │
│  5. CTO creates issues from tasks  │                                      │
│                                    ▼                                      │
│     ┌─────────────────────────────────────────────────────────────────┐  │
│     │  CREATE ISSUES (Linear API + GitHub API)                         │  │
│     │                                                                  │  │
│     │  For each task in tasks.json:                                    │  │
│     │    - Create Linear sub-issue under PRD issue                     │  │
│     │    - Create GitHub issue (mirror)                                │  │
│     │    - Set up dependency relationships                             │  │
│     └─────────────────────────────────────────────────────────────────┘  │
│                                                                           │
│  6. CTO updates PRD issue status to "Done"                               │
│                                                                           │
└──────────────────────────────────────────────────────────────────────────┘
```

### Webhook Endpoint Requirements

We need to expose a public HTTPS endpoint for Linear to POST webhooks to:

```
POST https://cto.5dlabs.com/webhooks/linear
```

**Requirements:**
- Must respond within 5 seconds (acknowledge quickly, process async)
- Must return HTTP 200 on success
- Must verify signature using webhook secret
- Must validate timestamp (reject if >60s old)

**Endpoint Implementation:**

```rust
// Simplified webhook handler
#[axum::debug_handler]
async fn linear_webhook(
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, AppError> {
    // 1. Verify signature
    let signature = headers.get("linear-signature")
        .ok_or(AppError::Unauthorized)?;
    verify_hmac_sha256(&body, signature, &WEBHOOK_SECRET)?;
    
    // 2. Parse payload
    let payload: WebhookPayload = serde_json::from_slice(&body)?;
    
    // 3. Validate timestamp
    if (Utc::now().timestamp_millis() - payload.webhook_timestamp).abs() > 60_000 {
        return Err(AppError::StaleWebhook);
    }
    
    // 4. Acknowledge immediately (Linear expects response in 5s)
    tokio::spawn(async move {
        handle_webhook_async(payload).await;
    });
    
    Ok(StatusCode::OK)
}

async fn handle_webhook_async(payload: WebhookPayload) {
    match payload.action.as_str() {
        "created" => handle_agent_session_created(payload).await,
        "prompted" => handle_agent_session_prompted(payload).await,
        _ => {}
    }
}
```

**Deployment Options:**

| Option | Pros | Cons |
|--------|------|------|
| K8s Ingress + Service | Uses existing infra, co-located with controller | Needs public ingress setup |
| Cloudflare Worker | Low latency, no infra changes, auto-scaling | Separate codebase, needs API to call controller |
| Cloudflare Tunnel | Uses existing tunnel, no new ingress | May add latency |

### Component Responsibilities

#### Webhook Ingress
- Verify Linear webhook signatures using HMAC-SHA256
- Validate `webhookTimestamp` is within 60 seconds
- Route to appropriate controller instance
- Handle retries (Linear retries at 1min, 1hr, 6hr intervals)

#### CTO Linear Controller
- **Session Router**: Parse incoming webhooks, route to handlers, handle `stop` signals
- **Activity Emitter**: Send Agent Activities back to Linear with proper typing
- **Linear API Client**: GraphQL client for creating/updating issues, querying data

#### Workflow Handlers
- **Intake Handler**: Extract PRD from issue, trigger intake workflow, create task issues
- **Play Handler**: Map issue to task, trigger play workflow, stream progress
- **Status Handler**: Sync Argo workflow state to Linear issue status

---

## Intake Workflow via Linear

### Processing Documents During Intake

When the agent processes a PRD issue, it can also pull in linked documents as additional context:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    INTAKE WITH DOCUMENTS                                 │
│                                                                          │
│  1. User delegates PRD Issue to @CTO-Agent                              │
│                                                                          │
│  2. Agent extracts:                                                      │
│     ├─ PRD content from issue description                               │
│     ├─ Linked documents (architecture, API spec, etc.)                  │
│     └─ Guidance from workspace/team settings                            │
│                                                                          │
│  3. Agent fetches document content via Linear API:                      │
│     ┌────────────────────────────────────────────────────────────────┐  │
│     │  query IssueWithDocuments($issueId: String!) {                 │  │
│     │    issue(id: $issueId) {                                       │  │
│     │      description                                               │  │
│     │      attachments {                                             │  │
│     │        nodes {                                                 │  │
│     │          url           # Check for linear.app/docs links       │  │
│     │          title                                                 │  │
│     │        }                                                       │  │
│     │      }                                                         │  │
│     │    }                                                           │  │
│     │  }                                                             │  │
│     └────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  4. For each linked document, fetch content:                            │
│     ┌────────────────────────────────────────────────────────────────┐  │
│     │  query DocumentContent($documentId: String!) {                 │  │
│     │    document(id: $documentId) {                                 │  │
│     │      title                                                     │  │
│     │      content          # Markdown content                       │  │
│     │    }                                                           │  │
│     │  }                                                             │  │
│     └────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  5. Pass combined context to intake workflow:                           │
│     {                                                                    │
│       "prd_content": "...",           // From issue description         │
│       "architecture_content": "...",  // From linked Architecture doc   │
│       "api_spec_content": "...",      // From linked API Spec doc       │
│       "additional_context": [...]     // Other linked documents         │
│     }                                                                    │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

#### Document Discovery Methods

The agent can discover linked documents through:

1. **Issue Attachments** — Documents attached to the PRD issue
2. **Description Links** — Markdown links in issue description: `[Architecture](/docs/arch-123)`
3. **Project Documents** — All documents in the same Linear project
4. **Explicit Custom Field** — Custom field containing document IDs

#### Architecture Document Handling

The existing intake workflow already supports an `architecture_content` parameter. This maps cleanly:

| Current Intake | Linear Source |
|----------------|---------------|
| `prd_content` | Issue description |
| `architecture_content` | Linked "Architecture" document |
| `api_spec_content` (new) | Linked "API Spec" document |

### Intake Output Structure

When intake completes, it produces a `.tasks/` directory with the following structure:

```
project-name/
└── .tasks/
    ├── tasks/
    │   └── tasks.json          # All tasks with metadata, dependencies, priorities
    ├── docs/
    │   ├── task-1/
    │   │   ├── prompt.md       # Agent prompt (markdown)
    │   │   ├── prompt.xml      # Agent prompt (XML)
    │   │   └── acceptance.md   # Acceptance criteria
    │   ├── task-2/
    │   │   └── ...
    │   └── task-N/
    │       └── ...
    ├── reports/                # Complexity analysis, etc.
    └── state.json              # Workflow state tracking
```

#### tasks.json Schema

```json
{
  "metadata": {
    "taskCount": 6,
    "completedCount": 0,
    "version": "1.0.0",
    "lastModified": "2025-12-06T10:40:57Z"
  },
  "tasks": [
    {
      "id": "1",
      "title": "Setup project foundation and database infrastructure",
      "description": "Initialize Rust/Axum project with PostgreSQL...",
      "details": "1. Create Cargo workspace with axum 0.7...\n2. Setup PostgreSQL...",
      "dependencies": [],
      "priority": "high",
      "status": "pending",
      "testStrategy": "Unit tests for health endpoints, integration tests...",
      "agentHint": "rex",
      "subtasks": []
    }
  ]
}
```

This output is used to create issues in Linear (and optionally GitHub).

### User Journey

```
┌──────────────────────────────────────────────────────────────────────────┐
│  STEP 1: USER creates PRD Issue in Linear                                │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   ┌────────────────────────────────────────┐                            │
│   │ Title: TeamSync API - Project Intake   │                            │
│   │ Description: [PRD markdown content]    │                            │
│   │ Labels: prd, intake                    │                            │
│   │ Project: Q1 Initiatives                │                            │
│   │ Attachments: Architecture.doc          │                            │
│   └────────────────────────────────────────┘                            │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌──────────────────────────────────────────────────────────────────────────┐
│  STEP 2: USER delegates issue to @CTO-Agent                              │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   Linear automatically sends webhook to CTO endpoint:                    │
│   POST https://cto.5dlabs.com/webhooks/linear                           │
│                                                                          │
│   {                                                                      │
│     "action": "created",                                                 │
│     "agentSession": {                                                    │
│       "id": "session-123",                                               │
│       "issue": { "id": "issue-456", "title": "TeamSync API..." }        │
│     }                                                                    │
│   }                                                                      │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌──────────────────────────────────────────────────────────────────────────┐
│  STEP 3: CTO acknowledges (within 10 seconds)                            │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   Activity: thought                                                      │
│   "📋 Received PRD for **TeamSync API**. Starting intake process..."    │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌──────────────────────────────────────────────────────────────────────────┐
│  STEP 4: CTO runs intake workflow                                        │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   4a. Extract PRD from issue description                                 │
│   4b. Fetch linked documents (Architecture, API spec)                    │
│   4c. Submit to Argo intake workflow                                     │
│                                                                          │
│   Activity: action                                                       │
│   action: "Running intake workflow"                                      │
│   parameter: "TeamSync API + Architecture doc"                           │
│                                                                          │
│   ┌────────────────────────────────────────┐                            │
│   │  ARGO WORKFLOW: intake                 │                            │
│   │  - Parse PRD                           │                            │
│   │  - Analyze complexity                  │                            │
│   │  - Generate tasks.json (6 tasks)       │                            │
│   │  - Create agent prompts                │                            │
│   └────────────────────────────────────────┘                            │
│                                                                          │
│   Activity: action (result)                                              │
│   action: "Running intake workflow"                                      │
│   result: "Generated 6 tasks with dependencies"                          │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌──────────────────────────────────────────────────────────────────────────┐
│  STEP 5: CTO creates issues from tasks.json                              │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   For each task in tasks.json:                                           │
│                                                                          │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │  LINEAR: Create sub-issue                                        │   │
│   │  - Title: "Task 1: Setup project foundation..."                  │   │
│   │  - Description: details + acceptance criteria                    │   │
│   │  - Parent: PRD issue                                             │   │
│   │  - Labels: priority:high, agent:rex                              │   │
│   │  - Blocking relationships from dependencies                      │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │  GITHUB: Create mirror issue (optional)                          │   │
│   │  - Title: "Task 1: Setup project foundation..."                  │   │
│   │  - Body: details + acceptance + Linear link                      │   │
│   │  - Labels: priority:high, agent:rex, cto-task                    │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│   Activity: action                                                       │
│   action: "Creating task issues"                                         │
│   result: "Created TSK-1 through TSK-6 (Linear) + #42-#47 (GitHub)"     │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌──────────────────────────────────────────────────────────────────────────┐
│  STEP 6: CTO completes intake                                            │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   Activity: response                                                     │
│   "✅ **Intake complete!**                                              │
│                                                                          │
│   ## Summary                                                             │
│   - **6 tasks** created in [TeamSync project](link)                     │
│   - **GitHub issues**: #42-#47                                          │
│                                                                          │
│   ## Task Dependencies                                                   │
│   ```                                                                    │
│   TSK-1 (Setup) ── TSK-2 (Auth) ── TSK-3 (Teams) ── TSK-4 (Tasks)       │
│                                           └────── TSK-5 (Notifications)  │
│                                                          └── TSK-6 (UI)  │
│   ```                                                                    │
│                                                                          │
│   ## Next Steps                                                          │
│   Delegate any task to me to begin implementation."                      │
│                                                                          │
│   Update PRD issue status → "Done"                                       │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

### Issue Metadata Extraction

The agent should extract/infer these parameters from the Linear issue:

| Linear Field | CTO Parameter | Extraction Method |
|--------------|---------------|-------------------|
| Issue description | `prd_content` | Direct extraction |
| Custom field: Repository | `repository` | Custom field value |
| Custom field: Project Name | `project_name` | Custom field or issue title |
| Label: `local` | `local` mode | Label presence |
| Guidance field | Additional context | Workspace/team settings |

### Task Issue Creation

After intake generates `tasks.json`, the agent creates child issues:

```graphql
mutation CreateTaskIssue($input: IssueCreateInput!) {
  issueCreate(input: $input) {
    success
    issue {
      id
      identifier
      url
    }
  }
}
```

Input for each task:
```json
{
  "teamId": "team-id",
  "title": "Task 1: Set up authentication framework",
  "description": "## Description\n${task.description}\n\n## Implementation Details\n${task.details}\n\n## Acceptance Criteria\n${task.testStrategy}",
  "labelIds": ["task-label-id"],
  "projectId": "project-id",
  "priority": 2,
  "estimate": 3,
  "parentId": "parent-prd-issue-id"
}
```

### Dependency Mapping

TaskMaster dependencies map to Linear's blocking relationships:

```graphql
mutation AddBlockingRelation($input: IssueRelationCreateInput!) {
  issueRelationCreate(input: $input) {
    success
  }
}
```

```json
{
  "issueId": "TSK-5",
  "relatedIssueId": "TSK-1",
  "type": "blocks"
}
```

### GitHub Issue Creation (Mirror)

In addition to Linear issues, we can create GitHub issues for native PR linking:

```bash
# Using GitHub CLI or API
gh issue create \
  --repo "5dlabs/teamsync" \
  --title "Task 1: Setup project foundation and database infrastructure" \
  --body "$(cat <<'EOF'
## Description

Initialize Rust/Axum project with PostgreSQL and Redis integration, establishing core infrastructure for the TeamSync API

## Implementation Details

1. Create Cargo workspace with axum 0.7, sqlx, redis dependencies
2. Setup PostgreSQL connection pool with sqlx migrations
3. Configure Redis client for sessions and rate limiting
4. Implement health check endpoints (/health/live, /health/ready)
5. Add structured JSON logging with tracing and trace IDs
6. Create Docker multi-stage build with Rust 1.75+
7. Setup basic error handling and middleware stack

## Acceptance Criteria

- [ ] Unit tests for health endpoints
- [ ] Integration tests for DB/Redis connectivity
- [ ] Docker build completes successfully
- [ ] Container starts and passes health checks

## Metadata

- **Priority:** High
- **Agent:** Rex (implementation)
- **Dependencies:** None
- **Linear Issue:** [TSK-1](https://linear.app/workspace/issue/TSK-1)
EOF
)" \
  --label "priority:high" \
  --label "agent:rex" \
  --label "cto-task"
```

#### GitHub Issue Body Template

```markdown
## Description

${task.description}

## Implementation Details

${task.details}

## Acceptance Criteria

${task.testStrategy.split(', ').map(c => `- [ ] ${c}`).join('\n')}

## Metadata

- **Priority:** ${task.priority}
- **Agent:** ${task.agentHint}
- **Dependencies:** ${task.dependencies.length ? task.dependencies.map(d => `#${d}`).join(', ') : 'None'}
- **Linear Issue:** [${linearIssue.identifier}](${linearIssue.url})
```

#### Cross-Linking Strategy

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           CROSS-LINKING                                  │
│                                                                          │
│  LINEAR                           GITHUB                                 │
│  ──────                           ──────                                 │
│                                                                          │
│  TSK-1 ◄──────────────────────► #42 (issue)                             │
│    │   (custom field: gh_issue)    │   (body contains Linear link)      │
│    │                               │                                     │
│    │                               │                                     │
│    └───────────────────────────────┼─► PR #55 (closes #42)              │
│        (attachment/comment)        │      │                              │
│                                    │      │                              │
│  When PR merged:                   │      │                              │
│  - TSK-1 status → Done             │      │                              │
│  - TSK-1 gets PR link              ◄──────┘                              │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

The agent creates PRs with `Closes #42` (GitHub issue number) so merging automatically:
1. Closes the GitHub issue
2. Webhook triggers Linear status update

---

## Play Workflow via Linear

### How Play Gets Triggered

After intake creates task issues, user triggers play on the **PRD issue** (not individual tasks). Play handles all task orchestration internally:

```
┌──────────────────────────────────────────────────────────────────────────┐
│                    TRIGGERING PLAY                                        │
│                                                                           │
│  After intake, you have task issues in Linear:                           │
│                                                                           │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │  PRD: TeamSync API ✅ Intake Done                                    │ │
│  │    │                                                                 │ │
│  │    ├── TSK-1: Setup project foundation      ⏳ Todo                  │ │
│  │    ├── TSK-2: Auth system                   ⏳ Todo                  │ │
│  │    ├── TSK-3: Team management               ⏳ Todo                  │ │
│  │    ├── TSK-4: Task board                    ⏳ Todo                  │ │
│  │    ├── TSK-5: Notifications                 ⏳ Todo                  │ │
│  │    └── TSK-6: Dashboard                     ⏳ Todo                  │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                           │
│  To start implementation:                                                 │
│                                                                           │
│  User comments on PRD: "@CTO-Agent start play" (or re-delegates)         │
│      │                                                                    │
│      └──► Linear sends webhook                                           │
│               │                                                           │
│               └──► CTO triggers PLAY WORKFLOW                            │
│                        │                                                  │
│                        │  Play handles everything:                        │
│                        │  - Task ordering & dependencies                  │
│                        │  - Parallel execution                            │
│                        │  - Agent orchestration (Rex→Cleo→Tess→etc)      │
│                        │                                                  │
│                        └──► CTO just updates Linear status as tasks done │
│                                                                           │
└──────────────────────────────────────────────────────────────────────────┘
```

### Morgan PM — The Status Sync Engine

Morgan PM is an existing agent that runs as a **daemon during play workflows**, monitoring state and syncing to GitHub. For Linear integration, we'd extend Morgan to also sync to Linear.

```
┌──────────────────────────────────────────────────────────────────────────┐
│                    MORGAN PM - HOW IT WORKS                               │
│                                                                           │
│  Morgan PM launches as Step 0 of play workflow (daemon mode):            │
│                                                                           │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │  1. INITIALIZATION PHASE (runs once)                                │ │
│  │     a. Clone docs repo, read tasks.json                             │ │
│  │     b. Create GitHub Project + Issues for each task                 │ │
│  │     c. Set up custom fields (Current Agent, Stage, Priority)        │ │
│  │     d. Store task→issue mapping in /shared/morgan-pm/               │ │
│  │                                                                      │ │
│  │  2. MONITORING PHASE (loops every 30s)                              │ │
│  │     a. Query Kubernetes workflow state via kubectl                  │ │
│  │        - current-stage label (implementation, quality, testing...)  │ │
│  │        - workflow phase (Running, Succeeded, Failed)                │ │
│  │     b. Map stage → agent (Rex, Cleo, Cipher, Tess)                  │ │
│  │     c. Update GitHub Project fields via GraphQL                     │ │
│  │     d. Update issue labels for filtering                            │ │
│  │                                                                      │ │
│  │  3. COMPLETION (when parent workflow finishes)                      │ │
│  │     a. Final sync of all task statuses                              │ │
│  │     b. Exit daemon                                                   │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                           │
│  Stage → Agent Mapping (from Kubernetes labels):                         │
│  ─────────────────────────────────────────────                           │
│  implementation         → Rex (Implementation)                           │
│  quality-in-progress    → Cleo (Quality)                                │
│  security-in-progress   → Cipher (Security)                              │
│  testing-in-progress    → Tess (QA)                                     │
│  Succeeded              → Complete ✅                                    │
│                                                                           │
└──────────────────────────────────────────────────────────────────────────┘
```

### Linear Integration Options for Morgan PM

We have three options for syncing play status to Linear:

```
┌──────────────────────────────────────────────────────────────────────────┐
│  OPTION A: Extend Morgan PM to sync to both GitHub AND Linear            │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Morgan PM (daemon)                                                      │
│      │                                                                   │
│      ├──► GitHub: Update project fields (existing)                      │
│      │                                                                   │
│      └──► Linear: Update issue status via GraphQL (NEW)                 │
│                                                                          │
│  Pros: Single source of truth, less complexity                          │
│  Cons: Morgan needs Linear API credentials, bigger blast radius         │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────────┐
│  OPTION B: Morgan emits events, separate Linear sync service consumes    │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Morgan PM (daemon)                                                      │
│      │                                                                   │
│      ├──► GitHub: Update project fields (existing)                      │
│      │                                                                   │
│      └──► Event: Publish to Redis/NATS (NEW)                            │
│              │                                                           │
│              ▼                                                           │
│         Linear Sync Service                                              │
│              │                                                           │
│              └──► Linear: Update issue status                           │
│                                                                          │
│  Pros: Decoupled, Linear service can be independently deployed          │
│  Cons: More moving parts, event infrastructure needed                   │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────────┐
│  OPTION C: Parallel Linear PM daemon (separate from Morgan)              │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Play Workflow                                                           │
│      │                                                                   │
│      ├──► Morgan PM (daemon) → GitHub sync (existing)                   │
│      │                                                                   │
│      └──► Linear PM (daemon) → Linear sync (NEW)                        │
│                                                                          │
│  Both poll Kubernetes workflow state independently                       │
│                                                                          │
│  Pros: Clean separation, Linear logic isolated                          │
│  Cons: Duplicate polling logic, two daemons running                     │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘

Recommendation: Start with OPTION A (extend Morgan) for simplicity.
If Linear becomes complex, refactor to OPTION B (event-driven).
```

### What Morgan Needs to Sync to Linear

For each task status change, Morgan would need to:

```rust
// Pseudo-code for Linear status update

struct LinearStatusUpdate {
    issue_id: String,           // Linear issue ID (from task→issue mapping)
    status: String,             // "Todo", "In Progress", "Done"
    agent: Option<String>,      // Current agent working (for display)
    pr_url: Option<String>,     // Link to PR when created
}

async fn update_linear_task_status(update: LinearStatusUpdate) {
    // 1. Map workflow stage to Linear status
    let linear_status = match current_stage {
        "pending" => "Todo",
        "implementation" | "quality-in-progress" | "testing-in-progress" => "In Progress",
        "Succeeded" => "Done",
        "Failed" => "Blocked",
    };
    
    // 2. Find Linear workflow state ID for the status
    let state_id = get_linear_workflow_state_id(team_id, linear_status).await?;
    
    // 3. Update Linear issue via GraphQL
    linear_client.issue_update(IssueUpdateInput {
        id: update.issue_id,
        state_id: Some(state_id),
    }).await?;
    
    // 4. Optionally add comment with progress
    if let Some(agent) = update.agent {
        linear_client.comment_create(CommentCreateInput {
            issue_id: update.issue_id,
            body: format!("🤖 {} is now working on this task", agent),
        }).await?;
    }
    
    // 5. Attach PR when created
    if let Some(pr_url) = update.pr_url {
        linear_client.attachment_create(AttachmentCreateInput {
            issue_id: update.issue_id,
            url: pr_url,
            title: "Pull Request",
        }).await?;
    }
}
```

### Task → Issue Mapping Storage

Morgan already stores task-to-GitHub-issue mappings. For Linear, we'd extend this:

```json
// /shared/morgan-pm/task-issue-map.json (extended)
{
  "1": {
    "github_issue_number": 123,
    "github_item_id": "PVTI_...",
    "github_node_id": "I_...",
    "linear_issue_id": "issue-uuid-1",      // NEW
    "linear_issue_identifier": "TSK-1"       // NEW
  },
  "2": {
    "github_issue_number": 124,
    "github_item_id": "PVTI_...",
    "github_node_id": "I_...",
    "linear_issue_id": "issue-uuid-2",
    "linear_issue_identifier": "TSK-2"
  }
}
```

### Linear's Role During Play

Linear integration is minimal during play execution:

```
┌──────────────────────────────────────────────────────────────────────────┐
│                    LINEAR DURING PLAY                                     │
│                                                                           │
│  1. INITIATE: User triggers play via PRD issue                           │
│                                                                           │
│  2. STATUS UPDATES: CTO pushes status to Linear as tasks complete        │
│                                                                           │
│     Play workflow internally:          Linear sees:                       │
│     ─────────────────────────          ────────────                       │
│     Task 1 started                 ──► TSK-1 status → "In Progress"      │
│     Task 1 PR created              ──► TSK-1 gets PR link                │
│     Task 1 merged                  ──► TSK-1 status → "Done"             │
│     Task 2,3 started (parallel)    ──► TSK-2,3 status → "In Progress"   │
│     Task 2 merged                  ──► TSK-2 status → "Done"             │
│     ... etc                                                               │
│                                                                           │
│  3. COMPLETION: All tasks done → PRD status → "Complete"                 │
│                                                                           │
│  That's it! Play handles orchestration, Linear is just the dashboard.    │
│                                                                           │
└──────────────────────────────────────────────────────────────────────────┘
```

### Distinguishing Intake vs Play

When CTO receives a webhook, how does it know whether to run intake or play?

```
┌──────────────────────────────────────────────────────────────────────────┐
│                    WEBHOOK ROUTING LOGIC                                  │
│                                                                           │
│  Webhook received for PRD issue:                                          │
│                                                                           │
│  Check PRD status / context:                                              │
│     ├── No tasks exist yet? ──► Run INTAKE                               │
│     └── Tasks already exist? ──► Run PLAY                                │
│                                                                           │
│  OR use explicit commands in comment:                                     │
│     ├── "@CTO-Agent intake" or delegate with no tasks ──► INTAKE         │
│     └── "@CTO-Agent play" or "@CTO-Agent start" ──► PLAY                 │
│                                                                           │
│  OR use labels:                                                           │
│     ├── Add "run-intake" label ──► INTAKE                                │
│     └── Add "run-play" label ──► PLAY                                    │
│                                                                           │
└──────────────────────────────────────────────────────────────────────────┘
```

### User Journey

```
┌──────────────────────────────────────────────────────────────────────────┐
│  STEP 1: USER triggers play on PRD issue                                 │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   User comments: "@CTO-Agent start play"                                 │
│   OR re-delegates PRD issue after intake complete                        │
│                                                                          │
│   Linear sends webhook → CTO receives                                    │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌──────────────────────────────────────────────────────────────────────────┐
│  STEP 2: CTO acknowledges and starts play                                │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   Activity: thought                                                      │
│   "🚀 Starting play workflow for **TeamSync API** (6 tasks)"            │
│                                                                          │
│   Activity: action                                                       │
│   action: "Launching play workflow"                                      │
│   parameter: "parallel_execution=true, repository=5dlabs/teamsync"       │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌──────────────────────────────────────────────────────────────────────────┐
│  STEP 3: Play runs (handles orchestration internally)                    │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   Play workflow handles:                                                 │
│   - Task ordering & dependencies                                         │
│   - Parallel execution where possible                                    │
│   - Agent orchestration (Rex → Cleo → Tess → Atlas → Bolt)              │
│                                                                          │
│   CTO streams high-level progress:                                       │
│                                                                          │
│   Activity: action                                                       │
│   action: "Task 1: Setup foundation"                                     │
│   result: "In progress..."                                               │
│                                                                          │
│   → Update Linear: TSK-1 status → "In Progress"                         │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌──────────────────────────────────────────────────────────────────────────┐
│  STEP 4: CTO pushes status updates to Linear as tasks complete           │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   Task 1 PR merged:                                                      │
│   → Update Linear: TSK-1 status → "Done", attach PR #42                 │
│   → Activity: "✅ Task 1 complete (PR #42)"                             │
│                                                                          │
│   Tasks 2,3 started (parallel):                                          │
│   → Update Linear: TSK-2, TSK-3 status → "In Progress"                  │
│                                                                          │
│   Task 2 PR merged:                                                      │
│   → Update Linear: TSK-2 status → "Done", attach PR #43                 │
│                                                                          │
│   ... continues until all tasks done ...                                 │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌──────────────────────────────────────────────────────────────────────────┐
│  STEP 5: Play complete                                                   │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   Activity: response                                                     │
│   "✅ **Play complete!**                                                │
│                                                                          │
│   ## Summary                                                             │
│   - 6/6 tasks completed                                                  │
│   - 6 PRs merged                                                         │
│   - All tests passing                                                    │
│                                                                          │
│   ## PRs                                                                 │
│   - PR #42: Setup foundation                                             │
│   - PR #43: Auth system                                                  │
│   - PR #44: Team management                                              │
│   - PR #45: Task board                                                   │
│   - PR #46: Notifications                                                │
│   - PR #47: Dashboard                                                    │
│                                                                          │
│   Project ready for deployment!"                                         │
│                                                                          │
│   → Update Linear: PRD status → "Complete"                              │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

### Handling User Interactions Mid-Workflow

Users can interact during execution:

#### Stop Signal
```json
{
  "action": "prompted",
  "agentActivity": {
    "content": { "type": "prompt", "body": "Stop" },
    "signals": ["stop"]
  }
}
```

Agent response:
1. Cancel Argo workflow
2. Emit activity:
   ```json
   {
     "type": "response",
     "body": "🛑 Stopped. Workflow cancelled at Rex phase.\n\nPartial changes may exist in branch `feature/task-1`."
   }
   ```

#### Clarification Request
User: "Which branch should this target?"

Agent response:
```json
{
  "type": "elicitation",
  "body": "Which branch should I target for this PR?",
  "signal": "select",
  "signalMetadata": {
    "options": [
      { "value": "main" },
      { "value": "develop" },
      { "value": "feature/q1-release" }
    ]
  }
}
```

#### Adding Context
User: "Make sure to use the existing AuthService class"

Agent:
1. Inject context into active workflow
2. Acknowledge:
   ```json
   {
     "type": "thought",
     "body": "📝 Noted: Will integrate with existing `AuthService` class"
   }
   ```

---

## Data Model Mapping

### Intake Output → Issue Trackers

After intake completes, the `.tasks/tasks/tasks.json` contains rich task data that can be synced to **both** Linear and GitHub:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         tasks.json                                       │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │  {                                                                 │  │
│  │    "id": "1",                                                      │  │
│  │    "title": "Setup project foundation...",                         │  │
│  │    "description": "Initialize Rust/Axum...",                       │  │
│  │    "details": "1. Create Cargo workspace...",                      │  │
│  │    "dependencies": [],                                             │  │
│  │    "priority": "high",                                             │  │
│  │    "status": "pending",                                            │  │
│  │    "testStrategy": "Unit tests for health...",                     │  │
│  │    "agentHint": "rex"                                              │  │
│  │  }                                                                 │  │
│  └───────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
                    │                              │
                    ▼                              ▼
        ┌───────────────────────┐      ┌───────────────────────┐
        │    LINEAR ISSUE       │      │    GITHUB ISSUE       │
        │    ─────────────      │      │    ─────────────      │
        │                       │      │                       │
        │  Title: Task title    │      │  Title: Task title    │
        │  Description: details │      │  Body: details +      │
        │  + acceptance         │      │  acceptance criteria  │
        │  Priority: mapped     │      │  Labels: priority,    │
        │  Status: workflow     │      │  agent-hint           │
        │  Blocks: deps         │      │  Milestone: project   │
        │  Project: PRD project │      │  Linked PR: auto      │
        │  Labels: agent-hint   │      │                       │
        │  Delegate: CTO Agent  │      │  References: #deps    │
        └───────────────────────┘      └───────────────────────┘
```

#### Task Field Mapping

| tasks.json Field | Linear Issue | GitHub Issue |
|------------------|--------------|--------------|
| `id` | Custom field or external ID | Issue number (auto) |
| `title` | Issue title | Issue title |
| `description` | Description (summary) | Body (opening paragraph) |
| `details` | Description (implementation) | Body (## Implementation) |
| `dependencies` | Blocking relationships | "Depends on #X" in body |
| `priority` | Priority (1-4 scale) | Label: `priority:high` |
| `status` | Workflow state | Open/Closed + Labels |
| `testStrategy` | Description (acceptance) | Body (## Acceptance Criteria) |
| `agentHint` | Label: `agent:rex` | Label: `agent:rex` |
| `subtasks` | Sub-issues | Task list in body |

#### Sync Strategies

**Option A: Linear as Primary, GitHub as Mirror**
- Linear is the source of truth for status/assignment
- GitHub issues created for PR linking convenience
- Status synced Linear → GitHub (one-way)

**Option B: GitHub as Primary, Linear as View**
- GitHub issues are source of truth (native PR linking)
- Linear issues created for project management views
- Status synced GitHub → Linear (one-way)

**Option C: Bidirectional Sync**
- Both systems are kept in sync
- Changes in either propagate to the other
- More complex, risk of conflicts

**Recommendation:** Option A (Linear primary) since:
- Linear has superior project management UX
- Agent interaction happens in Linear
- GitHub issues just provide PR linking

### Linear → CTO Platform

| Linear Entity | CTO Concept | Notes |
|---------------|-------------|-------|
| Workspace | Organization | Top-level container |
| Team | Repository/Service | Maps to a codebase |
| Project | Project (intake unit) | Groups related tasks |
| Cycle | Sprint/Milestone | Time-boxed work |
| Issue (PRD) | PRD Document | Intake input |
| Issue (Task) | TaskMaster Task | Play unit of work |
| Sub-issue | Subtask | Nested work items |
| Issue Delegate | Agent assignment | Triggers workflow |
| Issue Assignee | Human owner | Reviewer/approver |
| Issue Status | Task status | Synced bidirectionally |
| Blocking relation | Task dependency | `dependsOn` in tasks.json |
| Label | Task metadata | Priority, type, etc. |
| Custom Field | Config parameters | Repository, model, etc. |
| Agent Session | Workflow execution | Tracks agent work |
| Agent Activity | Progress updates | Real-time feedback |

### CTO Platform → Linear

| CTO Event | Linear Action |
|-----------|---------------|
| Intake complete | Create task issues, link dependencies |
| Task started | Update issue status to "In Progress" |
| Agent phase complete | Emit action activity with result |
| Task complete | Update status to "Done", link PR |
| Task failed | Update status to "Blocked", emit error |
| PR merged | Update issue status, trigger downstream |

---

## Implementation Phases

### Phase 1: Foundation (2-3 weeks)

**Goal:** Basic agent infrastructure and intake trigger

**Deliverables:**
- [ ] Linear OAuth App registration with agent scopes
- [ ] Webhook receiver service (signature verification, routing)
- [ ] Linear GraphQL client library (Rust or TypeScript)
- [ ] Agent Activity emitter with all activity types
- [ ] Basic intake trigger: delegate issue → run intake
- [ ] Simple response: completion message with PR link

**Technical tasks:**
```
crates/linear/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── client.rs        # GraphQL client
│   ├── webhooks.rs      # Webhook parsing & verification
│   ├── activities.rs    # Activity emission
│   └── models.rs        # Linear entity types
```

### Phase 2: Intake Integration (2-3 weeks)

**Goal:** Full intake workflow with task issue creation

**Deliverables:**
- [ ] PRD extraction from issue description
- [ ] Parameter extraction from custom fields/labels
- [ ] Linked document discovery and fetching
- [ ] Architecture document content extraction
- [ ] Intake workflow trigger with progress streaming
- [ ] Task issue creation from `tasks.json`
- [ ] Dependency relationship creation (blocking/blocked)
- [ ] Issue status updates during workflow
- [ ] GitHub issue mirror creation (optional)

**User-visible features:**
- Create PRD issue → link design docs → delegate → see tasks created
- Agent acknowledges linked documents in initial thought
- Real-time progress via Agent Activities
- Task issues with proper metadata and relationships
- GitHub issues created with Linear cross-links

### Phase 3: Play Integration (3-4 weeks)

**Goal:** Execute play via Linear and sync status back

**Deliverables:**
- [ ] Play trigger via PRD issue (comment or re-delegation)
- [ ] High-level progress streaming via Agent Activities
- [ ] Extend Morgan PM to sync status to Linear
  - [ ] Add Linear API client to Morgan scripts
  - [ ] Extend task-issue-map.json with Linear issue IDs
  - [ ] Map workflow stages to Linear workflow states
  - [ ] Update Linear issue status on each poll cycle
- [ ] PR attachment to Linear issues when created
- [ ] Stop signal handling (cancel Argo workflow)

**Morgan PM Changes:**
```bash
# New in morgan-pm.sh.hbs

# Initialize: Create Linear issues (in addition to GitHub)
for task in tasks.json:
  github_issue = create_github_issue(task)
  linear_issue = create_linear_issue(task)  # NEW
  store_mapping(task.id, github_issue, linear_issue)

# Monitor loop: Update both GitHub and Linear
while workflow_running:
  for task in tasks:
    stage = get_workflow_stage(task.id)
    agent = map_stage_to_agent(stage)
    status = map_stage_to_status(stage)
    
    update_github_project(task, agent, status)
    update_linear_issue(task, agent, status)  # NEW
```

**User-visible features:**
- Trigger play from Linear PRD issue → see all tasks progress
- Linear issues update in real-time as agents work
- PRs attached to Linear issues when created
- Stop button in Linear → workflow cancelled

### Phase 4: Advanced Features (2-3 weeks)

**Goal:** Enhanced UX and automation

**Deliverables:**
- [ ] Elicitation support (repository selection, clarifications)
- [ ] Parallel execution visualization
- [ ] Automatic next-task delegation (optional)
- [ ] Custom field sync for advanced parameters
- [ ] Guidance/system prompt integration

**User-visible features:**
- Agent asks clarifying questions when needed
- Project view shows execution DAG
- Configure agent behavior via workspace settings

### Phase 5: Production Hardening (2 weeks)

**Goal:** Reliability and observability

**Deliverables:**
- [ ] Retry handling for webhook failures
- [ ] Graceful degradation when Linear is unavailable
- [ ] Metrics and alerting for agent health
- [ ] Rate limiting compliance
- [ ] Audit logging for agent actions

---

## Open Questions

### Product Questions

1. **Should intake create a Linear Project automatically?**
   - Option A: Always create a new project for each PRD
   - Option B: Let user specify existing project
   - Option C: Use project from parent issue

2. **How should we handle multi-repository projects?**
   - Some PRDs may span multiple repos
   - Linear issues are team-scoped (team ≈ repo)

3. **How granular should Linear status updates be during play?**
   - Option A: Update each task status as it progresses (In Progress → Done)
   - Option B: Only update on completion (Todo → Done)
   - Option C: Batch updates (update all changed statuses every N minutes)
   - **Consideration:** More updates = better visibility but more API calls

4. **Should users be able to stop/pause play mid-execution?**
   - Option A: Yes, via `stop` signal in Linear → cancel Argo workflow
   - Option B: No, play runs to completion once started
   - **Consideration:** Stop signal support requires workflow to check for cancellation

5. **How should Morgan PM integrate with Linear?**
   - Option A: Extend Morgan to sync both GitHub AND Linear (recommended for v1)
   - Option B: Morgan emits events, separate Linear sync service consumes
   - Option C: Parallel Linear PM daemon alongside Morgan
   - **Consideration:** Option A is simplest, Option B allows independent scaling

6. **Should Linear status updates include comments?**
   - Option A: Yes, comment on each agent transition (e.g., "🤖 Rex is now working...")
   - Option B: No, just update status field silently
   - Option C: Configurable per-project
   - **Consideration:** Comments provide audit trail but may be noisy

5. **Should we create GitHub issues in addition to Linear issues?**
   - Option A: Yes, always mirror to GitHub for native PR linking
   - Option B: Yes, but only if repository has GitHub Issues enabled
   - Option C: No, rely on Linear attachments for PR linking
   - **Consideration:** GitHub issues enable `Closes #X` in PRs for automatic closure

6. **Which system is source of truth for task status?**
   - Option A: Linear primary → GitHub mirror (recommended)
   - Option B: GitHub primary → Linear view
   - Option C: Bidirectional sync (complex)
   - **Consideration:** Agent interaction happens in Linear, so Linear-primary makes sense

7. **How should agent discover linked documents?**
   - Option A: Issue attachments only (explicit linking)
   - Option B: Issue attachments + description link parsing
   - Option C: All documents in the same project (auto-discovery)
   - Option D: Custom field with document IDs
   - **Consideration:** Explicit linking (A or B) gives user control; auto-discovery (C) may pull in irrelevant docs

8. **Should agent create documents during intake?**
   - Option A: Yes, create Architecture/API docs from generated content
   - Option B: No, only create task issues; docs are user-maintained
   - Option C: Optional, controlled by parameter
   - **Consideration:** Generated docs could help, but may duplicate PRD content

### Technical Questions

1. **Where does the webhook receiver run?**
   - Option A: Dedicated K8s service in `cto` namespace (new service)
   - Option B: Cloudflare Worker → calls internal API (edge processing)
   - Option C: Add route to existing controller via Cloudflare Tunnel
   - **Consideration:** Need public HTTPS endpoint; Linear must reach it

2. **What's the public URL for the webhook?**
   - Option A: `https://cto.5dlabs.com/webhooks/linear` (via CF Tunnel)
   - Option B: `https://linear-webhook.5dlabs.workers.dev` (CF Worker)
   - Option C: `https://api.5dlabs.com/cto/webhooks/linear` (shared API gateway)

3. **How do we handle long-running workflows?**
   - Linear expects activity within 30 minutes
   - Play workflows can take hours
   - Need periodic heartbeat activities

4. **State management for sessions?**
   - Option A: Store session state in ConfigMap (current pattern)
   - Option B: Linear is source of truth (query as needed)
   - Option C: Redis/database for faster access

5. **How do we test the integration?**
   - Need Linear sandbox workspace
   - Mock webhook payloads for unit tests
   - E2E tests with real Linear instance

---

## Appendix: Linear API Reference

### OAuth Scopes Required

```
app:assignable    # Allow delegation to agent
app:mentionable   # Allow @mentions of agent
read              # Read issues, projects, etc.
write             # Create/update issues
```

### Key GraphQL Operations

#### Create Issue
```graphql
mutation IssueCreate($input: IssueCreateInput!) {
  issueCreate(input: $input) {
    success
    issue { id identifier url }
  }
}
```

#### Update Issue Status
```graphql
mutation IssueUpdate($id: String!, $input: IssueUpdateInput!) {
  issueUpdate(id: $id, input: $input) {
    success
  }
}
```

#### Create Agent Activity
```graphql
mutation AgentActivityCreate($input: AgentActivityCreateInput!) {
  agentActivityCreate(input: $input) {
    success
    agentActivity { id }
  }
}
```

#### Query Team Workflow States
```graphql
query TeamStates($teamId: String!) {
  team(id: $teamId) {
    states {
      nodes { id name type position }
    }
  }
}
```

### Webhook Payload Examples

#### AgentSessionEvent.created
```json
{
  "action": "created",
  "type": "AgentSessionEvent",
  "createdAt": "2025-12-06T10:00:00.000Z",
  "organizationId": "org-123",
  "agentSession": {
    "id": "session-456",
    "issue": {
      "id": "issue-789",
      "identifier": "TSK-1",
      "title": "Set up authentication",
      "description": "## PRD content...",
      "state": { "name": "Todo", "type": "unstarted" },
      "team": { "id": "team-abc", "key": "TSK" }
    },
    "comment": null,
    "previousComments": [],
    "guidance": "Repository: 5dlabs/teamsync"
  },
  "webhookTimestamp": 1733482800000
}
```

#### AgentSessionEvent.prompted
```json
{
  "action": "prompted",
  "type": "AgentSessionEvent",
  "agentSession": { "id": "session-456" },
  "agentActivity": {
    "id": "activity-xyz",
    "content": {
      "__typename": "AgentActivityPromptContent",
      "body": "Use the existing AuthService class"
    },
    "signals": []
  }
}
```

---

## References

- [Linear Developers: Getting Started with Agents](https://linear.app/developers/agents)
- [Linear Developers: Agent Interaction](https://linear.app/developers/agent-interaction)
- [Linear Developers: Webhooks](https://linear.app/developers/webhooks)
- [Linear Developers: Signals](https://linear.app/developers/agent-signals)
- [Linear GraphQL Schema Explorer](https://studio.apollographql.com/public/Linear-API/variant/current/home)
- [Linear Webhook Schema Explorer](https://studio.apollographql.com/public/Linear-Webhooks/variant/current/schema/reference/objects)

