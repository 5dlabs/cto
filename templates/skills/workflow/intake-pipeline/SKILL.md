---
name: intake-pipeline
description: Intake pipeline architecture, fan-out generation, tool discovery, and end-to-end testing.
agents: [morgan, atlas, tess]
triggers: [intake pipeline, fan-out, lobster workflow, task generation, pipeline testing, intake-util]
---

# Intake Pipeline

The intake pipeline transforms a PRD (Product Requirements Document) into a fully decomposed task breakdown with agent-specific prompts, documentation, tool manifests, and security remediation plans. It runs as a set of Lobster workflow YAML files orchestrated by `pipeline.lobster.yaml`.

## Pipeline Architecture

### Top-Level Orchestrator: `pipeline.lobster.yaml`

```
load-config ─┬─> setup-repo
             ├─> create-linear-project
             ├─> build-infra-context
             ├─> discover-tools          ← NEW: live MCP tool discovery
             ├─> codebase-analysis       (conditional: include_codebase=true)
             └─> deliberation            (conditional: deliberate=true)
                      │
                      ▼
                   intake               ← intake.lobster.yaml
```

**Phase 0 steps run in parallel** after `load-config`:
- `setup-repo` — create or validate GitHub repo, attach webhook
- `create-linear-project` — Linear project + PRD issue for visibility
- `build-infra-context` — kubectl queries for operators, ArgoCD apps, CRDs, services
- `discover-tools` — extract MCP tools from cto-config.json agents + kubectl cluster queries

**Config threading**: `cto-config.json` → `load-config` step → outputs flat key-value pairs (`primary_provider`, `primary_model`, `voter_1_provider`, etc.) → threaded to all downstream workflows.

### Intake Workflow: `intake.lobster.yaml`

```
parse-prd → analyze-complexity → review-tasks (approval gate)
                                        │
                                   refine-tasks (expand → vote → revise loop, max 2 rounds)
                                        │
                    ┌───────────────┬────┴──────────┬──────────────┐
              generate-scaffolds  search-skills  generate-scale-tasks  (parallel)
                    │                   │                  │
              fan-out-docs        discover-skills   generate-security-report
                    │                   │                  │
              validate-docs       generate-tool-manifest  generate-remediation-tasks
                    │                   │
                write-docs         fan-out-prompts
                                        │
                                  validate-prompts
                                        │
                                   write-prompts
                                        │
                              ┌─────────┴──────────┐
                        sync-linear-issues    commit-outputs
                                                    │
                                                create-pr
```

### Model Tiers

| Tier | Purpose | Default |
|------|---------|---------|
| `primary` | Core analysis (parse-prd, expand-tasks, complexity) | `claude-opus-4-6` |
| `fast` | Artifact generation (docs, prompts, scaffolds, tool manifest) | `claude-sonnet-4-6` |
| `frontier` | Critical decisions (deliberation, security report) | `claude-opus-4-6` |
| `committee` | 5-model voting panel for quality gating | Opus, GPT-5.2, Sonnet, o3-pro, Gemini 3.1 |

Configured in `cto-config.json` under `defaults.intake.models`.

### Allowed Models (guardrail)

`intake/config/openclaw-llm-task.json` lists all models the `llm-task` plugin will accept. Individual steps override with their own provider/model from config tiers. The list must include every model any step might request:

```
anthropic/claude-opus-4-6, anthropic/claude-sonnet-4-6, anthropic/claude-haiku-4-5
openai/gpt-5.2, openai/o3-pro
google/gemini-3.1-pro-preview, google-vertex/gemini-3.1-pro-preview
minimax/MiniMax-M2.5
bedrock/us.anthropic.claude-opus-4-6-v1:0, bedrock/us.anthropic.claude-sonnet-4-5-20250929-v1:0
```

## Fan-Out Generation

Docs and prompts are generated **per-task in parallel** rather than in a single LLM call, using bounded-concurrency fan-out.

### How It Works

