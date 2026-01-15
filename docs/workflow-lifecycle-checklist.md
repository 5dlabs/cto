# CTO Platform Workflow Lifecycle Checklist

This document outlines the complete lifecycle from PRD to deployed code, with verification conditions for each step.

---

## Pre-Flight: Local Environment Setup

### Step 0.1: Start Local Services
- PM Server, Healer, and Controller must be running locally
- **Conditions to verify:**
  - PM Server running: `curl http://localhost:8080/health`
  - Healer running: `curl http://localhost:8081/health`
  - Controller running: check logs for "started"
  - All services connected to Kubernetes cluster

### Step 0.2: Cloudflare Tunnels
- Webhook URLs must be accessible from Linear
- **Conditions to verify:**
  - Cloudflare tunnel running (`cloudflared tunnel list`)
  - PM Server webhook URL reachable from internet
  - Linear webhook configured with correct URL
  - Test webhook delivery in Linear settings

### Step 0.3: Credentials Verification
- OAuth and API credentials configured correctly
- **Conditions to verify:**
  - Linear OAuth credentials configured (NOT API key)
  - GitHub App installed with correct permissions
  - Anthropic API key valid (for Claude)
  - All credentials in `cto-config.json` or environment

---

## Phase 1: Intake (PRD → Tasks)

### Step 1.1: PRD Submission via MCP Tool
- Use MCP `intake` tool to create Linear issue and auto-assign Morgan
- **Conditions to verify:**
  - MCP tool invoked (NOT local mode - `local` parameter must not exist)
  - PRD content is non-empty
  - `project_name` is provided
  - `cto-config.json` is provided (**REQUIRED**, not optional)
  - Target repository is accessible
  - Linear issue created with PRD in description
  - Morgan auto-assigned as delegate

### Step 1.2: Linear Project Creation
- Morgan creates Linear project and PRD issue
- **Conditions to verify:**
  - Linear OAuth token is valid (not API key)
  - Linear team ID is configured
  - Project created with correct name
  - PRD issue created with full content in description
  - `architecture.md` attached (if provided)
  - `cto-config.json` attached (**REQUIRED**)
  - Morgan auto-assigned as delegate
  - Morgan agent app assigned to issue (two-way dialog enabled)

### Step 1.3: Intake Workflow Triggered
- PM Server webhook receives Linear event → triggers Argo Workflow
- **Conditions to verify:**
  - PM Server is running and receiving webhooks
  - Webhook URL accessible via Cloudflare tunnel
  - Argo Workflow namespace is accessible
  - `intake-workflow` template exists
  - Workflow pod starts successfully
  - Intake running in **CLI mode** (not API mode)

### Step 1.4: Task Generation (AI - CLI Mode)
- Intake CLI parses PRD and generates tasks with AI
- **Conditions to verify:**
  - Running in CLI mode (check logs for CLI invocation)
  - AI model accessible (Anthropic API key valid)
  - Tasks generated with correct structure (id, title, description, dependencies)
  - Agent hints assigned via content-based routing (NOT hardcoded Task 1 = Bolt)
  - Subtasks generated with execution levels (if complexity warrants)
  - Complexity analysis completed (if enabled)
  - **Test strategy defined for each task** (important for downstream testing)

### Step 1.5: Auto-Append Deploy Task
- If `autoAppendDeployTask: true` in config, final Bolt deploy task is added
- **Conditions to verify:**
  - `autoAppendDeployTask: true` passed through in `cto-config.json`
  - No existing deploy task detected
  - Deploy task has all other tasks as dependencies
  - Deploy task assigned to `bolt` agent

### Step 1.6: Documentation Generation
- Agent prompts (MD + XML) and acceptance criteria files created
- **Conditions to verify:**
  - `.tasks/tasks/tasks.json` created
  - `.tasks/docs/{task_id}/prompt.md` for each task
  - `.tasks/docs/{task_id}/prompt.xml` for each task
  - `.tasks/docs/{task_id}/acceptance.md` for each task
  - `cto-config.json` generated with agent tool configurations
  - **Subtasks included in task files** (for subagent dispatch)

### Step 1.7: PR Creation
- Intake creates PR to target repository with generated files
- **Conditions to verify:**
  - GitHub App has write access to repository
  - PR created from `intake/{project}` branch
  - PR contains all `.tasks/` files
  - PR description includes task summary

