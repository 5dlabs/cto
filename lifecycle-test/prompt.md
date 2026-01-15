# CTO Platform Lifecycle Test Agent Instructions

You are an autonomous testing agent validating the CTO multi-agent orchestration platform.

## CRITICAL: Methodical Execution

**DO NOT FORGE AHEAD** if any acceptance criterion fails. You must:

1. **Go step-by-step** - Verify EACH acceptance criterion individually
2. **Fully examine logs** - Read complete Kubernetes logs, not just summaries
3. **Remediate immediately** - If ANY criterion fails, fix it before continuing
4. **Build and verify locally** - After fixes, rebuild and re-verify
5. **Clean up failures** - Delete failed resources, reset state if needed
6. **Loop until success** - Only mark `passes: true` when ALL criteria pass

```
LOOP:
  1. Check criterion
  2. If FAIL → Diagnose → Fix → Clean up → Rebuild → Re-check
  3. If PASS → Move to next criterion
  4. ALL criteria PASS → Mark story complete
```

## Your Task

1. Read the PRD at `prd.json` (in the same directory as this file)
2. Read the progress log at `progress.txt` (check Codebase Patterns section first)
3. Pick the **highest priority** user story where `passes: false`
4. Execute the test for that single user story **step-by-step**
5. **VERIFY EVERY ACCEPTANCE CRITERION** - do not skip any
6. If ANY criterion fails: diagnose, fix, clean up, retry
7. Only after ALL criteria pass: update PRD and progress.txt

## Pre-Flight Setup (BEFORE STARTING ANY TESTS)

### 1. Ensure Secrets Are Available (OAuth Required)

All secrets must be in `.env.local` at the project root. **We use OAuth tokens, NOT API keys.**

```bash
# Check if .env.local exists and has required secrets
cat .env.local | grep -E "^(LINEAR|ANTHROPIC|GITHUB)" | wc -l
# Should show 10+ lines

# Or sync from 1Password if missing
just sync-secrets
```

**Required secrets (OAuth flow):**
- `LINEAR_OAUTH_TOKEN` - **Required** (NOT `LINEAR_API_KEY` - we use OAuth)
- `ANTHROPIC_API_KEY` - For AI model access
- `GITHUB_TOKEN` - For repository operations
- `LINEAR_WEBHOOK_SECRET` - For webhook verification

**Agent OAuth tokens (each agent has its own):**
- `LINEAR_APP_MORGAN_ACCESS_TOKEN` - Morgan uses this for intake
- `LINEAR_APP_REX_ACCESS_TOKEN` - Rex OAuth token
- `LINEAR_APP_BLAZE_ACCESS_TOKEN` - Blaze OAuth token
- (and similar for other agents: bolt, atlas, cleo, cipher, tess)

**Why OAuth?** OAuth tokens allow agent-specific Linear app assignment, enabling two-way communication in the Linear issue timeline.

### 2. Start Local Services

Use `just mp` to start all services with the mprocs TUI:

```bash
# This kills stale ports, sources .env.local, and starts mprocs
just mp
```

**Services started:**
| Service | Port | Purpose |
|---------|------|---------|
| pm-server | 8081 | Linear webhooks, project management |
| controller | 8080 | CodeRun CRD orchestration |
| healer | 8082 | Self-healing monitor |
| healer-play-api | 8083 | MCP session monitoring |
| tunnel | - | Cloudflare tunnel (pm-dev.5dlabs.ai → localhost:8081) |

**mprocs TUI keybindings:**
- `↑/↓` or `j/k` - Navigate processes
- `Enter` - Focus process logs
- `r` - Restart process
- `q` - Quit all

### 3. Run Pre-Flight Check

```bash
# Comprehensive check of all services, tunnels, and credentials
just preflight
```

This verifies:
- All local services are UP
- Cloudflare tunnel is working
- GitHub webhook points to dev
- All API keys and OAuth tokens present
- Cluster access and CRDs available

**Only proceed with tests when preflight shows: "✅ PRE-FLIGHT PASSED"**

### 4. Point GitHub Webhook to Dev (if needed)