1. `intake-util fan-out` receives an array of tasks on stdin
2. For each task, spawns `openclaw.invoke --tool llm-task --action json` as a subprocess
3. Semaphore limits concurrency (default 4 parallel)
4. Failed items retry twice with exponential backoff (1s, 2s)
5. Results merge into a single array; failures reported separately
6. `intake-util validate` checks completeness and structural integrity

### Fan-Out Flow

```
tasks JSON array (stdin)
        │
   fan-out.ts ─── Semaphore(4) ───┬── openclaw.invoke (task 1) → result
                                   ├── openclaw.invoke (task 2) → result
                                   ├── openclaw.invoke (task 3) → result
                                   └── openclaw.invoke (task 4) → result
                                              │
                                   merge results[] + failures[]
                                              │
                                   validate (completeness + structure)
                                              │
                                   write-files (to .tasks/docs/)
```

### Single-Task Input Mode

The system prompts (`smart-docs-system.md`, `smart-prompts-system.md`) support two modes:

- **Batch mode**: `expanded_tasks` array → output wrapped in `task_docs[]` / `task_prompts[]`
- **Single-task mode** (fan-out): `task` single object → output is a flat object (not array-wrapped)

Fan-out always uses single-task mode. The per-item schemas are:
- `intake/schemas/smart-doc-item.schema.json` — `{task_id, task_md, decisions_md, acceptance_md}`
- `intake/schemas/smart-prompt-item.schema.json` — `{task_id, prompt_md, prompt_xml, subtasks[]}`

### Validation Checks

`intake-util validate --type <docs|prompts> --task-ids <json>`

**Docs validation:**
- All expected task_ids present in output
- `task_md`, `decisions_md`, `acceptance_md` non-empty for every task

**Prompts validation:**
- All expected task_ids present in output
- `prompt_md`, `prompt_xml` non-empty for every task
- Each subtask has `subtask_id` and non-empty `prompt_md`

## Live Tool Discovery

Instead of a static YAML catalog, MCP tools are discovered dynamically from two sources:

### Source 1: cto-config.json Agent Tool Registry
Extracts unique tool server names from `agents.*.tools.remote` across all agents. These are agent-local MCP servers (npm packages on agent containers).

### Source 2: kubectl Cluster Queries
- `kubectl get svc -n operators -l 'app.kubernetes.io/component=mcp-server'` — labeled MCP services
- `kubectl get svc -n operators | grep -iE 'postgres|redis|mysql|mongo|nats'` — database/cache services

The `discover-tools` step in `pipeline.lobster.yaml` runs parallel with `build-infra-context` and produces a text inventory passed to `intake.lobster.yaml` as the `tool_context` input.

## Key Files

| File | Purpose |
|------|---------|
| `intake/workflows/pipeline.lobster.yaml` | Top-level orchestrator |
| `intake/workflows/intake.lobster.yaml` | Main intake pipeline |
| `intake/workflows/task-refinement.lobster.yaml` | Expand → vote → revise loop |
| `intake/workflows/voting.lobster.yaml` | 5-model committee voting |
| `intake/workflows/deliberation.lobster.yaml` | Optimist/Pessimist debate |
| `intake/workflows/codebase-analysis.lobster.yaml` | Repomix + LLM analysis |
| `cto-config.json` | Model tiers, committee, agent config |
| `intake/config/openclaw-llm-task.json` | LLM plugin config + allowedModels |
| `apps/intake-util/src/index.ts` | CLI entry point |
| `apps/intake-util/src/fan-out.ts` | Parallel LLM invocation |
| `apps/intake-util/src/validate.ts` | Output validation |
| `apps/intake-util/src/write-files.ts` | Disk writer |
| `apps/intake-util/src/tally.ts` | Vote tallying |
| `apps/intake-util/src/sync-linear.ts` | Linear issue creation |
| `intake/prompts/` | System prompts for each LLM step |
| `intake/schemas/` | JSON schemas for LLM outputs |