### Step 1.8: Linear Issue Sync
- Each task becomes a Linear issue linked to the project
- **Conditions to verify:**
  - Linear issues created for each task
  - Issues have correct titles and descriptions
  - Dependencies reflected in Linear (blocks/blocked-by)
  - Issues in "Backlog" or "Todo" state
  - Linear updated dynamically as workflow progresses

### Step 1.9: Linear Edit → Task Sync (Mid-Flight Update Test)
- Edit a Linear issue and verify task/prompt updates in GitHub
- **Conditions to verify:**
  - Edit issue title in Linear → task title updated
  - Edit issue description in Linear → task description updated
  - Edit acceptance criteria in Linear → `acceptance.md` updated
  - Changes appear as new commit or PR in target repo
  - Local customizations preserved (`test_strategy`, `agent_hint` only update if explicitly set)
  - Sync completes before any agent starts working on that task

---

## Phase 2: Play Workflow (Tasks → Implementation)

### Step 2.1: Play Submission
- User triggers `play` MCP tool or workflow starts automatically after intake PR merges
- **Conditions to verify:**
  - Target repository has `.tasks/` folder
  - `tasks.json` contains at least one task
  - Task dependencies are resolvable (no cycles)
  - Required agents configured in `cto-config.json`
  - Using Claude CLI for this test

### Step 2.2: Task Selection
- Next available task selected based on dependencies and priority
- **Conditions to verify:**
  - All dependency tasks are `completed`
  - Task status is `pending` or `in_progress`
  - If `parallel_execution: true`, multiple tasks may start simultaneously

### Step 2.3: CodeRun Creation
- Kubernetes CodeRun CR created for the selected task
- **Conditions to verify:**
  - CodeRun CR created in correct namespace
  - Agent correctly assigned (from task `agent_hint` or routing)
  - CLI tool configured: **Claude** for this test
  - Model configured for agent
  - **Agent app assigned to Linear issue** (same as Morgan for intake)
  - **Two-way communication enabled** via Linear agent dialog
  - Tools (remote + local) configured
  - Linear integration configured (session_id, issue_id, team_id)
  - Subtasks included if present

### Step 2.4: Two-Way Communication Test
- Verify agent dialog works bidirectionally through Linear
- **Conditions to verify:**
  - Agent can receive messages from Linear input
  - Agent activities appear in Linear issue timeline
  - User can send message via Linear → agent receives it
  - Agent signals (errors, completions) logged in Linear
  - Messages flow through Linear sidecar correctly

### Step 2.5: Workspace Setup
- PVC created/mounted, repository cloned
- **Conditions to verify:**
  - PVC created or existing PVC mounted
  - Repository cloned to `/workspace`
  - Correct branch checked out
  - SSH keys / GitHub token available for push
  - `.tasks/` folder present in workspace

### Step 2.6: Agent Prompt Generation
- Handlebars templates rendered with task context
- **Conditions to verify:**
  - `prompt.md` and `prompt.xml` readable
  - All template variables resolved
  - Agent-specific partials included
  - **Correct prompts for each agent** (verify in logs)
  - Subagent config passed if `subagents.enabled: true`
  - Subtasks passed to template context

### Step 2.6.1: Skills Loading (Claude CLI Test)
- Skills copied to native skill directory based on agent + job type
- **Conditions to verify:**
  - `skill-mappings.yaml` loaded from `/templates/skills/`
  - Agent's **default skills** loaded (always):
    - Context engineering: `context-fundamentals`, `context-degradation`, `context-optimization`
    - Tool skills: `openmemory`, `context7`, `llm-docs`, `github-mcp`
    - Agent-specific skills based on `skill-mappings.yaml`
  - Agent's **job-type skills** merged (based on `coder`, `healer`, `test`, etc.)
  - **Optional skills triggered** by keywords in task description (if configured)
  - Skills copied to `$CLAUDE_WORK_DIR/.claude/skills/` directory
  - Each skill has `SKILL.md` file copied
  - **Log output shows:**
    ```
    ═══════════════════════════════════════════════════════════════
    ║               SKILLS SETUP                                   ║
    ═══════════════════════════════════════════════════════════════
    Setting up Claude Code skills...
      ✓ Source: /templates/skills
      ✓ Target: /workspace/.claude/skills
      ✓ Loaded skill: context-fundamentals
      ✓ Loaded skill: context-degradation
      ...
    📚 Skills summary
      ✓ Loaded count: 12
      ✓ Loaded list: context-fundamentals context-degradation ...
    ```
  - Verify specific skills for test agent (e.g., Rex):
    - Default: `rust-patterns`, `mcp-development`
    - Coder job: `tool-design`, `firecrawl`

