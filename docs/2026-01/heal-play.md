# Healer Play Workflow Monitoring

> Comprehensive monitoring strategy for Healer to provide a bird's-eye view of the CTO Play lifecycle, detect issues, and orchestrate remediations.

## Overview

Healer acts as the **observability and self-healing layer** for the entire CTO Play workflow. Rather than just reacting to individual alerts, Healer should maintain awareness of the complete lifecycle from PRD intake through merged code, proactively identifying issues before they cascade.

### Design Philosophy

1. **Lifecycle Awareness**: Healer understands each stage's expected behaviors and transitions
2. **Proactive Detection**: Identify anomalies before they become failures
3. **Intelligent Remediation**: Spawn targeted fixes with appropriate context
4. **Memory Integration**: Learn from past issues to prevent recurrence

---

## 📊 Implementation Status Summary

| Phase | Status | Key Files |
|-------|--------|-----------|
| **-1: Play Start Notification** | ✅ Complete | `api.rs`, `session.rs`, MCP `notify_healer()` |
| **0: Agent Logging** | ✅ Complete | `tool_inventory.rs`, integrated in `controller.rs` |
| **1: Foundation** | ❌ Not started | Lifecycle state machine, ConfigMap detection |
| **2: Detection** | ✅ Complete | Scanner patterns A10-A12 in `scanner.rs` |
| **3: Dual-Model Architecture** | ✅ Complete | `evaluation_spawner.rs`, `remediation_spawner.rs`, `orchestrator.rs` |
| **4: Remediation Strategies** | ✅ Complete | Strategies + escalation via `escalate.rs` |
| **5: Intelligence** | ❌ Not started | OpenMemory integration, predictive detection |

### What Works Now

1. **MCP → Healer notification**: When you call `play()`, Healer is immediately notified
2. **Session tracking**: Healer stores CTO config, tasks, and expected tools
3. **Evaluation/Remediation spawners**: Can create Claude CLI CodeRuns
4. **Language matching**: Verifies Cleo/Cipher/Tess use correct language tools
5. **Integration tests**: 15+ tests validate the full flow
6. **Tool inventory logging**: Controller logs declared vs available tools at CodeRun startup
7. **Scanner patterns A10-A12**: Detects tool mismatch, config issues, MCP init failures
8. **Loki integration**: Orchestrator queries actual logs from Loki
9. **Human escalation**: Discord notifications and GitHub issues via Escalator

### What Needs Work

1. **Lifecycle state machine**: Stage transitions not yet tracked
2. **Linear integration**: Task details not pulled from Linear API
3. **OpenMemory integration**: Pattern learning not yet implemented
4. **Feedback loop automation**: Re-evaluation after remediation not connected

---

## 🔴 Universal Pre-Flight Checks (EVERY Agent Run)

**These checks apply to EVERY agent, EVERY CLI, EVERY task - no exceptions.**

This has been a consistent pain point. Before ANY agent can do useful work, Healer must verify:

### 1. Prompt Verification

Does the agent have the correct prompts for its:
- **Role** (Rex=Rust, Blaze=Frontend, Cipher=Security, etc.)
- **Task** (implementation, review, testing, integration)
- **Language/Stack** (Rust, TypeScript, Go, etc.)

```yaml
Healer must verify:
  - Agent type matches task requirements
  - Prompt template loaded successfully
  - Role-specific instructions present
  - Task context injected correctly
  - Language/stack hints included
```

**Log patterns to detect prompt issues:**
```
❌ template not found
❌ prompt.*missing
❌ failed to load.*template
❌ role.*undefined
❌ task context.*empty
```

### 2. MCP Tool Verification (from CTO Config)

Does the agent have access to the tools defined in `cto-config.json`?

```yaml
Healer must verify for each agent:
  Remote Tools (from tools-server):
    - All declared tools are accessible
    - tools-server is reachable
    - Authentication is valid
    
  Local Servers (filesystem, git, etc.):
    - MCP servers initialized
    - Permissions correct
    - Paths accessible
```

**The CTO Config contract:**
```json
{
  "agents": {
    "rex": {
      "tools": {
        "remote": ["memory_create_entities", "github_*", ...],
        "localServers": {
          "filesystem": { "enabled": true, "tools": ["read_file", "write_file", ...] },
          "git": { "enabled": true, "tools": ["git_status", "git_commit", ...] }
        }
      }
    }
  }
}
```

**Healer verifies:** `declared tools in CTO config` == `available tools in CLI`

**Log patterns to detect tool issues:**
```
❌ tool.*not found
❌ mcp.*failed to initialize
❌ tools-server.*unreachable
❌ authentication.*failed
❌ permission denied
❌ Tool inventory MISMATCH
❌ [tool_name] not available
```

### Why This Is Universal

| Agent | Needs Prompts? | Needs MCP Tools? |
|-------|---------------|------------------|
| Morgan (Intake) | ✅ Yes | ✅ Yes (Linear, memory) |
| Bolt (Infra) | ✅ Yes | ✅ Yes (kubectl, helm) |
| Rex (Rust) | ✅ Yes | ✅ Yes (filesystem, git, cargo) |
| Blaze (Frontend) | ✅ Yes | ✅ Yes (filesystem, git, npm) |
| Cleo (Quality) | ✅ Yes | ✅ Yes (git, github) |
| Cipher (Security) | ✅ Yes | ✅ Yes (filesystem, security scanners) |
| Tess (Testing) | ✅ Yes | ✅ Yes (filesystem, git, test runners) |
| Atlas (Integration) | ✅ Yes | ✅ Yes (git, github) |

**Every single agent needs both. No exceptions.**

### Evaluation Agent Must Check (for every CodeRun)

Within the first 60 seconds of any agent starting:

```markdown
## Pre-Flight Checklist

### Prompts
- [ ] Agent type identified: {agent_type}
- [ ] Role matches task: {role} → {task_type}
- [ ] Template loaded: {template_name}
- [ ] Language context: {language/stack}

### MCP Tools (from CTO Config)
- [ ] CTO config loaded: {config_path}
- [ ] Remote tools declared: {count}
- [ ] Remote tools accessible: {count}/{total}
- [ ] Local servers enabled: {list}
- [ ] Local servers initialized: {count}/{total}

### Verdict
- [ ] ✅ PASS: Agent ready to work
- [ ] ❌ FAIL: {specific_failure_reason}
```

### If Pre-Flight Fails

1. **Evaluation Agent** creates issue immediately
2. **Do NOT wait** for the agent to fail downstream
3. Issue includes:
   - Which checks failed
   - Expected vs actual tools
   - CTO config excerpt
   - Suggested fix (usually platform/config issue)

---

## Healer's Data Sources & Architecture

**Healer is an OBSERVER, not an actor.** It replaces the human watching agent runs.

### How Healer Gets Data

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         HEALER ARCHITECTURE                              │
│                                                                          │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐            │
│  │    LOKI      │     │  KUBERNETES  │     │   GITHUB     │            │
│  │  (Pod Logs)  │     │    (API)     │     │    (API)     │            │
│  └──────┬───────┘     └──────┬───────┘     └──────┬───────┘            │
│         │                    │                    │                     │
│         ▼                    ▼                    ▼                     │
│  ┌─────────────────────────────────────────────────────────────┐       │
│  │                        HEALER                                │       │
│  │  • Watches logs via Loki (LogQL queries)                    │       │
│  │  • Watches pods/events via Kubernetes API                    │       │
│  │  • Watches PR state via GitHub API                          │       │
│  │  • Detects patterns → spawns CodeRun remediations           │       │
│  └─────────────────────────────────────────────────────────────┘       │
│                              │                                          │
│                              ▼                                          │
│                      ┌──────────────┐                                   │
│                      │   CodeRun    │ (Creates PRs for fixes)          │
│                      └──────────────┘                                   │
└─────────────────────────────────────────────────────────────────────────┘
```

### ✅ IMPLEMENTED: Play Start Notification

**Status:** MCP server now notifies Healer immediately when a Play starts.

```
┌─────────────────┐      HTTP POST       ┌─────────────────┐
│  MCP play()     │  ─────────────────▶  │  HEALER API     │
│  (User submits) │   /api/v1/session/   │   (:8083)       │
└─────────────────┘        start         └─────────────────┘
```

**Implemented flow:**
1. User calls `play()` MCP tool
2. MCP server submits Argo workflow
3. **MCP server POSTs to Healer** with: play_id, repository, CTO config, tasks
4. Healer immediately stores session with expected tools per agent
5. CodeRuns start with Healer already aware

#### Configuration

In `cto-config.json`:
```json
{
  "defaults": {
    "play": {
      "healerEndpoint": "http://localhost:8083"
    }
  }
}
```

#### Healer API Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/health` | GET | Health check |
| `/api/v1/session/start` | POST | MCP calls this on play start |
| `/api/v1/session/{play_id}` | GET | Get session details |
| `/api/v1/sessions` | GET | List all sessions |
| `/api/v1/sessions/active` | GET | List active sessions only |

---

## 🧠 Target Architecture: Dual-Model System

The current Healer uses **programmatic detection** (regex, pattern matching) which is brittle and can't understand context. The target architecture uses **two specialized models** in a feedback loop:

### Overview

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                        DUAL-MODEL HEALER ARCHITECTURE                            │
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │                      DATA SOURCES                                        │    │
│  │  • Loki (all pod logs)                                                  │    │
│  │  • Kubernetes (CodeRuns, Pods, Events)                                   │    │
│  │  • GitHub (PRs, comments, CI status)                                     │    │
│  │  • Linear (task status, PRD, acceptance criteria)                        │    │
│  │  • CTO Config (expected tools, agent settings)                           │    │
│  └────────────────────────────────┬────────────────────────────────────────┘    │
│                                   │                                              │
│                                   ▼                                              │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │                    MODEL 1: EVALUATION AGENT                             │    │
│  │                                                                          │    │
│  │  Purpose: Intelligent observer that understands full context             │    │
│  │                                                                          │    │
│  │  Capabilities:                                                           │    │
│  │  • Parses and comprehends ALL logs (not just regex)                     │    │
│  │  • Knows when Play started and expected lifecycle                        │    │
│  │  • Understands CTO config and what tools should be available             │    │
│  │  • Correlates events across agents, pods, and GitHub                     │    │
│  │  • Identifies root cause, not just symptoms                              │    │
│  │                                                                          │    │
│  │  Output: Detailed GitHub Issue with:                                     │    │
│  │  • Root cause analysis                                                   │    │
│  │  • Relevant log excerpts                                                 │    │
│  │  • Expected vs actual behavior                                           │    │
│  │  • Suggested remediation approach                                        │    │
│  │  • Files/code likely involved                                            │    │
│  └────────────────────────────────┬────────────────────────────────────────┘    │
│                                   │                                              │
│                                   │ Creates detailed issue                       │
│                                   ▼                                              │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │                    MODEL 2: REMEDIATION AGENT                            │    │
│  │                                                                          │    │
│  │  Purpose: Fix the problem described in the issue                         │    │
│  │                                                                          │    │
│  │  Receives:                                                               │    │
│  │  • Detailed issue from Evaluation Agent                                  │    │
│  │  • Repository access                                                     │    │
│  │  • Context about what was tried before                                   │    │
│  │                                                                          │    │
│  │  Actions:                                                                │    │
│  │  • Makes targeted code/config changes                                    │    │
│  │  • Creates PR with fix                                                   │    │
│  │  • Reports outcome back to issue                                         │    │
│  └────────────────────────────────┬────────────────────────────────────────┘    │
│                                   │                                              │
│                                   │ Reports success/failure                      │
│                                   ▼                                              │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │                         FEEDBACK LOOP                                    │    │
│  │                                                                          │    │
│  │  If remediation failed:                                                  │    │
│  │  1. Evaluation Agent reviews remediation attempt                         │    │
│  │  2. Updates issue with new analysis                                      │    │
│  │  3. Remediation Agent tries again with refined context                   │    │
│  │  4. Repeat until fixed or escalate to human                              │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Model 1: Evaluation Agent