## Bridge Communication (NATS-Free)

The pipeline uses HTTP webhooks instead of NATS. Two bridges handle human-in-the-loop:

| Bridge | Port | Role | Network |
|--------|------|------|---------|
| `linear-bridge` | 3100 | Control plane — webhooks, run registry, Lobster resume, Linear Agent API | External via `agents.5dlabs.ai` (Cloudflare Tunnel) |
| `discord-bridge` | 3200 | Rendering surface — embeds, buttons, select menus | Internal only (ClusterIP) |

**Linear-bridge is the authority.** It receives all external webhooks (Linear Agent Session events), manages run state (RunRegistry), and triggers Lobster resume via durable resume tokens. Discord-bridge renders votes/decisions and forwards user interactions back to linear-bridge.

### Bridge URLs (Environment-Dependent)

| Environment | Linear Bridge URL | Discord Bridge URL |
|-------------|------|------|
| **Prod cluster** (main K8s) | `http://linear-bridge.bots.svc:3100` (internal) or `https://agents.5dlabs.ai` (external) | `http://discord-bridge.bots.svc:3200` |
| **Test EKS cluster** | `https://agents.5dlabs.ai` (via Cloudflare Tunnel — no local bridge needed) | N/A (called by linear-bridge, not by agents) |

Test agents on EKS reach linear-bridge through the Cloudflare Tunnel at `agents.5dlabs.ai`. No bridge deployment is needed on the test cluster — all webhook routing, run registration, and Lobster resume flow through the prod linear-bridge.

### Notification Flow

```
Workflow step → intake-util bridge-notify
  → POST linear-bridge/notify  (→ Linear comment / Agent Activity)
  → POST discord-bridge/notify (→ Discord embed in allocated room)

Elicitation → intake-util bridge-elicitation
  → POST linear-bridge/elicitation   (→ Linear select signal)
  → POST discord-bridge/elicitation  (→ Discord embed + buttons)
  → Lobster approval gate (pause)

Human responds on EITHER platform:
  Linear:  webhook → linear-bridge → Lobster resume
  Discord: interaction → discord-bridge → POST linear-bridge/runs/{id}/callback → Lobster resume

Cross-cancel: whichever platform answers first cancels the other.
```

### Key Bridge Files

| File | Purpose |
|------|---------|
| `apps/linear-bridge/src/http-server.ts` | Full HTTP router — webhooks, run registry, elicitation |
| `apps/linear-bridge/src/run-registry.ts` | Maps runId → {agent, sessionKey, resumeToken} |
| `apps/linear-bridge/src/agent-session-manager.ts` | Maps deliberation sessions → Linear sessions |
| `apps/linear-bridge/src/elicitation-handler.ts` | Linear select signal + Lobster resume |
| `apps/discord-bridge/src/http-server.ts` | Notification + elicitation HTTP API |
| `apps/discord-bridge/src/elicitation-handler.ts` | Discord embed rendering + interaction → callback |
| `infra/manifests/linear-bridge/tunnel-binding.yaml` | Cloudflare TunnelBinding for agents.5dlabs.ai |
| `docs/cloudflare-tunnel-intake-agent.md` | Tunnel setup instructions |

## intake-util CLI

```
intake-util <subcommand> [options]

Subcommands:
  write-files           --base-path <dir> --type <docs|prompts>
  tally                 --ballots-json <file>
  tally-decision-votes  (stdin: vote ballot array)
  parse-decision-points (stdin: debate turn JSON)
  fan-out               --prompt <path> --schema <path> --context <json> --provider <p> --model <m> [--concurrency <n>]
  validate              --type <docs|prompts|tasks|decision-vote|debate-turn|deliberation-result|tally|workflows> [--strict]
  sync-linear init      --project-name <n> --team-id <id> --prd-content <file>
  sync-linear issues    --project-id <id> --prd-issue-id <id> --team-id <id> --base-url <url>
  bridge-notify         --from <agent> --to <target> [--metadata <json>]
  bridge-elicitation    --session-id <id> --decision-id <id> --vote-result <json> --resume-token <token> --human-review-mode <mode>
  stack-validators      (stdin: scaffold JSON → validates stack/framework detection)
  generate-workflow     (stdin: expanded tasks → per-task implementation workflow YAML)
```