**Skills by Agent Reference (for verification):**

| Agent | Default Skills | Job-Specific Skills |
|-------|---------------|---------------------|
| **blaze** | shadcn-stack, anime-js, effect-frontend-patterns, frontend-excellence | coder: tool-design, firecrawl |
| **rex** | rust-patterns, mcp-development | coder: tool-design, firecrawl |
| **grizz** | go-patterns | coder: tool-design, firecrawl |
| **nova** | effect-patterns | coder: tool-design, better-auth, firecrawl |
| **bolt** | kubernetes-operators, argocd-gitops, secrets-management, storage-operators | deploy: tool-design |
| **cleo** | code-review, evaluation, advanced-evaluation, repomix, firecrawl | quality: tool-design |
| **tess** | testing-strategies, evaluation, advanced-evaluation, webapp-testing | test: tool-design |
| **cipher** | security-analysis | security: tool-design |
| **morgan** | project-development, skill-authoring | intake: prd-analysis, multi-agent-patterns |
| **atlas** | git-integration, repomix | integration: multi-agent-patterns |

**Common skills loaded for ALL agents:**
- `context-fundamentals`, `context-degradation`, `context-optimization`
- `openmemory`, `context7`, `llm-docs`, `github-mcp`

### Step 2.6.2: Skills Native vs Baked-In
- Different CLIs handle skills differently
- **CLIs with Native Skill Support (files copied to skill directory):**
  - Claude Code: `.claude/skills/`
  - Factory/Droid: `.factory/skills/`
  - OpenCode: `.claude/skills/` (compatible)
  - Codex: `.codex/skills/`
- **CLIs without Native Skills (baked into AGENTS.md):**
  - Cursor, Gemini, other CLIs
- **Conditions to verify (for Claude test):**
  - Skill files exist in `.claude/skills/{skill-name}/SKILL.md`
  - Claude Code discovers and loads skills automatically
  - Skills appear in Claude's context window

### Step 2.7: Subagent Dispatch (Claude with Subtasks)
- Top-level agent spawns subagents for subtasks
- **Conditions to verify:**
  - `subagents.enabled: true` in agent config
  - `maxConcurrent` set appropriately (e.g., 5)
  - Subtasks grouped by `execution_level`
  - Subagents spawned for parallel execution
  - Coordinator agent manages subtask completion
  - Each subagent has focused context

### Step 2.8: Tool Usage Verification
- Verify configured tools are actually being used
- **Conditions to verify:**
  - Tools defined in `cto-config.json` appear in agent context
  - **Tools showing up in logs** as being invoked
  - MCP servers started (if local tools configured)
  - Remote tool server reachable (if configured)
  - Tool filtering working correctly per agent

### Step 2.9: Implementation
- Agent writes code, creates commits
- **Conditions to verify:**
  - Agent has write access to workspace
  - Commits follow conventional commit format
  - Tests passing (if `test_strategy` defined)
  - Lints passing
  - Linear updated with progress activities

### Step 2.10: Acceptance Criteria Probe
- Probe script checks acceptance criteria completion
- **Conditions to verify:**
  - `acceptance.md` has checkboxes
  - Checkboxes are checked by agent
  - All required criteria met (>= threshold)

### Step 2.11: Model Rotation Test
- Test retry with model rotation between Opus and Sonnet
- **Conditions to verify:**
  - First attempt uses configured model (e.g., Sonnet)
  - On retry, model rotates (e.g., to Opus)
  - Model name appears correctly in logs
  - Both models can complete the task

### Step 2.12: Fresh Start Test (if retry > threshold)
- Agent retries with fresh context after threshold
- **Conditions to verify:**
  - Retry count > `freshStartThreshold` (default: 3)
  - Context files cleared (`.conversation_id`, `.session_state`, `.agent_context`)
  - Agent restarts with clean context
  - Task can still complete after fresh start

### Step 2.13: PR Creation (per task)
- Agent creates PR for completed task
- **Conditions to verify:**
  - Branch created with task changes
  - PR created to target branch
  - PR description includes acceptance criteria status
  - CI checks pass