**Role:** The intelligent observer that replaces regex-based detection.

**Prompt Context Must Include:**
- Play ID and when it started
- CTO config (expected agents, tools, models)
- All lifecycle stage expectations
- Complete logs from Loki (structured, not raw)
- Kubernetes events and pod states
- GitHub PR/comment state
- Linear task details and acceptance criteria

**What It Looks For (in priority order):**

```yaml
# ═══════════════════════════════════════════════════════════════════════
# PRIORITY 1: Universal Pre-Flight Checks (EVERY agent, EVERY run)
# ═══════════════════════════════════════════════════════════════════════
# These MUST pass before anything else matters.
# Check within first 60 seconds of agent startup.

Prompts (Universal):
  - Does agent have correct prompts for its role?
  - Does prompt match task type (impl/review/test/integrate)?
  - Is language/stack context included?
  - Did template load successfully?

MCP Tools (Universal - from CTO Config):
  - Is CTO config loaded and valid?
  - Are ALL declared remote tools accessible?
  - Is tools-server reachable and authenticated?
  - Are ALL declared local servers initialized?
  - Do declared tools == available tools? (MUST MATCH)

# If ANY pre-flight check fails → IMMEDIATE issue creation
# Do NOT wait for downstream failures

# ═══════════════════════════════════════════════════════════════════════
# PRIORITY 2: Lifecycle Progression (after pre-flight passes)
# ═══════════════════════════════════════════════════════════════════════

Lifecycle Progression:
  - Is the agent in the expected stage?
  - Has it been stuck too long?
  - Are stage transitions happening correctly?

Agent Behavior:
  - Is the agent making progress?
  - Are there error patterns in logs?
  - Is it following the expected workflow?

Infrastructure:
  - Are all platform services healthy?
  - Are there resource constraints?
  - Is the agent hitting rate limits?
```

**Output Format:** GitHub Issue with structured sections:

```markdown
## 🔍 Healer Evaluation Report

### Summary
[One-line description of the problem]

### Play Context
- **Play ID:** play-task-42
- **Stage:** Implementation (Rex)
- **Duration:** 15 minutes
- **Expected:** Should have created PR by now

### Root Cause Analysis
[Detailed explanation of what went wrong and why]

### Evidence
#### Relevant Logs
\`\`\`
[Actual log excerpts that show the problem]
\`\`\`

#### Timeline
- 14:00:00 - Play started
- 14:02:15 - Rex CodeRun created
- 14:05:30 - Tool initialization failed ← **ISSUE**
- 14:05:31 - Agent continued without tools

### Suggested Remediation
1. [Specific action to fix]
2. [Verification step]

### Files Likely Involved
- `crates/controller/src/tools.rs`
- `config/cto-config.json`

### Prior Attempts
- None (first detection)
```

### Model 2: Remediation Agent

**Role:** Takes the detailed issue and fixes the problem.

**Receives:**
- Complete GitHub issue from Evaluation Agent
- Repository checkout at the relevant commit
- History of prior remediation attempts (if any)
- Access to make code changes and create PRs

**Actions:**
1. Reads the issue thoroughly
2. Investigates the codebase based on hints
3. Makes targeted fixes
4. Creates PR with clear description
5. Comments on issue with outcome

**Reports Back:**

```markdown
## Remediation Attempt #1

### Changes Made
- Modified `crates/controller/src/tools.rs` to handle missing config
- Added fallback for tool initialization

### PR Created
#1234 - "fix: handle missing CTO config gracefully"

### Verification
- [ ] CI passing
- [ ] Manual testing needed

### Status: PENDING_CI
```

### Feedback Loop

If the remediation fails (CI fails, problem persists, etc.):

1. **Evaluation Agent** reviews:
   - The PR that was created
   - New logs after the fix attempt
   - CI failure output
   - Why the fix didn't work

2. **Updates the issue:**

   ```markdown
   ## Re-evaluation After Attempt #1
   
   ### What Happened
   The fix addressed symptom but not root cause...
   
   ### New Analysis
   [Refined understanding]
   
   ### Revised Remediation Approach
   [Different strategy]
   ```

3. **Remediation Agent** tries again with refined context

4. **Escalation:** After N attempts (configurable), escalate to human with full history

### Why Two Models?

| Aspect | Single Model | Dual Model |
|--------|-------------|------------|
| **Context** | Must fit detection + fix in one context | Each model has focused context |
| **Specialization** | Jack of all trades | Expert at one job |
| **Debugging** | Hard to know where it went wrong | Clear: detection vs remediation |
| **Iteration** | Restarts from scratch | Builds on prior analysis |
| **Cost** | One large call | Two smaller, targeted calls |

### Model Selection

| Role | Recommended Model | Reasoning |
|------|------------------|-----------|
| **Evaluation Agent** | claude-sonnet-4-5 or opus | Needs deep reasoning, long context |
| **Remediation Agent** | claude-sonnet-4-5 | Needs coding ability, can be faster |

### Implementation Requirements

#### For Evaluation Agent
- [ ] Access to all data sources (Loki, K8s, GitHub, Linear)
- [ ] Play start notification (Phase -1 prerequisite)
- [ ] CTO config for the specific play
- [ ] Long context window (to process full logs)
- [ ] Structured output format (for parsing)

#### For Remediation Agent
- [ ] Repository access (checkout, branch, commit)
- [ ] GitHub App for PR creation
- [ ] Issue commenting capability
- [ ] Access to prior attempt history

#### For Feedback Loop
- [ ] State machine tracking attempt count
- [ ] Issue threading (all attempts in one issue)
- [ ] Escalation rules and thresholds
- [ ] Human notification mechanism

---