All subcommands read JSON from stdin when file path arguments are omitted.

## End-to-End Testing

### Prerequisites

- Kubernetes cluster access (`kubectl` configured)
- `openclaw` CLI installed and authenticated
- `intake-util` built: `cd apps/intake-util && bun run build`
- ArgoCD access (for `build-infra-context`)
- GitHub CLI (`gh`) authenticated
- Linear API key (for `sync-linear` steps, optional)

### Level 1: Unit Tests (No Cluster Required)

**Type check:**
```bash
cd apps/intake-util && npx tsc --noEmit
```

**Validate subcommand — success case:**
```bash
echo '[{"task_id":1,"task_md":"# T1","decisions_md":"# D1","acceptance_md":"# A1"},{"task_id":2,"task_md":"# T2","decisions_md":"# D2","acceptance_md":"# A2"}]' | \
  intake-util validate --type docs --task-ids '[1,2]'
# Should output: {"valid":true,"errors":[]}
```

**Validate subcommand — failure case (missing task):**
```bash
echo '[{"task_id":1,"task_md":"# T1","decisions_md":"# D1","acceptance_md":"# A1"}]' | \
  intake-util validate --type docs --task-ids '[1,2]'
# Should output: {"valid":false,"errors":["Missing docs for task_id 2"]}
# Exit code: 1
```

**Validate prompts — empty field detection:**
```bash
echo '[{"task_id":1,"prompt_md":"","prompt_xml":"<x/>"}]' | \
  intake-util validate --type prompts --task-ids '[1]'
# Should output errors about empty prompt_md
```

**Write-files — docs output:**
```bash
echo '{"task_docs":[{"task_id":1,"task_md":"# T1","decisions_md":"# D1","acceptance_md":"# A1"}]}' | \
  intake-util write-files --base-path /tmp/test-docs --type docs
ls /tmp/test-docs/task-1/
# Should contain: task.md, decisions.md, acceptance.md
```

**Tally — vote counting:**
```bash
echo '[{"voter":"v1","vote":"approve","confidence":0.9},{"voter":"v2","vote":"approve","confidence":0.8},{"voter":"v3","vote":"revise","confidence":0.7}]' | \
  intake-util tally
# Should output verdict with approve/revise counts
```

### Level 2: Fan-Out Integration (Requires openclaw)

**Mock fan-out with a simple schema:**
```bash
echo '[{"id":1,"title":"Test task 1"},{"id":2,"title":"Test task 2"}]' | \
  intake-util fan-out \
    --prompt "intake/prompts/smart-docs-system.md" \
    --schema "intake/schemas/smart-doc-item.schema.json" \
    --context '{"scaffolds":{},"codebase_context":"","infrastructure_context":""}' \
    --provider anthropic \
    --model claude-sonnet-4-6 \
    --concurrency 2
# Should produce array of 2 doc items
# Verify: each has task_id, task_md, decisions_md, acceptance_md
```

**Fan-out then validate pipeline:**
```bash
TASKS='[{"id":1,"title":"Auth service","agent":"rex"},{"id":2,"title":"Web dashboard","agent":"blaze"}]'
echo "$TASKS" | \
  intake-util fan-out \
    --prompt "intake/prompts/smart-docs-system.md" \
    --schema "intake/schemas/smart-doc-item.schema.json" \
    --context '{}' \
    --provider anthropic \
    --model claude-sonnet-4-6 \
    --concurrency 2 | \
  intake-util validate --type docs --task-ids '[1,2]'
# Should output: {"valid":true,"errors":[]}
```

### Level 3: Tool Discovery (Requires Cluster)

