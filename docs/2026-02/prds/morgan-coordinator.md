# Project: Morgan - Intelligent Project Coordinator

## Vision

Morgan is the intelligent coordination layer for CTO — a Clawdbot agent that maintains real-time awareness of all agent activity, provides unified status to humans, and ensures work stays aligned with objectives. Think of Morgan as an actual PM: high-level awareness of all projects, statuses, and blockers without getting into implementation details.

Morgan is the **single point of contact** for humans wanting to know "what's happening?" across all CTO agent activity.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         MORGAN                                       │
│              (Clawdbot Agent + ACP Server)                          │
├─────────────────────────────────────────────────────────────────────┤
│  Core Capabilities                                                   │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐        │
│  │ Status         │  │ Scope          │  │ Alert          │        │
│  │ Aggregation    │  │ Guardian       │  │ Engine         │        │
│  └───────┬────────┘  └───────┬────────┘  └───────┬────────┘        │
│          │                   │                   │                  │
├──────────┴───────────────────┴───────────────────┴──────────────────┤
│  State & Communication                                               │
│  ┌─────────┐ ┌─────────────┐ ┌─────────────┐                       │
│  │  State  │ │ ACP/A2A     │ │ Notify      │                       │
│  │  Store  │ │ Protocol    │ │ Channels    │                       │
│  │ (Redis) │ │ Handler     │ │ (Discord,   │                       │
│  │         │ │             │ │  Slack,etc) │                       │
│  └─────────┘ └─────────────┘ └─────────────┘                       │
└─────────────────────────────────────────────────────────────────────┘
          ▲                                       │
          │ ACP (tasks/send, status updates)      │ Notifications
          │                                       ▼
    ┌─────┴─────┬───────────┬───────────┐   ┌─────────────┐
    │           │           │           │   │   Human     │
  Stitch      Rex        Intake      Pixel  │   Users     │
  (review)   (code)      (PRD)       (app)  │  (Discord,  │
                                            │   Slack,    │
                                            │   Desktop)  │
                                            └─────────────┘
```

## Protocol

Morgan implements the **Agent Communication Protocol (ACP)** for agent-to-agent communication, complementing our existing MCP usage for agent-to-tool communication.

**Reference**: https://agentcommunicationprotocol.dev/

### Key Protocol Features Used

- **RESTful endpoints** for status updates from agents
- **Task lifecycle** (submitted → working → completed)
- **Agent discovery** via metadata
- **Async-first** with sync support
- **SSE/Webhooks** for real-time streaming to human interfaces

---

## Features

### 1. Status Aggregation Service

**Priority**: High  
**Complexity**: Medium

The core capability that receives status updates from all agents and maintains a unified view.

**Responsibilities**:
- Receive ACP `tasks/send` messages from agents
- Maintain real-time state of all active work
- Provide query interface for "what's happening?" requests
- Track task lifecycle (submitted → working → completed)

**Data Models**:
```
AgentStatus {
  agent_id: string        // "stitch", "rex", "intake", etc.
  current_task: Task | null
  status: "idle" | "working" | "blocked" | "waiting"
  last_update: timestamp
  context: string         // Brief description of current work
}

ProjectStatus {
  project_id: string      // Linear project ID or PRD name
  active_agents: AgentStatus[]
  progress: number        // 0-100%
  blockers: Blocker[]
  last_activity: timestamp
}
```

**Acceptance Criteria**:
- [ ] Receives status updates from agents via ACP endpoint
- [ ] Maintains in-memory + Redis state of all agent activity
- [ ] Responds to "status" queries with aggregated view
- [ ] Handles agent timeout/disconnect gracefully

---

### 2. Scope Guardian

**Priority**: High  
**Complexity**: Medium

Monitors work against original objectives and flags drift.

**Responsibilities**:
- Know original objectives (from PRDs, Linear issues)
- Compare current work against stated goals
- Detect scope creep or drift
- Alert when work diverges significantly

**Logic**:
```
on_status_update(agent, task):
  project = get_project(task.project_id)
  original_objectives = project.prd.objectives
  
  if not aligns_with_objectives(task, original_objectives):
    alert("Scope drift detected", {
      agent: agent,
      task: task,
      original: original_objectives
    })