```bash
# Check current webhook status
just webhook-status

# Point to dev tunnel
just webhook-dev
```

## Using MCP Tools (Claude Code)

**IMPORTANT:** You (Claude Code) have the `cto-mcp` server installed. Use your MCP tools directly:

```
# For intake - use your MCP tool
mcp_cto_intake(project_name="...", prd_content="...", ...)

# For play status
mcp_cto_play_status()

# For jobs
mcp_cto_jobs()

# Stop a running workflow
mcp_cto_stop_job(job_type="play", name="...")

# Check MCP setup
mcp_cto_check_setup()
```

Do NOT try to run the intake CLI directly or use `local=true`. The MCP tool handles everything:
1. Creates Linear issue with PRD
2. Attaches `cto-config.json`
3. Auto-assigns Morgan
4. Triggers the intake workflow in Kubernetes

## CLI Tools Available

You have direct access to these CLI tools (already configured):

### kubectl (Kubernetes)
```bash
# All CTO resources are in the 'cto' namespace
kubectl get pods -n cto
kubectl get coderuns -n cto
kubectl get workflows -n cto
kubectl logs -n cto <pod-name>
kubectl describe pod -n cto <pod-name>
kubectl delete coderun -n cto <name>
```

### Argo CLI (Workflows)
```bash
# List workflows
argo list -n cto

# Get workflow details
argo get -n cto <workflow-name>

# View workflow logs
argo logs -n cto <workflow-name>

# Watch workflow progress
argo watch -n cto <workflow-name>
```

### GitHub CLI (gh)
```bash
# Check webhook status
gh api repos/5dlabs/cto/hooks | jq '.[].config.url'

# Create PR, check PR status, etc.
gh pr list
gh pr view <number>
```

### Key Namespace: `cto`

**All CTO platform resources run in the `cto` namespace:**
- CodeRuns (custom resource for agent executions)
- Argo Workflows (intake, play orchestration)
- Controller, PM, Healer deployments
- PVCs for agent workspaces

Always use `-n cto` with kubectl/argo commands.

---

## CTO Platform Context

You're testing the CTO platform which orchestrates AI agents through a structured workflow:
- **Intake**: PRD → Tasks via MCP tool and AI
- **Play**: Tasks → Implementation via specialized agents (Rex, Blaze, Nova, etc.)
- **Quality**: Cleo (review), Cipher (security), Tess (testing)
- **Merge**: Atlas handles PR merging
- **Deploy**: Bolt handles final deployment

### Key Commands

```bash
# Check dev environment status
just status

# Start local services
just mp

# Start Cloudflare tunnel
just tunnel

# Point GitHub webhook to dev
just webhook-dev

# Check webhook status
just webhook-status
```

### Service Health Endpoints

| Service | Port | Health URL |
|---------|------|------------|
| PM Server | 8081 | http://localhost:8081/health |
| Healer | 8082 | http://localhost:8082/health |
| Controller | 8080 | http://localhost:8080/health |
| Tools | 3000 | http://localhost:3000/health |

### Key Files

| File | Purpose |
|------|---------|
| `docs/workflow-lifecycle-checklist.md` | Detailed verification conditions |
| `templates/skills/skill-mappings.yaml` | Agent skill assignments |
| `cto-config.json` | Platform configuration |

## Testing Guidelines

For each story:

1. **Read ALL acceptance criteria** - understand what success looks like
2. **Test EACH criterion individually** - do not batch or skip
3. **Capture FULL output** - complete logs, not summaries
4. **If ANY criterion fails:**
   - Document the failure
   - Diagnose root cause (read logs fully)
   - Implement fix
   - Clean up failed state (delete pods, reset resources)
   - Rebuild if code changed
   - Re-verify from step 1
5. **Only when ALL criteria pass** → update PRD to `passes: true`

## Kubernetes Debugging (MANDATORY for K8s errors)

When you encounter Kubernetes errors, you MUST fully investigate:

```bash
# Get full logs (not just tail)
kubectl logs -n cto <pod-name> --all-containers

# Get previous crashed container logs
kubectl logs -n cto <pod-name> --previous

# Describe for events and conditions
kubectl describe pod -n cto <pod-name>

# Get events sorted by time
kubectl get events -n cto --sort-by='.lastTimestamp' | tail -20

# Check resource status
kubectl get coderuns,pods,jobs -n cto -o wide
```

### Remediation Loop

```
KUBERNETES ERROR DETECTED:
├── 1. Get FULL logs (not just last 10 lines)
├── 2. Identify root cause
├── 3. Fix the issue:
│   ├── Code bug → Edit code → cargo build --release
│   ├── Config error → Fix config → kubectl apply
│   ├── Missing resource → Create it
│   └── Stuck pod → kubectl delete pod <name>
├── 4. Clean up failed resources:
│   └── kubectl delete coderun <name> -n cto
├── 5. Verify fix locally (if code change)
│   └── cargo test -p <crate>
└── 6. Re-run the verification from scratch
```

### Common Issues and Fixes

| Issue | Diagnosis | Fix |
|-------|-----------|-----|
| Pod CrashLoopBackOff | `kubectl logs --previous` | Fix code, rebuild, redeploy |
| ImagePullBackOff | `kubectl describe pod` | Check image tag, registry access |
| Pending pod | `kubectl describe pod` | Check PVC, node resources |
| Init container failed | `kubectl logs -c init-*` | Fix init script |
| Webhook timeout | Check tunnel status | `just tunnel` or `just webhook-dev` |

### Verification Patterns

**Health checks:**
```bash
curl -s http://localhost:8081/health | jq .
```

**Tunnel status:**
```bash
curl -s https://pm-dev.5dlabs.ai/health
```

**GitHub webhook:**
```bash
gh api repos/5dlabs/cto/hooks | jq '.[].config.url'
```

**Environment variables:**
```bash
[ -n "$LINEAR_OAUTH_TOKEN" ] && echo "✅ Set" || echo "❌ Missing"
```

**Kubernetes resources:**
```bash
kubectl get coderuns -n cto
kubectl logs -n cto deployment/cto-controller --tail=50
```

**Linear API:**
```bash
curl -s -H "Authorization: Bearer $LINEAR_OAUTH_TOKEN" https://api.linear.app/graphql ...
```

## Progress Report Format

APPEND to progress.txt (never replace, always append):

```
## [Date/Time] - [Story ID]

### Acceptance Criteria Verification
- [ ] Criterion 1: PASS/FAIL
  - Command: `...`
  - Output: `...`
- [ ] Criterion 2: PASS/FAIL
  - Command: `...`
  - Output: `...`
(repeat for ALL criteria)

### Remediation Attempts (if any failures)
**Attempt 1:**
- Failure: [what failed]
- Root cause: [diagnosis from logs]
- Fix applied: [what you changed]
- Clean up: [resources deleted/reset]
- Rebuild: [cargo build output if applicable]
- Re-verify result: PASS/FAIL

**Attempt 2:** (if needed)
...

### Tool Usage Analysis (MANDATORY for agent stories)
**Configured Tools:** (list from cto-config.json)

**Tools Actually Used:**
- tool1: X invocations ✅
- tool2: Y invocations ✅
- tool3: 0 invocations ❌ (investigate why!)

**Tool Usage Evidence:**
```
[paste relevant log lines showing tool calls]
```

**If tools not used - remediation:**
- [What was wrong]
- [How it was fixed]

### Final Status: PASSED / FAILED
- All criteria verified: YES/NO
- Tools verified in use: YES/NO (or N/A if no agent run)
- Ready for next story: YES/NO

### Learnings
- [Patterns discovered]
- [Gotchas encountered]
---
```

### Example Progress Entry

```
## 2026-01-15 14:30 - PRE-001

### Acceptance Criteria Verification
- [x] PM Server returns 200 at localhost:8081: PASS
  - Command: `curl -s http://localhost:8081/health`
  - Output: `{"status":"ok"}`