**Discover tools step in isolation:**
```bash
python3 -c "
import json, os
config_path = os.path.join(os.environ.get('WORKSPACE', '.'), 'cto-config.json')
with open(config_path) as f:
  c = json.load(f)
tools = set()
for agent_name, agent_cfg in c.get('agents', {}).items():
  for t in agent_cfg.get('tools', {}).get('remote', []):
    server = t.split('_')[0]
    tools.add(server)
for t in sorted(tools):
  print(f'- {t}')
"
# Should list: ai, argocd, context7, firecrawl, github, grafana, kubernetes, linear, loki, nano, octocode, openmemory, prometheus, repomix, shadcn, terraform
```

**Cluster MCP service query:**
```bash
kubectl get svc -n operators -l 'app.kubernetes.io/component=mcp-server' -o jsonpath='{range .items[*]}- {.metadata.name}{"\n"}{end}' 2>/dev/null || echo "(no cluster access)"
```

**Cluster database/cache services:**
```bash
kubectl get svc -n operators --no-headers 2>/dev/null | grep -iE 'postgres|redis|mysql|mongo|nats' | awk '{print "- " $1}'
```

### Level 4: Workflow Validation (Requires Lobster)

**Validate workflow YAML syntax:**
```bash
lobster validate intake/workflows/pipeline.lobster.yaml
lobster validate intake/workflows/intake.lobster.yaml
lobster validate intake/workflows/task-refinement.lobster.yaml
lobster validate intake/workflows/voting.lobster.yaml
```

**Dry-run with test PRD:**
```bash
lobster run intake/workflows/pipeline.lobster.yaml \
  --input prd_content="$(cat tests/intake/sample-prd.md)" \
  --input project_name="test-e2e" \
  --input num_tasks=3 \
  --input deliberate=false \
  --input include_codebase=false \
  --dry-run
```

### Level 5: Full Pipeline (Requires Everything)

**End-to-end with a real PRD:**
```bash
lobster run intake/workflows/pipeline.lobster.yaml \
  --input prd_content="$(cat path/to/your-prd.md)" \
  --input project_name="my-project" \
  --input num_tasks=10 \
  --input deliberate=true \
  --input include_codebase=false \
  --input pr_base_branch=main
```

**Verify outputs:**
```bash
# Check generated files
ls .tasks/docs/task-*/
# Each task-N/ should have: task.md, decisions.md, acceptance.md, prompt.md, prompt.xml

# Check PR was created
gh pr list --head "intake/my-project-*"

# Check Linear issues (if configured)
# Linear project should have task + subtask issues
```

### Level 6: Regression Checks

After any pipeline change, verify these invariants:

1. **Fan-out concurrency**: With `--concurrency 2` and 4 tasks, at most 2 openclaw.invoke processes run simultaneously
2. **Retry behavior**: Kill an openclaw.invoke subprocess mid-run; fan-out should retry up to 2 times
3. **Validation catches missing tasks**: Remove one result from fan-out output; validate should fail with specific task_id
4. **Model guardrail**: Request a model not in `allowedModels`; openclaw.invoke should reject it
5. **Tool discovery fallback**: With no cluster access, discover-tools should still produce agent tool registry from config
6. **Vote-gated refinement**: Committee must approve before docs/prompts generation begins
7. **Write-files idempotency**: Running write-files twice with same input produces identical file structure

## Red Flags - STOP

- Sending all tasks to a single LLM call instead of using fan-out — this defeats the parallelism
- Hardcoding model names in workflow YAML instead of threading from `cto-config.json` via `load-config`
- Adding models to workflow steps without adding them to `openclaw-llm-task.json` allowedModels
- Referencing the deleted static `intake/catalogs/tool-catalog.yaml` — use `discover-tools` output
- Skipping the `validate-docs` / `validate-prompts` steps — silent failures will cascade
- Using batch mode schemas (`smart-docs.schema.json`) with fan-out — use the per-item schemas (`smart-doc-item.schema.json`)