```

**Acceptance Criteria**:
- [ ] Loads and understands PRD objectives
- [ ] Evaluates task alignment with objectives
- [ ] Generates drift alerts with context
- [ ] Allows human override ("this is intentional")

---

### 3. Alert Engine

**Priority**: Medium  
**Complexity**: Low

Proactive notification system for issues requiring human attention.

**Alert Types**:
- **Blocked**: Agent stuck for > N minutes
- **Conflict**: PR conflicts detected
- **Drift**: Work deviating from objectives
- **Failure**: CI failures, errors
- **Milestone**: Significant progress (optional)

**Routing**:
- All alerts → unified Morgan channel
- Critical alerts → direct mention/DM
- Summary digests → scheduled (hourly/daily)

**Acceptance Criteria**:
- [ ] Detects blocked agents via timeout
- [ ] Monitors GitHub for PR conflicts
- [ ] Routes alerts to configured channels
- [ ] Supports alert suppression/snooze

---

### 4. Human Interface

**Priority**: High  
**Complexity**: Medium

Single point of contact for human queries about system status.

**Capabilities**:
- Answer "what's happening?" with coherent summary
- Provide project-specific status on request
- Escalate blockers proactively
- Post updates to Discord/Slack/Teams/Desktop

**Query Examples**:
- "Morgan, what's the status of the OAuth PR?"
- "Morgan, what is Rex working on?"
- "Morgan, any blockers I should know about?"

**Integration Points**:
- Discord (via Clawdbot)
- Slack (via Clawdbot)
- Desktop app (via CTO)
- Linear comments (status updates)

**Acceptance Criteria**:
- [ ] Responds to natural language status queries
- [ ] Posts proactive updates on significant events
- [ ] Single unified presence (not per-agent channels)
- [ ] Configurable update frequency

---

### 5. Integration Coordinator

**Priority**: Medium  
**Complexity**: Medium

Ensures PRs and work integrate in correct order.

**Responsibilities**:
- Track PR dependencies
- Warn about merge order issues
- Detect conflicting changes early
- Coordinate multi-agent contributions to same PR

**Acceptance Criteria**:
- [ ] Tracks open PRs and their dependencies
- [ ] Alerts when merge order would cause issues
- [ ] Detects file-level conflicts before merge
- [ ] Coordinates when multiple agents touch same area

---

## Technical Requirements

### Infrastructure
- Redis for state persistence
- Clawdbot for agent runtime
- ACP endpoint (HTTP server)

### Integration Points
- GitHub API (PR status, conflicts)
- Linear API (issue status, objectives)
- Clawdbot messaging (Discord, Slack)

### Configuration
```json
{
  "morgan": {
    "acp_port": 8090,
    "redis_url": "redis://localhost:6379",
    "alert_channels": {
      "discord": "channel-id",
      "slack": "#cto-alerts"
    },
    "thresholds": {
      "blocked_timeout_minutes": 30,
      "drift_sensitivity": 0.7
    }
  }
}
```

---

## Non-Goals (Explicit)

Morgan does NOT:
- Write code
- Review PRs (that's Stitch)
- Make architecture decisions
- Implement features
- Debug technical issues

Morgan is the PM layer — knows what's happening, ensures alignment, alerts on issues, but delegates all "how" to specialized agents.

---

## Success Metrics

- Human can get accurate status within 1 message
- Scope drift detected before significant wasted effort
- Blockers surfaced proactively (not discovered by human)
- Single unified status view vs. checking multiple channels

---

## Open Questions

1. **State persistence**: Redis sufficient or need PostgreSQL for history?
2. **Agent adoption**: How do we update existing agents to report to Morgan?
3. **ACP vs custom**: Full ACP compliance or simplified subset?
4. **Objective understanding**: How does Morgan "understand" PRD objectives? LLM analysis on each update?