### What Healer Does (Current State)
- **Observes** log streams from Loki
- **Detects** error patterns, anomalies, stuck states
- **Spawns** CodeRun remediations (which make PRs)
- **Learns** from OpenMemory to prevent recurrence

### What Healer Does NOT Do
- Parse CTO config files directly
- Run MCP tools itself
- Interact with Linear (that's PM Server's job)
- Execute code in agent pods

---

## 🚨 Priority: MCP Tool Accessibility (Log-Based Detection)

**The CLI/agents must LOG their tool status so Healer can observe.**

### The Contract: What Agents Should Log

For Healer to verify tool accessibility, **our Rust code must log**:

#### On Agent Startup (within first 60 seconds)
```
✅ INFO: CTO config loaded: cto-config.json
✅ INFO: Agent 'rex' tools configured: remote=[memory_create_entities, memory_add_observations], local=[filesystem, git]
✅ INFO: Connected to tools-server at http://cto-tools:8080
✅ INFO: MCP tools initialized: 15 remote tools available
✅ INFO: Filesystem MCP server ready
✅ INFO: Git MCP server ready
```

#### On Tool Failure
```
❌ ERROR: Tool 'brave_search' not found in tools-server
❌ ERROR: MCP server 'filesystem' failed to initialize: permission denied
❌ ERROR: Remote tool 'memory_create_entities' unreachable: connection refused
❌ ERROR: Tool authentication failed: invalid token for 'github_*'
❌ ERROR: CTO config missing or invalid: parse error at line 42
```

### What Healer Watches For (Log Patterns)

| Pattern | Severity | Meaning |
|---------|----------|---------|
| `tool.*not found` | Critical | Tool declared but missing |
| `mcp.*error\|failed to initialize mcp` | Critical | MCP system broken |
| `routing failed\|server not connected` | Critical | tools-server unreachable |
| `permission denied.*filesystem` | Critical | Can't read/write files |
| `git command failed` | High | Can't commit code |
| `unauthorized\|invalid token` | Critical | Auth broken |
| `cto-config.*missing\|invalid` | Critical | Config not synced |

### 🔴 Critical Gap: Config ↔ CLI Reconciliation

**This is a known pain point.** The CTO config (from Linear) defines what tools an agent *should* have, but we don't know if the CLI actually has them.

```
┌─────────────────────┐         ┌─────────────────────┐
│   CTO Config        │   ???   │   CLI Runtime       │
│   (Linear JSON)     │ ─────── │   (Actual Tools)    │
├─────────────────────┤         ├─────────────────────┤
│ rex.tools.remote:   │         │ What Claude Code    │
│ - memory_create     │         │ actually sees:      │
│ - memory_add        │         │ - ???               │
│ - brave_search      │         │ - ???               │
└─────────────────────┘         └─────────────────────┘
```

**Failure modes we've seen:**
- Config says agent has `brave_search` but tools-server doesn't route it
- Config defines `filesystem` server but CLI doesn't have MCP configured
- Tool name in config doesn't match actual MCP tool name (typo, versioning)

#### Solution: Agents Must Log Tool Inventory

**Requirement for CLI/Controller:**

On startup, the agent should log:
```
INFO: Tool inventory check - CTO config declares: [memory_create_entities, memory_add_observations, brave_search]
INFO: Tool inventory check - CLI has available: [memory_create_entities, memory_add_observations]
ERROR: Tool inventory MISMATCH - missing from CLI: [brave_search]
```

**Healer then watches for:**
```
❌ Tool inventory MISMATCH
❌ missing from CLI
❌ declared.*not found
```

### Why This Matters