---

## Phase 3: Quality Gates

### Step 3.1: Bolt Infrastructure Setup (Task 1 if applicable)
- Bolt sets up infrastructure with Helm chart and shared config
- **Conditions to verify:**
  - Helm chart created in `/charts/{project}` or similar
  - Chart can be used by Tess for deployment testing
  - **Shared ConfigMap created** with URLs and secrets
  - ConfigMap accessible to downstream agents
  - Kubernetes resources created (namespace, RBAC, etc.)

### Step 3.2: Cleo Review (Quality)
- Cleo agent reviews code quality based on implementation language
- **Conditions to verify:**
  - CodeRun created for Cleo with `job: review` (triggers `quality` job-type skills)
  - Cleo has access to PR diff
  - **Skills loaded for language/framework:**
    - Default: `code-review`, `evaluation`, `advanced-evaluation`, `repomix`, `firecrawl`
    - Optional (triggered by task content):
      - `rust-patterns` for Rust projects
      - `go-patterns` for Go projects
      - `effect-patterns` for TypeScript/Effect projects
      - `shadcn-stack` for React/Next.js frontend
      - `better-auth` for auth-related code
  - **Quality checks conditional on implementation agent/language:**
    - Rust: `cargo fmt`, `cargo clippy --pedantic`, idioms
    - TypeScript/React: ESLint, Prettier, React best practices
    - Go: `go fmt`, `go vet`, idioms
    - Python: Black, Ruff, type hints
  - **Context7 access configured** for documentation lookup
  - **Correct prompt for language/frameworks** in use
  - Best practices applied per language
  - Review comments posted to PR (if issues found)
  - Approval or request-changes status set

### Step 3.3: Cipher Review (Security)
- Cipher agent performs security analysis relative to implementation
- **Conditions to verify:**
  - CodeRun created for Cipher with `job: security`
  - **Skills loaded for security analysis:**
    - Default: `security-analysis`, `observability`
    - Optional (triggered by task content):
      - `rust-patterns` for Rust-specific security
      - `go-patterns` for Go-specific security
      - `effect-patterns` for TypeScript security
      - `better-auth` for auth/OAuth/token security
  - **Analysis relative to implementation agent nuances:**
    - Rust: unsafe blocks, memory safety
    - TypeScript: XSS, CSRF, injection
    - Go: race conditions, input validation
    - Python: SQL injection, pickle deserialization
  - Security scan tools available (gitleaks, trivy, semgrep)
  - No critical/high vulnerabilities
  - Security report generated
  - Linear updated with security status

### Step 3.4: Tess Testing (Unit, Integration, E2E)
- Tess agent runs comprehensive test suite
- **Conditions to verify:**
  - CodeRun created for Tess with `job: test`
  - **Test strategy from intake** used to guide testing
  - Test framework available for language
  
  **Unit Tests:**
  - Unit tests written/run for new code
  - Coverage meets threshold (if configured)
  
  **Integration Tests:**
  - Integration tests run against local services
  - Database/API mocks or real services used
  
  **Full E2E Tests (Kubernetes):**
  - Application deployed to Kubernetes (using Helm chart from Bolt)
  - Health endpoints verified: `curl http://{service}/health`
  - **All exposed endpoints tested** with curl/httpie
  - Smoke tests pass
  - Application logs checked for errors
  - Deployment cleaned up after tests

---

## Phase 4: Integration & Merge

### Step 4.1: Atlas Merge
- Atlas agent merges PR after all checks pass
- **Conditions to verify:**
  - All CI checks passed
  - All required reviews approved (Cleo, Cipher)
  - No merge conflicts
  - Branch is up-to-date with target
  - PR merged successfully
  - Linear status dynamically updated

### Step 4.2: Linear Status Update
- Task status updated to "Done" in Linear
- **Conditions to verify:**
  - Linear issue state changed to "Done"
  - Activity logged via Linear sidecar
  - Linked PR shown in Linear
  - Duration/metrics captured

---

## Phase 5: Deployment (Final Task)

### Step 5.1: Bolt Deploy Task
- Final Bolt task deploys to production
- **Conditions to verify:**
  - All implementation tasks completed
  - Deploy task dependencies satisfied
  - Infrastructure credentials available (from shared ConfigMap)
  - Target environment accessible

