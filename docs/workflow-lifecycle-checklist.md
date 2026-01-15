# CTO Platform Workflow Lifecycle Checklist

This document outlines the complete lifecycle from PRD to deployed code, with verification conditions for each step.

---

## Phase 1: Intake (PRD → Tasks)

### Step 1.1: PRD Submission
- User submits PRD via MCP `intake` tool or directly in Linear
- **Conditions to verify:**
  - PRD content is non-empty
  - `project_name` is provided
  - `cto-config.json` exists (optional, provides defaults)
  - Target repository is accessible

### Step 1.2: Linear Project Creation
- Morgan creates Linear project and PRD issue
- **Conditions to verify:**
  - Linear API key is valid (`LINEAR_API_KEY`)
  - Linear team ID is configured
  - Project created with correct name
  - PRD issue created with full content in description
  - `architecture.md` and `cto-config.json` attached (if provided)
  - Morgan auto-assigned as delegate

### Step 1.3: Intake Workflow Triggered
- PM Server webhook receives Linear event → triggers Argo Workflow
- **Conditions to verify:**
  - PM Server is running and receiving webhooks
  - Argo Workflow namespace is accessible
  - `intake-workflow` template exists
  - Workflow pod starts successfully

### Step 1.4: Task Generation (AI)
- Intake CLI parses PRD and generates tasks with AI
- **Conditions to verify:**
  - AI model accessible (Anthropic API key valid)
  - Tasks generated with correct structure (id, title, description, dependencies)
  - Agent hints assigned via content-based routing
  - Subtasks generated with execution levels (if complexity warrants)
  - Complexity analysis completed (if enabled)

### Step 1.5: Auto-Append Deploy Task (Optional)
- If `autoAppendDeployTask: true`, a final Bolt deploy task is added
- **Conditions to verify:**
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

---

## Phase 2: Play Workflow (Tasks → Implementation)

### Step 2.1: Play Submission
- User triggers `play` MCP tool or workflow starts automatically after intake PR merges
- **Conditions to verify:**
  - Target repository has `.tasks/` folder
  - `tasks.json` contains at least one task
  - Task dependencies are resolvable (no cycles)
  - Required agents configured in `cto-config.json`

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
  - CLI tool configured (claude, cursor, codex, opencode, gemini)
  - Model configured for agent
  - Tools (remote + local) configured
  - Linear integration configured (session_id, issue_id, team_id)
  - Subtasks included if present

### Step 2.4: Workspace Setup
- PVC created/mounted, repository cloned
- **Conditions to verify:**
  - PVC created or existing PVC mounted
  - Repository cloned to `/workspace`
  - Correct branch checked out
  - SSH keys / GitHub token available for push
  - `.tasks/` folder present in workspace

### Step 2.5: Agent Prompt Generation
- Handlebars templates rendered with task context
- **Conditions to verify:**
  - `prompt.md` and `prompt.xml` readable
  - All template variables resolved
  - Agent-specific partials included
  - Subagent config passed if `subagents.enabled: true`
  - Subtasks passed to template context

### Step 2.6: CLI Invocation
- Agent CLI started with generated prompt
- **Conditions to verify:**
  - CLI binary available (`claude`, `cursor`, etc.)
  - Model accessible
  - Tools server reachable (if remote tools configured)
  - MCP servers started (if local tools configured)
  - Linear sidecar running (for activity updates)

### Step 2.7: Implementation
- Agent writes code, creates commits
- **Conditions to verify:**
  - Agent has write access to workspace
  - Commits follow conventional commit format
  - Tests passing (if `test_strategy` defined)
  - Lints passing

### Step 2.8: Acceptance Criteria Probe
- Probe script checks acceptance criteria completion
- **Conditions to verify:**
  - `acceptance.md` has checkboxes
  - Checkboxes are checked by agent
  - All required criteria met (>= threshold)

### Step 2.9: Retry Logic (if criteria not met)
- Agent retries with model rotation
- **Conditions to verify:**
  - Retry count < `max_retries`
  - Model rotation applied (if configured)
  - **Fresh Start** triggered if retry count > `freshStartThreshold`
    - Context files cleared (`.conversation_id`, `.session_state`, `.agent_context`)
    - Agent restarts with clean context

### Step 2.10: PR Creation (per task)
- Agent creates PR for completed task
- **Conditions to verify:**
  - Branch created with task changes
  - PR created to target branch
  - PR description includes acceptance criteria status
  - CI checks pass

---

## Phase 3: Quality Gates

### Step 3.1: Cleo Review (Quality)
- Cleo agent reviews code quality
- **Conditions to verify:**
  - CodeRun created for Cleo with `job: review`
  - Cleo has access to PR diff
  - Review comments posted to PR (if issues found)
  - Approval or request-changes status set

### Step 3.2: Cipher Review (Security)
- Cipher agent performs security analysis
- **Conditions to verify:**
  - CodeRun created for Cipher with `job: security`
  - Security scan tools available (gitleaks, trivy, semgrep)
  - No critical/high vulnerabilities
  - Security report generated

### Step 3.3: Tess Testing
- Tess agent runs/writes tests
- **Conditions to verify:**
  - CodeRun created for Tess with `job: test`
  - Test framework available for language
  - All tests pass
  - Coverage meets threshold (if configured)

---

## Phase 4: Integration & Merge

### Step 4.1: Atlas Merge
- Atlas agent merges PR after all checks pass
- **Conditions to verify:**
  - All CI checks passed
  - All required reviews approved
  - No merge conflicts
  - Branch is up-to-date with target
  - PR merged successfully

### Step 4.2: Linear Status Update
- Task status updated to "Done" in Linear
- **Conditions to verify:**
  - Linear issue state changed
  - Activity logged via Linear sidecar
  - Linked PR shown in Linear

---

## Phase 5: Deployment (Final Task)

### Step 5.1: Bolt Deploy Task
- Final Bolt task deploys to production (if PRD includes deployment)
- **Conditions to verify:**
  - All implementation tasks completed
  - Deploy task dependencies satisfied
  - Infrastructure credentials available
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
  - Local customizations preserved (test_strategy, agent_hint, priority only update if explicitly set in Linear)

---

## Monitoring & Observability

### Healer Monitoring
- Healer daemon monitors workflow health
- **Conditions to verify:**
  - Prometheus metrics being scraped
  - Loki logs being collected
  - Alerts firing on failures
  - Auto-remediation triggered (if configured)

### Linear Activity
- All agent activity logged to Linear
- **Conditions to verify:**
  - Linear sidecar running
  - Activities appearing in Linear issue
  - Signals (errors, completions) logged

---

## Quick Reference: Key Environment Variables

| Variable | Purpose | Required |
|----------|---------|----------|
| `LINEAR_API_KEY` | Linear API access | Yes |
| `GITHUB_TOKEN` | Repository access | Yes |
| `ANTHROPIC_API_KEY` | AI model access | Yes (for Claude) |
| `OPENAI_API_KEY` | AI model access | Yes (for GPT) |
| `GOOGLE_API_KEY` | AI model access | Yes (for Gemini) |

## Quick Reference: Key Config Files

| File | Purpose |
|------|---------|
| `cto-config.json` | Agent assignments, CLI config, tool profiles |
| `.tasks/tasks/tasks.json` | Task definitions and dependencies |
| `.tasks/docs/{id}/prompt.md` | Agent prompt for task |
| `.tasks/docs/{id}/acceptance.md` | Acceptance criteria checklist |