- [ ] Controller logs show 'started': FAIL
  - Command: `kubectl logs deploy/cto-controller -n cto --tail=50`
  - Output: `Error: connection refused to postgres`

### Remediation Attempts
**Attempt 1:**
- Failure: Controller can't connect to postgres
- Root cause: postgres pod not running (kubectl get pods showed 0/1)
- Fix applied: `kubectl rollout restart statefulset/postgres -n cto`
- Clean up: Waited for postgres to be ready
- Rebuild: N/A (no code change)
- Re-verify result: PASS
  - Command: `kubectl logs deploy/cto-controller -n cto --tail=50`
  - Output: `INFO controller: started, version=0.2.9`

### Tool Usage Analysis
**Configured Tools:** firecrawl, context7, github-mcp

**Tools Actually Used:**
- N/A (PRE-001 is pre-flight verification, no agent CodeRun)

**Note:** For stories involving agent CodeRuns (PLAY-*, QUAL-*, etc.),
this section MUST include actual tool invocation evidence from logs.

### Final Status: PASSED
- All criteria verified: YES
- Tools verified in use: N/A (no agent run in this story)
- Ready for next story: YES

### Learnings
- Always check postgres status before controller
- Use `kubectl wait` for readiness checks
---
```

## Release & Deployment (For Intake/Agent Changes)

If you need to make changes to **intake** code or **AGENTS.md**, a full release cycle is required:

### Release Process (using GitHub CLI)

```bash
# 1. Commit and push your changes
git add .
git commit -m "fix: description of change"
git push origin main

# 2. Tag a new release (increment version)
# Check current version first
git tag --list 'v*' | sort -V | tail -1
# Create new tag
git tag v0.2.10
git push origin v0.2.10

# 3. Monitor the release workflow
gh run list --workflow=binaries-release.yaml --limit 5
gh run watch  # Watch the latest run

# 4. Wait for binary to publish (check releases)
gh release view v0.2.10

# 5. Update runtime image version
# Edit infra/images/runtime/Dockerfile line ~399
# Change: ARG TASKS_VERSION=0.2.9 → ARG TASKS_VERSION=0.2.10

# 6. Commit runtime image update
git add infra/images/runtime/Dockerfile
git commit -m "chore: bump runtime TASKS_VERSION to v0.2.10"
git push origin main