### Step 5.2: Deployment Execution
- Bolt applies infrastructure changes and deploys
- **Conditions to verify:**
  - Kubernetes manifests valid
  - Helm charts lint successfully
  - ArgoCD app synced (if GitOps)
  - Health checks pass
  - Smoke tests pass

### Step 5.3: Verification
- Production deployment verified
- **Conditions to verify:**
  - Application accessible at expected URL
  - Health endpoints returning 200
  - No error spikes in logs
  - Metrics within expected range

---

## Phase 6: Post-Flight Verification

### Step 6.1: Telemetry Verification
- Verify all expected telemetry was captured
- **Conditions to verify:**
  - **Logs captured in Loki:**
    - Agent invocation logs
    - Tool usage logs
    - Error logs (if any)
    - Retry/fresh-start events
  - **Metrics captured in Prometheus:**
    - Task duration
    - Agent completion rate
    - Retry counts
    - Tool invocation counts
  - Grafana dashboards showing workflow data

### Step 6.2: Linear Activity Audit
- Verify all agent activity logged to Linear
- **Conditions to verify:**
  - All agents (Morgan, implementation, Cleo, Cipher, Tess, Atlas, Bolt) logged activities
  - Two-way communication worked (user messages received)
  - Signals logged (completion, errors)
  - Timeline shows complete workflow history

---

## Mid-Flight Updates (Optional)

### Update from PRD/Architecture Changes
- Use `intake update` to re-parse and generate delta
- **Conditions to verify:**
  - Existing tasks loaded
  - Delta computed (added/modified/removed)
  - PR created with changes only

### Sync from Linear Edits
- Use `intake sync-task` to pull changes from Linear issue
- **Conditions to verify:**
  - Linear issue fetched successfully
  - Task fields updated (title, description, acceptance criteria)
  - Local customizations preserved (`test_strategy`, `agent_hint`, `priority` only update if explicitly set in Linear)

---

## Quick Reference: Test Configuration

### Required `cto-config.json` Settings for This Test

```json
{
  "defaults": {
    "intake": {
      "autoAppendDeployTask": true
    },
    "play": {
      "freshStartThreshold": 3,
      "cli": "claude"
    }
  },
  "agents": {
    "rex": {
      "cli": "claude",
      "model": "claude-sonnet-4-5-20250514",
      "subagents": {
        "enabled": true,
        "maxConcurrent": 5
      }
    },
    "blaze": {
      "cli": "claude",
      "model": "claude-sonnet-4-5-20250514",
      "subagents": {
        "enabled": true,
        "maxConcurrent": 5
      }
    }
  },
  "linear": {
    "useOAuth": true
  }
}
```

### Model Rotation Configuration

```json
{
  "agents": {
    "rex": {
      "models": [
        "claude-sonnet-4-5-20250514",
        "claude-opus-4-5-20250514"
      ]
    }
  }
}
```

### Key Environment Variables

| Variable | Purpose | Required |
|----------|---------|----------|
| `LINEAR_OAUTH_TOKEN` | Linear OAuth access (preferred) | Yes |
| `GITHUB_TOKEN` | Repository access | Yes |
| `ANTHROPIC_API_KEY` | AI model access (Claude) | Yes |
| `CLOUDFLARE_TUNNEL_TOKEN` | Tunnel authentication | Yes |

### Key Config Files

| File | Purpose |
|------|---------|
| `cto-config.json` | Agent assignments, CLI config, tool profiles (**REQUIRED**) |
| `.tasks/tasks/tasks.json` | Task definitions and dependencies |
| `.tasks/docs/{id}/prompt.md` | Agent prompt for task |
| `.tasks/docs/{id}/acceptance.md` | Acceptance criteria checklist |

---

## Test Strategy in Intake

The `test_strategy` field should be defined for each task during intake. This guides Tess in selecting appropriate testing approaches:

| Test Strategy | Description | Example |
|---------------|-------------|---------|
| `unit` | Unit tests only | Pure functions, utilities |
| `integration` | Unit + integration tests | Services with dependencies |
| `e2e` | Full end-to-end tests | User-facing features |
| `smoke` | Basic health/smoke tests | Infrastructure tasks |
| `manual` | Manual verification required | UI/UX tasks |

Example in `tasks.json`:
```json
{
  "id": 3,
  "title": "Implement user authentication",
  "test_strategy": "e2e",
  "agent_hint": "nova"
}
```