Without tool access, agents will:
- Fail to read/write files (can't implement anything)
- Fail to commit code (can't complete tasks)
- Fail to search memory (loses context)
- Spin uselessly consuming resources
- Eventually timeout (A8/A9 alerts) - but we should catch this EARLIER

**Tool verification should fail fast within 60 seconds, not after 30 minutes of wasted compute.**

---

## Complete Play Lifecycle

```
┌──────────────────────────────────────────────────────────────────────────────────────┐
│                              CTO PLAY LIFECYCLE                                       │
│                                                                                       │
│  ┌─────────┐    ┌─────────┐    ┌──────────────┐    ┌─────────┐    ┌─────────────┐   │
│  │  PRD    │───▶│  Intake │───▶│Infrastructure│───▶│  Impl   │───▶│   Quality   │   │
│  │ Submit  │    │ (Morgan)│    │    (Bolt)    │    │(Rex/etc)│    │   (Cleo)    │   │
│  └─────────┘    └─────────┘    └──────────────┘    └─────────┘    └─────────────┘   │
│                                                                          │           │
│  ┌─────────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐           │           │
│  │   Merged    │◀───│  Atlas  │◀───│   Tess  │◀───│ Cipher  │◀──────────┘           │
│  │             │    │         │    │         │    │         │                        │
│  └─────────────┘    └─────────┘    └─────────┘    └─────────┘                        │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: PRD & Intake

### What Happens
1. User submits PRD via `intake()` MCP tool
2. Linear project and PRD issue created
3. Morgan assigned as delegate
4. PM Server webhook triggers Argo intake workflow
5. Morgan parses PRD, generates tasks.json, creates task issues

### What Healer Should Monitor

| Condition | Detection Method | Severity | Remediation |
|-----------|------------------|----------|-------------|
| **Intake workflow not starting** | No Argo workflow within 2min of PRD issue creation | High | Alert user; check PM server health |
| **Morgan pod stuck pending** | Pod in Pending state >5min | High | Check node resources, PVC bindings |
| **PRD parsing failure** | Error logs from Morgan: `parse error`, `invalid prd` | High | Spawn Morgan with error context |
| **tasks.json not generated** | No tasks.json in .tasks/ after 30min | High | Check Morgan logs, retry intake |
| **Linear API failures** | `linear api error`, `rate limit` in logs | Medium | Exponential backoff, alert if persistent |
| **Task issues not created** | tasks.json exists but Linear issues missing | Medium | Spawn Morgan to create missing issues |

### Expected Morgan Behaviors
```
✅ Acknowledged PRD
✅ Parsing requirements
✅ Generated N tasks
✅ Created task issues
✅ Intake complete
```

### Failure Indicators
```
❌ Failed to parse PRD
❌ Invalid architecture document
❌ Linear API rate limited
❌ Could not create project
❌ Timeout during task generation
```

---

## Phase 2: Infrastructure (Bolt)

### What Happens
1. Tasks with infrastructure hints are assigned to Bolt (typically early in task order)
2. Bolt provisions databases, caches, secrets
3. Creates infra-config ConfigMap
4. Opens PR with infrastructure definitions

> **Note:** Agent assignment is based on task hints/metadata from intake, not task number. Infrastructure tasks are *typically* generated first, but this is not hardcoded.

### What Healer Should Monitor

| Condition | Detection Method | Severity | Remediation |
|-----------|------------------|----------|-------------|
| **Bolt not starting** | No Bolt pod within 5min of play start | Critical | Check controller, create CodeRun |
| **Database not provisioning** | `cluster not ready`, `postgres failed` | High | Check CNPG operator, node resources |
| **Secret mounting failures** | `secret not found`, `env not set` | Critical | Check OpenBao connectivity, ESO sync |
| **infra-config not created** | No ConfigMap after Bolt completion | High | Review Bolt logs, retry step |
| **Helm chart failures** | `helm error`, `chart failed` | High | Check chart values, dependencies |
| **PVC stuck pending** | PVC in Pending state >10min | High | Check storage class, node affinity |

### Expected Bolt Behaviors
```
✅ kubectl apply (created/configured)
✅ helm install (release deployed)
✅ Database cluster ready
✅ ConfigMap created
✅ Secrets synced
✅ PR created
```

### Failure Indicators
```
❌ kubectl error / error from server
❌ helm error / chart failed
❌ CRD not found
❌ Connection refused
❌ PVC pending
❌ Operator not ready
```

---

## Phase 3: Implementation (Rex/Blaze/Grizz/Nova/Tap/Spark)

### What Happens
1. Implementation agent selected based on task `agentHint`
2. Agent writes code, runs local builds
3. Commits changes, creates/updates PR
4. Hands off to quality review

### What Healer Should Monitor

| Condition | Detection Method | Severity | Remediation |
|-----------|------------------|----------|-------------|
| **No commits for 30min** | Stale progress (A3 alert) | High | Send input nudge, check for blocks |
| **Git push failures** | `failed to push`, `permission denied` | High | Check GitHub App credentials |
| **Merge conflicts** | `CONFLICT`, `merge conflict` | High | Spawn Atlas for resolution |
| **Build failures** | `cargo build failed`, `npm error`, `tsc error` | High | Spawn same agent with error context |
| **Clippy/ESLint errors** | Linter failure patterns in logs | Medium | Include in remediation prompt |
| **Agent crash/OOM** | Pod in Error/OOMKilled state | Critical | Increase resources, retry |
| **Silent failure (A2)** | Container exited, pod still "Running" | Critical | Force pod restart |

### Agent-Specific Monitoring

#### Rex (Rust)
```
Success: cargo build, cargo test, git push, PR created
Failure: error[E0xxx], cannot compile, clippy errors
Anomaly: force push, retry attempts
```

#### Blaze (Next.js/React)
```
Success: npm build, next build, compiled successfully
Failure: TS errors, eslint errors, build failed
Anomaly: deprecation warnings, peer dependency issues
```

#### Nova (Bun/Effect)
```
Success: bun build, bun test, effect schema valid
Failure: Effect.fail, FiberFailure, validation failed
Anomaly: Effect.Defect, deprecations
```

### Implementation Stage Timeout
- **Expected duration**: 15-45 minutes per task
- **Warning threshold**: 30 minutes (A3 alert)
- **Critical threshold**: 60 minutes (escalate)

---

## Phase 4: Quality Review (Cleo)

### What Happens
1. Cleo receives PR for review
2. Analyzes code for quality issues
3. Posts review: APPROVE or REQUEST_CHANGES
4. If changes requested, loops back to implementation

### 🔴 Language Matching (Healer Must Verify)

**Cleo must use the correct linting tools based on implementation agent:**

| Implementation Agent | Language | Cleo Must Run |
|---------------------|----------|---------------|
| **Rex** | Rust | `cargo clippy`, `cargo fmt --check`, rustfmt rules |
| **Grizz** | Go | `golangci-lint`, `go vet`, `gofmt` |
| **Nova** | TypeScript/Node | `eslint`, `prettier`, type checking |
| **Blaze** | React/TS | `eslint`, `prettier`, React best practices |
| **Tap** | Expo/RN | Mobile-specific linting |
| **Spark** | Electron | Electron + React linting |

**Healer checks:** Is Cleo running language-appropriate quality checks for this task?

### What Healer Should Monitor

| Condition | Detection Method | Severity | Remediation |
|-----------|------------------|----------|-------------|
| **Wrong language linting** | Rust task but no clippy, Go task but no golangci-lint | Critical | Cleo misconfigured |
| **Review not posted** | No GitHub review after 15min | High | Check Cleo logs, retry step |
| **API rate limiting** | `rate limit`, `too many requests` | Medium | Backoff and retry |
| **Infinite review loop** | >3 rounds of changes requested | High | Escalate to human |
| **Approval without review** | APPROVE with no review comments | Low | Log anomaly for analysis |
| **PR fetch failures** | `could not fetch pr` | High | Check GitHub connectivity |

### Expected Cleo Behaviors
```
✅ Review submitted
✅ APPROVED / Changes requested
✅ Comment posted
✅ Review complete
```

### Failure Indicators
```
❌ Failed to post review
❌ API rate limit
❌ Could not fetch PR
❌ Review not submitted
```

---

## Phase 5: Security Scan (Cipher)

### What Happens
1. Cipher runs security analysis
2. Checks for vulnerabilities, exposed secrets
3. Runs cargo audit / npm audit
4. Approves or blocks based on findings

### 🔴 Language Matching (Healer Must Verify)

**Cipher must use the correct security scanning tools based on implementation agent:**

| Implementation Agent | Language | Cipher Must Run |
|---------------------|----------|-----------------|
| **Rex** | Rust | `cargo audit`, `cargo deny`, RUSTSEC advisory check |
| **Grizz** | Go | `gosec`, `govulncheck`, Go module audit |
| **Nova** | TypeScript/Node | `npm audit`, `snyk`, dependency scanning |
| **Blaze** | React/TS | `npm audit`, OWASP checks, XSS scanning |
| **Tap** | Expo/RN | Mobile security scanning, npm audit |
| **Spark** | Electron | Electron security advisories, npm audit |

**Healer checks:** Is Cipher running language-appropriate security scans for this task?

### What Healer Should Monitor

| Condition | Detection Method | Severity | Remediation |
|-----------|------------------|----------|-------------|
| **Wrong language scanning** | Rust task but no cargo audit, Go task but no gosec | Critical | Cipher misconfigured |
| **Vulnerability found** | `vulnerability`, `security issue` | Critical | Block merge, alert human |
| **Secret exposure detected** | `secret exposed`, `credential leak` | Critical | Immediate block, notify security |
| **Audit failures** | `audit failed`, `insecure dependency` | High | Update dependencies, retry |
| **Cipher timeout** | No completion after 20min | Medium | Check scan resources |

### Expected Cipher Behaviors
```
✅ Security scan passed
✅ No vulnerabilities
✅ Secrets verified
✅ Audit passed
```

### Failure Indicators
```
❌ Vulnerability found (CRITICAL - must block)
❌ Secret exposed (CRITICAL - must block)
❌ Audit failed
❌ Insecure dependency
```

---

## Phase 6: Testing (Tess)

### What Happens
1. Tess runs test suite
2. Validates implementation meets requirements
3. May run integration tests
4. Approves if tests pass

### 🔴 Language Matching (Healer Must Verify)

**Tess must use the correct test runner based on implementation agent:**

| Implementation Agent | Language | Tess Must Run |
|---------------------|----------|---------------|
| **Rex** | Rust | `cargo test`, `cargo test --all-features`, integration tests |
| **Grizz** | Go | `go test ./...`, benchmark tests, race detection |
| **Nova** | TypeScript/Node | `npm test`, `jest`, `vitest`, integration tests |
| **Blaze** | React/TS | `npm test`, React Testing Library, E2E tests |
| **Tap** | Expo/RN | `expo test`, mobile-specific testing |
| **Spark** | Electron | `npm test`, Electron testing utilities |

**Healer checks:** Is Tess running language-appropriate test suites for this task?

### What Healer Should Monitor

| Condition | Detection Method | Severity | Remediation |
|-----------|------------------|----------|-------------|
| **Wrong language testing** | Rust task but no cargo test, Go task but no go test | Critical | Tess misconfigured |
| **Test failures** | `FAILED`, `assertion failed` | High | Spawn impl agent with test context |
| **Test panics** | `panicked at`, `thread panicked` | Critical | Include stack trace in remediation |
| **Approved despite failures (A5)** | Approval + failing tests detected | Critical | Block merge, investigate |
| **Flaky tests** | Same test fails intermittently | Medium | Flag for human review |
| **No tests run** | Test output missing | High | Verify test command ran |
| **CI failure after Tess approval** | GitHub Actions failure post-approval | High | A5 alert, investigate |

### Expected Tess Behaviors
```
✅ test result: ok
✅ N passed, 0 failed
✅ All tests passed
✅ Tests complete
```

### Failure Indicators
```
❌ test result: FAILED
❌ assertion failed
❌ panicked at
❌ N failed
```

### Critical Anti-Pattern
**Tess approving with failing tests is a BUG that must never reach production.**

---

## Phase 7: Integration (Atlas)

### What Happens
1. Atlas rebases branch on main/develop
2. Resolves any conflicts
3. Ensures CI passes
4. Merges PR

### What Healer Should Monitor

| Condition | Detection Method | Severity | Remediation |
|-----------|------------------|----------|-------------|
| **Merge conflicts** | `CONFLICT`, `cannot merge` | High | Spawn Atlas remediation |
| **Rebase failures** | `rebase failed`, `could not rebase` | High | May need human intervention |
| **CI failures post-rebase** | GitHub Actions red after rebase | High | Spawn impl agent to fix |
| **Branch diverged significantly** | >50 commits behind main | Medium | Alert, may need manual merge |
| **Merge blocked** | GitHub required checks failing | High | Investigate blocking checks |

### Expected Atlas Behaviors
```
✅ Rebase successful
✅ Merge successful
✅ Conflicts resolved
✅ Branch updated
✅ PR ready to merge
```

### Failure Indicators
```
❌ CONFLICT / merge conflict
❌ Rebase failed
❌ Cannot merge
❌ Diverged
```

---

## Cross-Cutting Concerns

### Platform Health Checks (via Log Observation)

**Healer detects these failures by watching logs, not direct health checks.**

| Component | Log Pattern to Watch | Failure Impact |
|-----------|---------------------|----------------|
| **Tools-Server** | `tools-server.*unreachable`, `mcp.*connection refused` | All agents fail (no tool access) |
| **CTO Config** | `cto-config.*missing`, `cto-config.*invalid` | Agents don't know required tools |
| **MCP Initialization** | `failed to initialize mcp`, `mcp.*error` | Agent can't use tools |
| PM Server | `webhook.*failed`, `pm-server.*error` | Intake won't start |
| Controller | `controller.*error`, `failed to create coderun` | No CodeRuns created |
| ArgoCD | `argocd.*sync failed`, `application.*degraded` | GitOps deployments fail |
| GitHub Apps | `github.*authentication failed`, `invalid signature` | All git operations fail |
| OpenBao | `secret not found`, `vault.*error` | Secrets unavailable |
| Linear API | `linear.*rate limit`, `linear.*error` | Status updates fail |

### Resource Exhaustion

| Resource | Warning Threshold | Critical Threshold | Remediation |
|----------|-------------------|-------------------|-------------|
| Node memory | 80% utilization | 95% utilization | Scale cluster |
| Node CPU | 80% utilization | 95% utilization | Scale cluster |
| PVCs | 80% capacity | 95% capacity | Expand storage |
| GitHub rate limit | 100 remaining | 10 remaining | Pause operations |

### Global Failure Patterns

These patterns should trigger alerts regardless of which agent is running:

```
❌ panic / panicked
❌ fatal: / fatal error
❌ segmentation fault
❌ out of memory / OOM / killed
❌ permission denied
❌ authentication failed
❌ invalid signature/token/key
❌ unauthorized / 401
❌ forbidden / 403
❌ connection refused
❌ timeout / timed out
```

---

## Existing Alert Mapping

Current Healer alerts and how they fit into lifecycle monitoring:

| Alert | Description | Lifecycle Phase |
|-------|-------------|-----------------|
| A1 | Comment order mismatch | Implementation, Quality |
| A2 | Silent agent failure | All phases |
| A3 | Stale progress (no commits) | Implementation |
| A4 | Repeated approval loop | Quality, Security, Testing |
| A5 | Post-Tess CI/merge failure | Testing, Integration |
| A7 | Pod failure | All phases |
| A8 | Workflow step timeout | All phases |
| A9 | Stuck CodeRun | All phases |

### Alerts Still Needed

| ID | Description | Priority | Phase |
|----|-------------|----------|-------|
| **A10** | **Play started without Healer notification** | **Critical** | **Play Start** |
| **A11** | **MCP tools not accessible (from CTO config)** | **Critical** | **All (Pre-flight)** |
| **A12** | **CTO config ↔ CLI tool mismatch (declared vs actual)** | **Critical** | **All (Pre-flight)** |
| A13 | CTO config missing or invalid | Critical | All (Pre-flight) |
| A14 | Intake workflow stuck | High | Intake |
| A15 | Infrastructure provisioning failure | High | Infrastructure |
| A16 | Security vulnerability blocking merge | Critical | Security |
| A17 | Test suite not executed | High | Testing |
| A18 | Repeated task failures (>3 attempts) | High | All |
| A19 | Linear sync failure | Medium | All |
| A20 | Memory/OpenMemory connectivity loss | Medium | All |

---

## Remediation Strategies

### Level 1: Automatic Retry
- Transient network errors
- Rate limit backoffs
- Pod restarts for OOM

### Level 2: Agent Re-spawn
- Build failures with error context
- Test failures with stack traces
- Merge conflicts with conflict markers

### Level 3: Alternative Agent
- If Rex fails repeatedly, try with Factory
- Cross-agent knowledge transfer

### Level 4: Human Escalation
- Security vulnerabilities
- Repeated failures (>3 attempts)
- Architectural decisions needed
- Infinite approval loops

---

## Implementation Checklist

### Phase -1: Play Start Notification ✅ COMPLETE

**Implemented via Option B: MCP Server HTTP notification to Healer API.**

#### MCP Server (`crates/notify/mcp/src/main.rs`)
- [x] `notify_healer()` function sends POST to Healer on play start
- [x] Includes: play_id, repository, service, cto_config (with agents + tools), tasks
- [x] Non-fatal: play workflow continues even if Healer unreachable
- [x] `healerEndpoint` config field in `PlayDefaults`

#### Healer Play API (`crates/healer/src/play/api.rs`)
- [x] HTTP server on port 8083
- [x] `POST /api/v1/session/start` - receives MCP notification
- [x] `GET /api/v1/session/{play_id}` - retrieve session details
- [x] `GET /api/v1/sessions` - list all sessions
- [x] `GET /api/v1/sessions/active` - list active sessions only
- [x] `GET /health` - health check

#### Session Storage (`crates/healer/src/play/session.rs`)
- [x] `PlaySession` stores: play_id, repository, CTO config, tasks, issues, status
- [x] `SessionStore` with async methods for session management
- [x] `CtoConfig` with agents and their tool definitions
- [x] `AgentTools` with `remote` and `localServers` (camelCase serde aliases)

#### Configuration
- [x] `cto-config.json`: `healerEndpoint: "http://localhost:8083"`
- [x] `infra/charts/cto/cto-config.json`: `healerEndpoint: "http://cto-healer-play-api:8083"`
- [x] `cto-config.template.json`: documented field
- [x] `crates/config/src/types.rs`: `healer_endpoint` in `PlayDefaults`

#### Infrastructure
- [x] `justfile`: `just dev-healer-play-api`, `just healer-play-api`
- [x] `mprocs.yaml`: `healer-play-api` process
- [x] Port 8083 added to `kill-ports` and `preflight`

#### Integration Tests (`crates/healer/tests/play_integration.rs`)
- [x] 15 tests covering full MCP → Healer flow
- [x] Tests for exact JSON format (camelCase fields)
- [x] Tests for session lifecycle, issue tracking, language matching

### Phase 0: Agent Logging Requirements ✅ COMPLETE

**Healer observes logs - agents must emit the right signals.**

#### Controller/CLI Tool Inventory (`crates/controller/src/tasks/tool_inventory.rs`) ✅ COMPLETE
- [x] `log_tool_inventory()` - logs declared vs resolved tools
- [x] `validate_expected_tools()` - finds missing tools
- [x] `format_inventory_diff()` - human-readable output
- [x] Structured logging with tracing (info/warn levels)
- [x] Integrated into CodeRun startup flow (`crates/controller/src/tasks/code/controller.rs`)

#### Controller Integration (`crates/controller/src/tasks/code/controller.rs`) ✅ COMPLETE
- [x] Extracts `remote_tools` from CodeRun spec at job creation
- [x] Calls `log_tool_inventory()` with agent name and declared tools
- [x] Emits warning log with specific pattern for Healer detection:
  - `"⚠️ tool inventory mismatch - declared tools: [...] - missing: [...]"`

#### Healer Detection Patterns (A10-A12) ✅ COMPLETE (`crates/healer/src/scanner.rs`)
- [x] Pattern: `tool\s+inventory\s+mismatch` (A10 - tool inventory mismatch)
- [x] Pattern: `declared\s+tools.*missing` (A10 - specific missing tools)
- [x] Pattern: `cto-config.*(missing|invalid)` (A11 - config issues)
- [x] Pattern: `mcp.*failed\s+to\s+initialize` (A12 - MCP init failure)
- [x] Pattern: `tools-server.*unreachable` (A12 - tools-server down)
- [x] Unit tests for all new patterns

### Phase 1: Foundation
- [ ] Implement lifecycle state machine in Healer
- [ ] Add stage transition detection from ConfigMaps
- [ ] Create unified log aggregation from all agents

### Phase 2: Detection
- [ ] Implement remaining alerts (A12-A18)
- [ ] Add platform health monitoring
- [ ] Create resource exhaustion alerting

### Phase 3: Dual-Model Architecture ✅ CORE IMPLEMENTED

**LLM-based evaluation and remediation via Claude CLI CodeRuns.**

#### Evaluation Agent (`crates/healer/src/play/evaluation_spawner.rs`) ✅ COMPLETE
- [x] `EvaluationSpawner` creates Claude CLI CodeRuns
- [x] `build_evaluation_prompt()` generates comprehensive prompt with:
  - Play context (ID, repository, service)
  - Full CTO config with expected tools per agent
  - Task list with dependencies
  - Universal Pre-Flight Checks section
  - Log analysis instructions
- [x] `build_coderun_spec()` creates K8s CodeRun YAML
- [x] Play context injection from session storage
- [ ] **TODO:** Actually query Loki logs (currently scaffolded)
- [ ] **TODO:** Linear integration for task details

**Universal Pre-Flight in Evaluation Prompt:** ✅ INCLUDED
- [x] Prompt verification checklist
- [x] MCP tool verification checklist
- [x] Declared vs available tools comparison
- [x] Fail-fast instructions (60s timeout)
- [x] Specific tool names in prompt context

#### Remediation Agent (`crates/healer/src/play/remediation_spawner.rs`) ✅ COMPLETE
- [x] `RemediationSpawner` creates remediation CodeRuns
- [x] `get_remediation_strategy()` selects strategy based on issue type
- [x] `build_remediation_prompt()` includes:
  - Issue details (type, severity, description)
  - Session context (repository, service)
  - Prior attempt count
  - Strategy-specific instructions
- [x] Strategies: `FixCode`, `FixConfig`, `Retry`, `Restart`, `Escalate`

#### Orchestrator (`crates/healer/src/play/orchestrator.rs`) ✅ COMPLETE
- [x] `HealerOrchestrator` manages feedback loop
- [x] `process_session()` evaluates and remediates
- [x] `verify_language_match()` checks Cleo/Cipher/Tess use correct tools
- [x] `OrchestratorConfig` with `max_remediation_attempts`, `auto_escalate`
- [x] `ImplementationLanguage` mapping (Rex→Rust, Grizz→Go, etc.)
- [x] `run_evaluation_and_process_logs()` queries Loki for actual logs
- [x] `logs_to_issues()` converts filtered log entries to `SessionIssue`s
- [x] `escalate_issue()` uses Escalator for notifications and GitHub issues

#### Loki Integration ✅ COMPLETE
- [x] `LokiClient` integrated into `HealerOrchestrator`
- [x] Queries logs by play ID label within session time window
- [x] Filters logs using `LogScanner` (same patterns as CI healer)
- [x] Converts detected errors to typed `SessionIssue`s:
  - `IssueType::ToolMismatch` for A10 patterns
  - `IssueType::CtoConfigIssue` for A11 patterns
  - `IssueType::McpInitFailure` for A12 patterns

#### Feedback Loop ✅ CORE COMPLETE
- [x] State machine structure in orchestrator
- [x] Attempt counting per issue
- [x] Escalation threshold configurable (default: 3)
- [x] Escalation via `Escalator` (Discord + GitHub issues)
- [x] Remediation strategy selection based on issue type
- [ ] Issue threading (all attempts in one issue) - future enhancement
- [ ] Re-evaluation trigger after remediation - future enhancement

### Phase 4: Remediation Strategies ✅ COMPLETE

#### Remediation Strategy Selection (`crates/healer/src/play/remediation_spawner.rs`)
- [x] `determine_remediation_strategy()` maps issue types to strategies
- [x] Strategy types: `FixCode`, `FixConfig`, `Retry`, `Restart`, `Escalate`
- [x] Issue-specific strategies:
  - `ToolMismatch` → `FixConfig` (fix CTO config or tools-server)
  - `CtoConfigIssue` → `FixConfig`
  - `McpInitFailure` → `Restart` (try restarting MCP servers)
  - `PreflightFailed` → `FixConfig`
  - `LanguageMismatch` → `FixConfig`

#### Human Escalation Workflow (`crates/healer/src/ci/escalate.rs`) ✅ INTEGRATED
- [x] `Escalator` struct with `EscalationConfig`
- [x] Discord notifications via webhook
- [x] GitHub issue creation via `gh` CLI
- [x] PR commenting for in-context alerts
- [x] Configurable channels per escalation level
- [x] Integrated into `HealerOrchestrator.escalate_issue()`

### Phase 5: Intelligence & Learning
- [ ] Integrate with OpenMemory for pattern learning
- [ ] Implement predictive failure detection
- [ ] Track success rates by failure type
- [ ] Refine evaluation prompts based on outcomes

---

## Related Documentation

- [Play Workflow Guide](../2025-12/play-workflow-guide.html) - Interactive stage visualization
- [Linear Integration](linear-integration-workflow.md) - Linear Agent API integration
- [Troubleshooting](troubleshooting.md) - Known issues and debugging
- [Healer Templates](../../templates/healer) - Remediation prompt templates