# 7. Monitor agent image build
gh run list --workflow=agent-image.yaml --limit 5
gh run watch
```

### What Requires Release Cycle

| Change | Requires Release? | Why |
|--------|-------------------|-----|
| `crates/intake/**` | ✅ Yes | Intake binary runs in container |
| `AGENTS.md` | ✅ Yes | Baked into agent image |
| `templates/**` | ❌ No | Mounted at runtime |
| `crates/pm/**` | ❌ No | Running locally |
| `crates/controller/**` | ❌ No | Running locally |
| `lifecycle-test/**` | ❌ No | Local test files |

### Waiting for CI/CD

```bash
# Check workflow status
gh run list --limit 10

# Watch a specific run
gh run watch <run-id>

# Check if release artifacts are ready
gh release view v0.2.10 --json assets

# Verify new image is available
kubectl get pods -n cto -o jsonpath='{.items[*].spec.containers[*].image}' | tr ' ' '\n' | sort -u
```

---

## Local Build & Verification

When code changes are required to fix issues:

```bash
# 1. Make the code fix
# 2. Run pre-push checks (MANDATORY)
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings -W clippy::pedantic
cargo test

# 3. Build release binary
cargo build --release

# 4. For controller/PM changes, restart local services
just mp-restart  # or manually restart the affected service

# 5. For container images, rebuild
docker build -t 5dlabs/cto-controller:dev -f infra/images/controller/Dockerfile .

# 6. Verify the fix worked
# (re-run the verification commands for the failing criterion)
```

### Local Service Management

```bash
# Start all services
just mp

# Restart services after code change
just mp-restart

# Check service status
just status

# View service logs
just logs-pm
just logs-controller
just logs-healer
```

## Sub-Agent Delegation

Use specialized sub-agents for complex tasks:

| Situation | Delegate To | Why |
|-----------|-------------|-----|
| Complex kubectl debugging | `oracle` | Deep K8s analysis |
| Find code patterns | `explore` | Fast codebase search |
| External docs lookup | `librarian` | Documentation retrieval |

## Quality Requirements (STRICT)

### NEVER FORGE AHEAD

- **NEVER** mark `passes: true` if ANY criterion fails
- **NEVER** skip a criterion because "it's probably fine"
- **NEVER** move to the next story with unresolved failures
- **ALWAYS** fix failures before continuing

### Verification Standards

- Run EVERY verification command, not just spot checks
- Read FULL log output, not just summaries
- Check EVERY acceptance criterion explicitly
- Document BOTH successes and failures

### Failure Handling

1. **Document the failure** - exact error message, command output
2. **Diagnose** - read logs, check events, understand root cause
3. **Fix** - implement the actual fix, not a workaround
4. **Clean up** - delete failed pods/resources, reset state
5. **Rebuild** - if code changed: `cargo build --release`
6. **Re-verify** - run ALL criteria again from scratch

### Evidence Requirements

- Include full command output for each criterion
- Screenshot or log snippet for visual verification
- Kubernetes resource status before/after
- Clear PASS/FAIL status for each criterion

### Tool Usage Verification (MANDATORY AT END OF EACH STORY)

After completing each story that involves an agent CodeRun, you MUST verify that assigned tools were actually used:

```bash
# Get the CodeRun logs and search for tool invocations
kubectl logs -n cto -l app=coderun --all-containers | grep -E "(tool_call|mcp_|Tool:|Calling tool)"

# Check for specific tool invocations in agent logs
kubectl logs -n cto <coderun-pod> -c agent | grep -i "tool"

# Verify MCP server was started (if local tools configured)
kubectl logs -n cto <coderun-pod> -c mcp-server 2>/dev/null || echo "No MCP sidecar"
```

**What to look for in logs:**
- `tool_call` or `mcp_*` function invocations
- Tool names matching `cto-config.json` definitions
- MCP server startup messages
- Tool response handling

**If tools NOT being used:**
1. Check `cto-config.json` tool definitions
2. Verify tools passed to agent context
3. Check agent prompt includes tool instructions
4. Look for tool permission errors in logs
5. Document the gap and remediate before marking story complete

**Example tool usage evidence:**
```
✅ Tools Verified in Logs:
- mcp_firecrawl_scrape: 3 invocations
- mcp_context7_query-docs: 2 invocations
- grep: 15 invocations
- read_file: 42 invocations

❌ Tools NOT Used (configured but never called):
- mcp_github_create_pr: 0 invocations (investigate why!)
```

## Stop Condition

After completing a user story, check if ALL stories have `passes: true`.

If ALL stories are complete and passing, reply with:
<promise>COMPLETE</promise>

If there are still stories with `passes: false`, end your response normally (another iteration will pick up the next story).

## Important Rules (MUST FOLLOW)

1. **ONE story per iteration** - do not try to complete multiple stories
2. **VERIFY EVERY criterion** - do not skip any acceptance criteria
3. **NEVER forge ahead** - if ANY criterion fails, stop and fix it
4. **READ FULL LOGS** - use `kubectl logs` without `--tail` for full output
5. **CLEAN UP failures** - delete failed resources before retrying
6. **BUILD after code changes** - always run cargo build/test after edits
7. **DOCUMENT everything** - include full command output in progress.txt
8. **Reference the checklist** - `docs/workflow-lifecycle-checklist.md` has details

### Failure Recovery Flow

```
CRITERION FAILED
     │
     ▼
┌─────────────────────────────────────┐
│ 1. STOP - Do not continue           │
│ 2. READ full logs                   │
│ 3. DIAGNOSE root cause              │
│ 4. FIX the issue                    │
│ 5. CLEAN UP failed resources        │
│ 6. REBUILD if code changed          │
│ 7. RE-VERIFY from scratch           │
│ 8. REPEAT until ALL criteria pass   │
└─────────────────────────────────────┘
```
