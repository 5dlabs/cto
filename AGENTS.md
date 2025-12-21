# Repository Guidelines

## Git Workflow

| Setting | Value | Notes |
|---------|-------|-------|
| **Base Branch** | `develop` | All PRs should target `develop`, not `main` |
| **Release Branch** | `main` | Protected, releases only |
| **Feature Branches** | `feat/<name>` | Branch from `develop` |
| **Bugfix Branches** | `fix/<name>` | Branch from `develop` |

**Important:** Always create new branches from `develop`:
```bash
git checkout develop
git pull origin develop
git checkout -b feat/my-feature
```

## Play Workflow Overview

The CTO platform uses a multi-agent orchestration system called **Play** to take PRDs to shipped features.

**📖 Interactive Documentation:** See [docs/play-workflow-guide.html](docs/play-workflow-guide.html) for the complete flow diagram and acceptance criteria.

### Workflow Flow

```
PRD → Intake (Morgan) → Infrastructure (Bolt) → Implementation (Rex/Blaze) → Quality (Cleo) → Security (Cipher) → Testing (Tess) → Integration (Atlas) → Merged
```

**Note:** Bolt is always Task 1 - sets up databases, caches, and storage before implementation begins.

### Key MCP Tools

| Tool | Purpose |
|------|---------|
| `intake` | Process PRD to generate tasks with dependencies |
| `play` | Submit multi-agent workflow for a task |
| `play_status` | Query workflow progress and status |
| `jobs` | List running Argo workflows |
| `stop_job` | Stop a running workflow |

### Agents & Responsibilities

**Implementation Agents (Backend):**
| Agent | Language | Keywords | Default Stack |
|-------|----------|----------|---------------|
| **Rex** | Rust | rust, backend, api | axum, tokio, serde, sqlx |
| **Grizz** | Go | go, backend, api | chi, grpc, pgx, redis |
| **Nova** | Node.js/Bun | node, bun, backend, api | Elysia (or Hono, NestJS) |

**Implementation Agents (Frontend):**
| Agent | Language | Keywords | Default Stack |
|-------|----------|----------|---------------|
| **Blaze** | React/TS | react, frontend, ui | shadcn (Next.js 15, shadcn/ui, TailwindCSS) |
| **Tap** | Expo | expo, mobile, app | expo-router, react-native |
| **Spark** | Electron | electron, desktop | electron-builder, react |

**Support Agents:**
| Agent | Role | Keywords |
|-------|------|----------|
| **Cleo** | Quality assurance | quality, review, lint |
| **Cipher** | Security analysis | security, auth, vulnerabilities |
| **Tess** | Testing | test, qa, coverage |
| **Atlas** | Integration | merge, conflict, ci |
| **Bolt** | DevOps & Infrastructure | deploy, k8s, infra |
| **Morgan** | Project management | prd, intake, tasks |

**Conditional Prompts for Support Agents:**
Cleo, Cipher, and Tess receive **language-specific context** based on which implementation agent's work they're reviewing:

| Reviewing | Cleo Context | Cipher Context | Tess Context |
|-----------|--------------|----------------|--------------|
| Rex (Rust) | clippy, rustfmt, cargo | memory safety, unsafe blocks | cargo test, proptest |
| Grizz (Go) | golangci-lint, gofmt | goroutine safety, input validation | go test, testify |
| Nova (Node) | eslint, prettier | npm audit, injection | jest, supertest |
| Blaze (React) | eslint, biome | XSS, CSP, CORS | vitest, playwright |
| Tap (Expo) | eslint, expo-lint | secure storage, deep links | jest, detox |
| Spark (Electron) | eslint, electron-lint | nodeIntegration, contextIsolation | spectron, jest |

All support agents share the **same MCP servers and credentials** - only the prompt context differs.

### Full Agent Test Coverage

To test all agents, a PRD should include:

**Backend Microservices:**
- Rust service (Rex) - API with tokio/axum
- Go service (Grizz) - Microservice with chi/grpc
- Node service (Nova) - API with nestjs/express

**Frontend Applications:**
- React web app (Blaze) - Next.js with shadcn/tailwind
- Mobile app (Tap) - Expo with react-native
- Desktop app (Spark) - Electron with react

**Typical Task Distribution:**
```
Task 1: Bolt - Set up PostgreSQL, Redis, S3
Task 2-4: Rex/Grizz/Nova - Backend services (parallel)
Task 5-7: Blaze/Tap/Spark - Frontend apps (parallel)
Task 8-10: Cleo reviews each implementation
Task 11-13: Cipher audits each implementation
Task 14-16: Tess tests each implementation
Task 17: Atlas integrates all PRs
```

This architecture exercises **all 12 agents** across the workflow.

### Play Workflow Stages

Each task goes through multiple stages with dedicated agents. The workflow suspends between stages for human review if needed.

| Stage | Agent | Purpose | Timeout |
|-------|-------|---------|---------|
| **Pending** | - | Workflow initialized, ready to start | - |
| **Infrastructure** | Bolt | Task 1: Provision databases, caches, storage using operators | 15 min |
| **Implementation** | Rex / Blaze / etc. | Write code, create PR | 30 min |
| **Quality** | Cleo | Code review, style enforcement | 30 min |
| **Security** | Cipher | Security scan, vulnerability check | 30 min |
| **Testing** | Tess | Write tests, validate coverage | 30 min |
| **Integration** | Atlas | Resolve conflicts, prepare merge | 30 min |
| **Merged** | - | PR merged to main | - |

### Acceptance Criteria (Per Stage)

1. **Intake**: Tasks are atomic, dependencies correct, acceptance criteria defined
2. **Implementation**: Code compiles, tests pass, PR created
3. **Quality**: Style consistent, no code smells, documented
4. **Security**: No vulnerabilities, input validated, auth correct
5. **Testing**: Unit + integration tests, edge cases covered
6. **Integration**: No conflicts, CI passes, PR merged

### End-to-End Workflow Criteria

For a play workflow to be considered successful:

- [ ] **Workflow Starts** - Play workflow is submitted and reaches "Running" status
- [ ] **Task Discovery** - Tasks are loaded from docs repository successfully
- [ ] **Implementation** - Rex/Blaze creates working code and opens a PR
- [ ] **Quality** - Cleo reviews and code passes quality checks
- [ ] **Security** - Cipher scans and no critical vulnerabilities found
- [ ] **Testing** - Tess writes tests and all tests pass
- [ ] **Integration** - Atlas merges PR without conflicts
- [ ] **Completion** - Workflow reaches "Completed" status

### Per-Stage Acceptance Criteria

**Implementation Stage (Rex/Blaze):**
- CodeRun resource is created successfully
- Agent pod starts and reaches Running state
- GitHub authentication works (token generated from App credentials)
- Repository is cloned successfully
- Task files are loaded and parsed
- CLI (claude/cursor/code) executes without crash
- Code changes are committed to feature branch
- Pull request is created with proper description
- CI checks pass on the PR

**Quality Stage (Cleo):**
- PR code is reviewed for style and conventions
- Code complexity is within acceptable bounds
- No obvious code duplication detected
- Documentation is present where needed
- Suggested improvements are applied or documented

**Security Stage (Cipher):**
- Security scan completes without errors
- No critical or high vulnerabilities in code
- No vulnerable dependencies introduced
- Input validation is present for user data
- Authentication/authorization properly implemented

**Testing Stage (Tess):**
- Unit tests added for new functions
- Integration tests for API endpoints (if applicable)
- Edge cases and error paths tested
- Test coverage meets threshold (if configured)
- All existing tests still pass

**Integration Stage (Atlas):**
- Branch is rebased on latest main
- Merge conflicts (if any) are resolved correctly
- All CI checks pass after rebase
- PR is approved (or auto-merge enabled)
- PR is merged to main branch

### Intake via Linear Agent

Intake **always happens via an AI agent** triggered from Linear:

1. **Create Linear Issue** - Add label `prd` or `intake`, include PRD in description
2. **Attach Supporting Docs** - Architecture diagrams, API specs, design docs, reference links
3. **Morgan Reads & Researches** - Uses Firecrawl (URL scraping) and Context7 (tech docs)
4. **Generate Task Files** - Three files per task: `prompt.xml`, `prompt.md`, `acceptance_criteria.md`
5. **Project Created** - Linear project with Board view showing play phases
6. **Task Issues Created** - Each task becomes a child issue of the PRD

**Morgan's Tools:**
- **Firecrawl** - Scrapes URLs in PRD for additional context
- **Context7** - Looks up best practices for specified tech stack (Rust, React, etc.)

**Output Files (per task):**
| File | Purpose |
|------|---------|
| `prompt.xml` | Structured XML prompt (more complete solutions) |
| `prompt.md` | Markdown prompt for CLIs |
| `acceptance_criteria.md` | Checklist for task completion |

**Task 1 is always Bolt** - Sets up infrastructure based on PRD requirements using available operators.

**Infrastructure Operators (for Bolt):**
| Component | Operator | CRD Kind | Use Case |
|-----------|----------|----------|----------|
| PostgreSQL | CloudNative-PG | `Cluster` | Relational data, ACID |
| Redis/Valkey | Redis Operator | `Redis` | Caching, sessions |
| S3 Storage | SeaweedFS | N/A (Helm) | File uploads, artifacts |
| Kafka | Strimzi | `Kafka` | Event streaming |
| MongoDB | Percona | `PerconaServerMongoDB` | Document storage |
| MySQL | Percona | `PerconaXtraDBCluster` | MySQL HA clusters |
| RabbitMQ | RabbitMQ Operator | `RabbitmqCluster` | Message queues, AMQP |
| NATS | NATS Helm | N/A (Helm) | Lightweight messaging, JetStream |
| OpenSearch | OpenSearch Operator | `OpenSearchCluster` | Full-text search |
| ScyllaDB | ScyllaDB Operator | `ScyllaCluster` | Cassandra-compatible |
| QuestDB | QuestDB Operator | `QuestDB` | Time-series data |
| ClickHouse | ClickHouse Operator | `ClickHouseInstallation` | OLAP analytics |
| Temporal | Temporal Operator | `TemporalCluster` | Durable workflows |

**Credential Handoff:** Bolt creates `{{service}}-infra-config` ConfigMap. Other agents read connection strings from it.

**Order of Operations (`execution-levels.json`):**
Tasks are grouped into **execution levels** based on dependencies. Tasks within the same level run in **parallel**:
```json
{
  "levels": [["1", "2"], ["3"], ["4", "5", "6"], ["7"]],
  "stats": {"total_tasks": 7, "total_levels": 4, "max_parallelism": 3}
}
```
- **Level 0**: Tasks with no dependencies (start immediately)
- **Level N**: Tasks whose dependencies are all completed in levels < N
- Integration happens **level-by-level** (all Level 0 PRs merged before Level 1)

Enable parallel execution: `play({ parallel_execution: true })`

**CLI & Model Selection:**
PRD can specify which CLIs to use. Morgan populates `cto-config.json`:

| CLI | Primary Model | Thinking Model | Use Case |
|-----|--------------|----------------|----------|
| **claude** | claude-opus-4-5-20250929 | claude-opus-4-5-20250929 | Planning, complex reasoning |
| **code** | gpt-5.1 | o3, o4-mini | Multi-agent consensus, full-auto |
| **gemini** | gemini-2.5-pro | gemini-2.5-pro (thinking enabled) | Reasoning, multi-step problems |
| **opencode** | claude-opus-4-5-20250929 | claude-opus-4-5-20250929 | Provider-agnostic |

**Every Code Multi-Agent Modes:**
- `/plan` - Claude + Gemini + GPT-5 consensus for planning
- `/solve` - Fastest-first race for problem solving
- `/code` - Multi-worktree implementation with consensus
- `/auto` - Auto Drive for multi-step task orchestration

**Model Rotation (when acceptance criteria not met):**
```
claude (opus-4-5) → code (gpt-5.1) → gemini (2.5-pro) → code (/solve) → gemini (2.5-flash)
```

**Thinking Models Policy:**
- **Always prefer thinking models** when available for complex tasks
- Claude: Opus 4/5 has built-in extended thinking
- Code: Use `o3` or `o4-mini` for reasoning-heavy tasks
- Gemini: Enable `thinking` mode with 2.5 Pro

**Model API Identifiers:**
| Provider | Model | API Identifier | Notes |
|----------|-------|----------------|-------|
| Anthropic | Opus 4.5 | `claude-opus-4-5-20250929` | Extended thinking built-in |
| Anthropic | Sonnet 4 | `claude-sonnet-4-20250514` | Fast, capable |
| OpenAI | GPT-5.1 | `gpt-5.1` | Every Code default |
| OpenAI | GPT-5.2 | `gpt-5.2` | Latest coding model |
| OpenAI | o3 | `o3` | Reasoning model |
| OpenAI | o4-mini | `o4-mini` | Fast reasoning |
| Google | Gemini 2.5 Pro | `gemini-2.5-pro` | Thinking mode available |
| Google | Gemini 2.5 Flash | `gemini-2.5-flash` | Fast inference |

**CLI Rotation & Acceptance Probing:**
- Rotate CLIs **only if acceptance criteria not met**
- Each iteration uses a different CLI from the rotation list
- `max_retries` is the maximum, not guaranteed iterations
- If max reached without meeting criteria, **still complete successfully** (tunable)

**Linear Features Leveraged:**
- **Agent Activities** - Thoughts, actions, errors displayed in issue
- **Plan Checklist** - Progress steps shown in agent session
- **Projects** - Auto-created with workflow phases
- **Workflow States** - Ready → Implementation → Quality → Integration → Done
- **Agent Guidance** - Team-level instructions for agent behavior

**Interactive CLI Output:**
The agent's CLI stdout streams as activities. Watch the agent work in real-time in the Linear issue. The external URL links to Argo workflow for full logs.

**Event Flow (Intake → Play):**
1. **Pre-Intake**: GitHub workflow ensures repo access, adds webhook to Cloudflare tunnel
2. **Intake Creates**: Linear project + task issues + docs branch with task files
3. **Human Review**: Edit issues in Linear → changes sync to docs branch (one-way)
4. **Merge Trigger**: Human merges docs branch → webhook triggers play workflow
5. **Play Starts**: Workflow processes tasks per `execution-levels.json`

**Play does NOT start automatically** - human must merge the docs branch to trigger it.

### Using MCP Tool: `intake`

```json
{
  "name": "intake",
  "arguments": {
    "project_name": "my-project",
    "num_tasks": 15,
    "expand": true,
    "analyze": true,
    "enrich_context": true
  }
}
```

**Parameters:**
- `project_name` - Required. Directory containing prd.md
- `num_tasks` - Target number of tasks (default: 15)
- `expand` - Expand complex tasks into subtasks (default: true)
- `analyze` - Analyze task complexity (default: true)
- `enrich_context` - Use Firecrawl to scrape URLs in PRD (default: true)
- `local` - Run locally without Argo (default: false)

### Intake Output Structure

```
project-name/
├── prd.md                    # Input PRD
├── architecture.md           # Optional architecture doc
├── cto-config.json           # Generated agent config (stored in repo)
└── .tasks/
    ├── tasks.json            # All tasks with dependencies
    ├── execution-levels.json # Parallel execution order
    └── tasks/
        ├── task-1/
        │   ├── prompt.xml            # Structured XML prompt
        │   ├── prompt.md             # Markdown prompt
        │   └── acceptance_criteria.md
        ├── task-2/
        │   └── ...
        └── ...
```

### Using MCP Tool: `play`

```json
{
  "name": "play",
  "arguments": {
    "task_id": 1,
    "repository": "5dlabs/my-project",
    "service": "my-service",
    "docs_repository": "5dlabs/my-project",
    "docs_project_directory": "my-project",
    "parallel_execution": true
  }
}
```

**Key Parameters:**
- `task_id` - Task to implement (auto-detects if not provided)
- `repository` - GitHub repo for code
- `service` - Service identifier for workspace persistence
- `parallel_execution` - Run independent tasks in parallel

### Using MCP Tool: `play_status`

```json
{
  "name": "play_status",
  "arguments": {
    "repository": "5dlabs/my-project"
  }
}
```

Returns current workflow status, active tasks, and blocked tasks.

### Parallel vs Sequential Execution

| Mode | Use Case | Parameter |
|------|----------|-----------|
| **Sequential** | Default - one task at a time, simpler debugging | `parallel_execution: false` |
| **Parallel** | Faster PRs - run independent tasks simultaneously | `parallel_execution: true` |

### CLI Progress Matrix

Track which CLIs have been tested for each agent workflow:

| Agent | claude | code | gemini | opencode | cursor | dexter |
|-------|--------|------|--------|----------|--------|--------|
| **Morgan (Intake)** | ✅ Primary | ⬜ Planned | ⬜ Planned | ⬜ Planned | ⬜ Planned | ⬜ Planned |
| **Rex (Backend)** | ✅ Primary | ⬜ Planned | ⬜ Planned | ⬜ Planned | ⬜ Planned | ⬜ Planned |
| **Blaze (Frontend)** | ✅ Primary | ⬜ Planned | ⬜ Planned | ⬜ Planned | ⬜ Planned | ⬜ Planned |
| **Grizz (Go)** | ✅ Primary | ⬜ Planned | ⬜ Planned | ⬜ Planned | ⬜ Planned | ⬜ Planned |
| **Nova (Node)** | ✅ Primary | ⬜ Planned | ⬜ Planned | ⬜ Planned | ⬜ Planned | ⬜ Planned |
| **Tap (Expo)** | ✅ Primary | ⬜ Planned | ⬜ Planned | ⬜ Planned | ⬜ Planned | ⬜ Planned |
| **Spark (Electron)** | ✅ Primary | ⬜ Planned | ⬜ Planned | ⬜ Planned | ⬜ Planned | ⬜ Planned |
| **Cleo (Quality)** | ✅ Primary | ⬜ Planned | ⬜ Planned | ⬜ Planned | - | - |
| **Cipher (Security)** | ✅ Primary | ⬜ Planned | ⬜ Planned | ⬜ Planned | - | - |
| **Tess (Testing)** | ✅ Primary | ⬜ Planned | ⬜ Planned | ⬜ Planned | - | - |
| **Atlas (Integration)** | ✅ Primary | ⬜ Planned | ⬜ Planned | ⬜ Planned | - | - |
| **Bolt (Infra)** | ✅ Primary | ⬜ Planned | ⬜ Planned | ⬜ Planned | - | - |

**Legend:** ✅ Tested & Working | 🔄 In Progress | ⬜ Planned | ❌ Issues

### Token Usage & Cost Tracking

For CLIs that support it, track token usage and estimated cost per PR:

| Metric | Where | Description |
|--------|-------|-------------|
| Input Tokens | PR + Linear | Tokens sent to model |
| Output Tokens | PR + Linear | Tokens generated |
| Cached Tokens | PR + Linear | Tokens from cache (reduced cost) |
| Estimated Cost | PR + Linear | USD based on model pricing |
| Iterations | PR + Linear | CLI rotation attempts |

**Example PR Footer:**
```
### 📊 Cost Summary
- **Tokens**: 15,420 input / 3,890 output (1,200 cached)
- **Estimated Cost**: $0.0847
- **Iterations**: 2 (claude → code)
- **Duration**: 4m 32s
```

### Agent Tools: Linear + GitHub Sync

Each agent has access to **Linear MCP tools** to update task issues:
- **Linear Activities** - CLI stdout streams as thoughts/actions/errors
- **Linear Issue Updates** - Status, labels, comments via MCP
- **GitHub-Linear Sync** - Comments sync both ways automatically!

**Linear-GitHub Two-Way Sync:** When GitHub Issues Sync is enabled:
- Update comments in **either** GitHub OR Linear - they sync
- Use `magic words` in PR title/description to auto-link issues
- Status flows: PR opened → In Progress, PR merged → Done

**Magic Words (closing):** `close, closes, fix, fixes, resolve, completes`
**Magic Words (non-closing):** `ref, refs, part of, related to, contributes to`

### Sidecar Architecture

Each agent CLI runs with a **status-sync sidecar** that provides full [Linear Agent API](https://linear.app/developers/agents) integration:

| Function | Description |
|----------|-------------|
| **CLI Output Logging** | Streams stdout to Fluent-bit → Loki |
| **Linear Activities** | Parses `stream-json` output → thoughts, actions, errors |
| **Agent Plans** | Visual checklist in Linear UI tracking task progress |
| **External URL** | Links Linear session to Argo workflow for full logs |
| **Input REST Endpoint** | HTTP POST `/input` → FIFO → CLI stdin |
| **Status Sync** | Monitors `/workspace/status.json` → PM service |

### Two-Way Communication Flow

```
Linear User Comment
        │
        ▼
┌───────────────┐     Poll API     ┌──────────────┐
│  PM Service   │ ◄───────────────►│  Linear API  │
│  (pm-svc)     │                  └──────────────┘
└───────┬───────┘
        │ kubectl exec / HTTP
        ▼
┌───────────────┐     FIFO         ┌──────────────┐
│   Sidecar     │ ─────────────────►│  Agent CLI   │
│ (status-sync) │                  │ (claude/etc) │
└───────────────┘                  └──────────────┘
        ▲                                 │
        │ stream-json                     │ stdout
        └─────────────────────────────────┘
```

### Log Collection Pipeline

```
Kubernetes Pods              Observability Stack
┌──────────────┐            ┌──────────────────┐
│   stdout     │            │   Fluent-Bit     │
│   stderr     │───────────►│   (DaemonSet)    │
│ /var/log/    │  tail      │  + K8s metadata  │
│ containers/  │            └────────┬─────────┘
└──────────────┘                     │ OTLP/HTTP
                                     ▼
                            ┌──────────────────┐
                            │  OTEL Collector  │
                            │  batch, enrich   │
                            └────────┬─────────┘
                                     │ OTLP/HTTP
                                     ▼
                            ┌──────────────────┐
                            │      Loki        │
                            │  TSDB (7 days)   │
                            └────────┬─────────┘
                                     │ LogQL
                                     ▼
                            ┌──────────────────┐
                            │     Grafana      │
                            │    (UI + API)    │
                            └──────────────────┘
```

**Linear Agent API Activity Types:**
| Type | Use Case | Example |
|------|----------|---------|
| `thought` | Internal reasoning | "Analyzing the codebase structure..." |
| `action` | Tool invocation | `read_file("src/main.rs")` |
| `action` (with result) | Tool completion | `read_file` → "✅ 245 lines" |
| `error` | Failures | "❌ Build failed: missing dependency" |
| `response` | Final completion | "Completed | 45.2s | $0.0234 | 12 turns" |
| `ephemeral` | Transient status | Replaced by next activity (status updates) |

**Agent Plan States:**
| Status | Display |
|--------|---------|
| `pending` | ○ Not started |
| `inProgress` | ● Working |
| `completed` | ✓ Done |
| `canceled` | ✗ Skipped |

**Session External URL:** Links to Argo workflow UI for full logs and debugging. Prevents Linear from marking session as unresponsive.

**Applies to:** Intake (Morgan), Play tasks (Rex/Blaze/etc), Healer remediation jobs

**Two-way communication:** Linear comments → PM service polls → kubectl exec to sidecar → FIFO → CLI stdin

**Environment Variables for Sidecar:**
| Variable | Description |
|----------|-------------|
| `LINEAR_SESSION_ID` | Agent session ID |
| `LINEAR_OAUTH_TOKEN` | OAuth token for API access |
| `ARGO_WORKFLOW_URL` | URL to Argo workflow (set as external URL) |
| `TASK_ID` | Current task ID |
| `TASK_DESCRIPTION` | Task description for plan |
| `CLAUDE_STREAM_FILE` | Path to stream-json output (`/workspace/claude-stream.jsonl`) |

**Linear Developer Docs:**
- [Agent Interaction Guidelines (AIG)](https://linear.app/developers/aig)
- [Getting Started with Agents](https://linear.app/developers/agents)
- [Developing Agent Interactions](https://linear.app/developers/agent-interaction)
- [Interaction Best Practices](https://linear.app/developers/agent-best-practices)
- [Signals](https://linear.app/developers/agent-signals)

---

## Platform Infrastructure Inventory

Complete inventory of deployed infrastructure, operators, and services.

### Storage
| Component | Type | Description |
|-----------|------|-------------|
| Mayastor | Block Storage | High-performance NVMe storage class (`mayastor`) |
| SeaweedFS | Object Storage | S3-compatible storage with filer |

### AI/ML Infrastructure
| Component | Description | Models |
|-----------|-------------|--------|
| KubeAI | OpenAI-compatible API | Llama 3.1, DeepSeek R1, Qwen 2.5 |
| Ollama Operator | Ollama model management | phi4, qwen2.5-coder, nomic-embed |
| LlamaStack Operator | Meta's LlamaStack | Starter, vLLM distributions |
| NVIDIA GPU Operator | GPU provisioning | L4, H100, A100, GH200 profiles |

### Observability Stack
| Component | Purpose | Service |
|-----------|---------|---------|
| Prometheus | Metrics & alerting | `prometheus-server:80` |
| Loki | Log aggregation | `loki-gateway:80` |
| Grafana | Visualization | `grafana:80` |
| Alertmanager | Alert routing (Discord) | `alertmanager:9093` |
| OTEL Collector | Telemetry ingestion | `:4317` (gRPC), `:4318` (HTTP) |
| Fluent-bit | Log collection | DaemonSet (all nodes) |
| Kube-state-metrics | K8s resource metrics | `kube-state-metrics:8080` |
| Blackbox Exporter | External endpoint probing | `blackbox-exporter:9115` |
| Metrics Server | HPA/VPA metrics | `metrics-server:443` |
| Jaeger Operator | Distributed tracing | Per-namespace instances |
| OpenTelemetry Operator | Auto-instrumentation | Instrumentation CRs |

### Log Collection Pipeline

Logs flow through a multi-stage pipeline from containers to Grafana:

```
Kubernetes Pods → Fluent-bit → OTEL Collector → Loki → Grafana
   (stdout)       (DaemonSet)    (OTLP/HTTP)   (TSDB)   (UI/API)
```

**Pipeline Details:**

| Stage | Component | Configuration |
|-------|-----------|---------------|
| **Collection** | Fluent-bit | Tails `/var/log/containers/*.log`, enriches with K8s metadata |
| **Processing** | OTEL Collector | Receives OTLP, batches, adds resource attributes |
| **Storage** | Loki | TSDB with 7-day retention, 20Gi storage on Mayastor |
| **Query** | Grafana | Loki datasource, LogQL queries |

**Fluent-bit Enrichment:**

Fluent-bit automatically adds service metadata based on namespace/pod patterns:
- `service.name` - Component identifier (e.g., `agent-controller`, `tools-mcp`)
- `service.component` - Functional category (e.g., `controller`, `tools`)
- `service.namespace` - Kubernetes namespace
- `detected_level` - Log level (error/warn/info) parsed from log content

**Log Retention:** 7 days (`retention_period: 168h`)

### Querying Logs via API

**Port-Forward Requirements:**
```bash
# Loki (direct API access)
kubectl port-forward svc/loki-gateway -n observability 3100:80

# Grafana (web UI and API)
kubectl port-forward svc/grafana -n observability 3000:80
```

**Loki API (LogQL):**
```bash
# Query logs directly from Loki
curl -G "http://localhost:3100/loki/api/v1/query_range" \
  --data-urlencode 'query={namespace="cto"}' \
  --data-urlencode 'start=1h' \
  --data-urlencode 'limit=100'

# Get available labels
curl "http://localhost:3100/loki/api/v1/labels"

# Get label values
curl "http://localhost:3100/loki/api/v1/label/namespace/values"
```

**Grafana API:**
```bash
# Query via Grafana (uses datasource proxy)
curl -u admin:admin "http://localhost:3000/api/datasources/proxy/uid/loki/loki/api/v1/query_range" \
  --data-urlencode 'query={namespace="cto"}'
```

**MCP Tools for Logs:**

Use Loki MCP tools for programmatic access:
```bash
# Query logs with filters
loki_query({
  query: "{namespace=\"cto\"} |~ \"error|ERROR\"",
  limit: 100
})

# Get available labels
loki_label_names()
loki_label_values({ label: "pod" })
```

**Common LogQL Queries:**

| Use Case | Query |
|----------|-------|
| All CTO namespace logs | `{namespace="cto"}` |
| Errors only | `{namespace="cto"} \|~ "error\|ERROR"` |
| Specific pod | `{namespace="cto", pod=~"controller.*"}` |
| Agent CLI output | `{namespace="cto", container="agent"}` |
| JSON parsed | `{namespace="cto"} \| json \| level="error"` |
| By service name | `{service_name="agent-controller"}` |
| Play workflow logs | `{namespace="cto"} \|~ "coderun\|play"` |
| MCP tool errors | `{namespace="cto"} \|~ "Tool.*not found\|routing.*failed"` |

**Verify Log Pipeline:**

```bash
# 1. Check Fluent-bit is collecting logs
kubectl logs -n observability -l app.kubernetes.io/name=fluent-bit --tail=10

# 2. Check OTEL Collector is receiving and exporting
kubectl logs -n observability -l app.kubernetes.io/name=opentelemetry-collector --tail=10

# 3. Verify Loki is receiving logs
kubectl port-forward svc/loki-gateway -n observability 3100:80 &
curl -s "http://localhost:3100/loki/api/v1/labels" | jq .

# 4. Query recent logs
curl -G "http://localhost:3100/loki/api/v1/query_range" \
  --data-urlencode 'query={namespace="cto"}' \
  --data-urlencode 'limit=5' | jq '.data.result[0].values[:3]'

# 5. Access Grafana UI
kubectl port-forward svc/grafana -n observability 3000:80 &
# Open http://localhost:3000 (admin/admin)
# Navigate to Explore → Select Loki datasource → Run LogQL query
```

### Platform Services
| Component | Purpose | Namespace |
|-----------|---------|-----------|
| ArgoCD | GitOps deployment | `argocd` |
| ArgoCD Image Updater | Auto image updates | `argocd` |
| Argo Workflows | Workflow orchestration | `automation` |
| Argo Events | Event-driven triggers | `automation` |
| External Secrets | Secret sync from Bao | `external-secrets` |
| OpenBao | Secrets management (Vault) | `openbao` |
| Cert-Manager | TLS certificates | `cert-manager` |
| External DNS | DNS record management | `external-dns` |
| Cloudflare Operator | Tunnel management | `operators` |
| Ingress NGINX | Ingress controller | `ingress-nginx` |
| Gateway API | Gateway resources (CRDs) | Cluster-wide |
| Cilium | CNI & network policies | `kube-system` |
| Kilo | WireGuard VPN mesh | `kube-system` |

### CI/CD & Runners
| Component | Purpose | Config |
|-----------|---------|--------|
| ARC Controller | GitHub Actions | `arc-systems` |
| k8s-runner | Self-hosted runners | 2-6 runners, DinD |
| BuildKit | Remote builds | Caching enabled |

### CTO Platform Services
| Service | Image | Purpose |
|---------|-------|---------|
| Controller | `ghcr.io/5dlabs/controller` | CodeRun orchestration |
| PM Server | `ghcr.io/5dlabs/pm-server` | Linear/GitHub integration |
| Tools Server | `ghcr.io/5dlabs/tools` | MCP tool proxy |
| Healer | `ghcr.io/5dlabs/healer` | Remediation & monitoring |
| OpenMemory | `ghcr.io/5dlabs/openmemory` | Agent memory store |
| TweakCN | `ghcr.io/5dlabs/tweakcn` | UI component generation |
| Research | `ghcr.io/5dlabs/research` | Bookmark research pipeline |

### Identity & Auth
| Component | Purpose | Status |
|-----------|---------|--------|
| Keycloak Operator | Identity provider | Available |

---

## Available MCP Tools in Cursor

The following MCP servers and tools are available in this Cursor workspace for testing and implementation:

### 1. Morph MCP (Code Editing & Search)

**Purpose:** Advanced code editing and semantic search capabilities

| Tool | Description | Use Case |
|------|-------------|----------|
| `mcp_morph-mcp_edit_file` | Fast, accurate file editing with placeholders | Edit files without reading entire contents |
| `mcp_morph-mcp_warpgrep_codebase_search` | AI-powered codebase search subagent | Find code by semantic meaning across repo |
| `mcp_morph-mcp_codebase_search` | Semantic code search with embeddings | Search by natural language queries |

**Best Practices:**
- Use `edit_file` for large files (>1000 lines) instead of read/write
- Use `warpgrep_codebase_search` for exploratory searches
- Use `codebase_search` when you know what you're looking for

### 2. Browser Automation (cursor-ide-browser)

**Purpose:** Browser testing and web automation

| Tool | Description | Use Case |
|------|-------------|----------|
| `mcp_cursor-ide-browser_browser_navigate` | Navigate to URL | Open web pages |
| `mcp_cursor-ide-browser_browser_snapshot` | Capture accessibility snapshot | Read page structure |
| `mcp_cursor-ide-browser_browser_click` | Click elements | Interact with UI |
| `mcp_cursor-ide-browser_browser_type` | Type text into inputs | Fill forms |
| `mcp_cursor-ide-browser_browser_take_screenshot` | Capture screenshots | Visual verification |
| `mcp_cursor-ide-browser_browser_console_messages` | Get console logs | Debug JavaScript |
| `mcp_cursor-ide-browser_browser_network_requests` | Get network activity | Debug API calls |

**Best Practices:**
- Always snapshot before interacting with elements
- Use screenshots for visual inspection
- Check console messages for JavaScript errors

### 3. Firecrawl MCP (Web Scraping)

**Purpose:** Web scraping and content extraction

| Tool | Description | Use Case |
|------|-------------|----------|
| `mcp_firecrawl-mcp_firecrawl_scrape` | Scrape single URL | Extract content from one page |
| `mcp_firecrawl-mcp_firecrawl_search` | Search web and scrape results | Find information across multiple sites |
| `mcp_firecrawl-mcp_firecrawl_map` | Map website URLs | Discover site structure |
| `mcp_firecrawl-mcp_firecrawl_crawl` | Crawl multiple pages | Extract content from entire site |
| `mcp_firecrawl-mcp_firecrawl_extract` | Extract structured data with LLM | Parse specific fields from pages |

**Best Practices:**
- Use `search` without formats first, then scrape specific URLs
- Set `maxAge` parameter for 500% faster cached scrapes
- Use `map` before `crawl` to understand site structure

### 4. OpenMemory (Agent Memory)

**Purpose:** Persistent long-term memory for agents

| Tool | Description | Use Case |
|------|-------------|----------|
| `mcp_tools_openmemory_openmemory_query` | Semantic search memories | Find relevant past experiences |
| `mcp_tools_openmemory_openmemory_store` | Store new memory | Save important information |
| `mcp_tools_openmemory_openmemory_list` | List recent memories | Browse memory history |
| `mcp_tools_openmemory_openmemory_get` | Get specific memory | Retrieve by ID |
| `mcp_tools_openmemory_openmemory_reinforce` | Boost memory salience | Mark important memories |

**Best Practices:**
- Store episodic memories for task history
- Store semantic memories for learned patterns
- Reinforce important memories to prevent decay

### 5. Context7 (Library Documentation)

**Purpose:** Up-to-date library documentation and best practices

| Tool | Description | Use Case |
|------|-------------|----------|
| `mcp_context7_resolve-library-id` | Resolve library name to ID | Find Context7-compatible library ID |
| `mcp_context7_get-library-docs` | Get library documentation | Fetch API docs and examples |

**Two-Step Workflow:**
1. **Resolve library:** `resolve_library_id({ libraryName: "tokio rust" })`
2. **Get docs:** `get_library_docs({ context7CompatibleLibraryID: "/websites/rs_tokio_tokio", topic: "error handling async" })`

**Key Rust Library IDs:**
- **Tokio:** `/websites/rs_tokio_tokio` (async runtime, 93.8 score)
- **Serde:** `/websites/serde_rs` (serialization)
- **Clippy:** `/rust-lang/rust-clippy` (lints)
- **Anyhow:** `/dtolnay/anyhow` (app errors, 89.3 score)
- **Thiserror:** `/dtolnay/thiserror` (custom errors, 83.1 score)
- **Tracing:** `/tokio-rs/tracing` (logging)

**Best Practices:**
- Always resolve library names first
- Query focused topics: "error handling context" not "documentation"
- Use `mode='code'` for API references, `mode='info'` for guides

### 6. shadcn/ui Components

**Purpose:** Access shadcn/ui v4 component source and demos

| Tool | Description | Use Case |
|------|-------------|----------|
| `mcp_shadcn_get_component` | Get component source code | Implement shadcn components |
| `mcp_shadcn_get_component_demo` | Get component demo code | See usage examples |
| `mcp_shadcn_list_components` | List all components | Discover available components |
| `mcp_shadcn_get_component_metadata` | Get component metadata | Check dependencies and props |

**Best Practices:**
- Use for Blaze (React/TypeScript) implementations
- Get both source and demo for complete understanding
- Check metadata for required dependencies

### 7. Talos MCP (Node Management)

**Purpose:** Manage Talos Kubernetes nodes

| Tool | Description | Use Case |
|------|-------------|----------|
| `mcp_talos-mcp_containers` | List containers | Check running workloads |
| `mcp_talos-mcp_stats` | Get resource usage | Monitor CPU/memory |
| `mcp_talos-mcp_get_logs` | Get service logs | Debug services |
| `mcp_talos-mcp_list` | List files | Browse filesystem |
| `mcp_talos-mcp_read` | Read file | View config files |
| `mcp_talos-mcp_get_health` | Check cluster health | Verify cluster status |

**Best Practices:**
- Use for infrastructure debugging
- Check logs when services fail
- Monitor resource usage for capacity planning

### 8. CTO Platform Tools

**Purpose:** Platform-specific workflow management

| Tool | Description | Use Case |
|------|-------------|----------|
| `mcp_cto_intake` | Process PRD to tasks | Generate task breakdown |
| `mcp_cto_play` | Submit multi-agent workflow | Execute task implementation |
| `mcp_cto_play_status` | Query workflow status | Check progress |
| `mcp_cto_jobs` | List running workflows | Monitor active jobs |
| `mcp_cto_stop_job` | Stop workflow | Cancel running job |
| `mcp_cto_input` | Send message to running job | Provide live feedback |

**Best Practices:**
- Use `intake` for PRD processing
- Use `play` with `parallel_execution: true` for speed
- Monitor with `play_status` and `jobs`

### 9. Filesystem & Knowledge Graph

**Purpose:** File operations and knowledge management

| Tool | Description | Use Case |
|------|-------------|----------|
| `mcp_tools_read_file` | Read file contents | View single file |
| `mcp_tools_read_multiple_files` | Read multiple files | Batch file reading |
| `mcp_tools_write_file` | Write file | Create/overwrite file |
| `mcp_tools_edit_file` | Edit file with line-based changes | Precise edits |
| `mcp_tools_list_directory` | List directory contents | Browse directories |
| `mcp_tools_directory_tree` | Get recursive tree | Understand structure |
| `mcp_tools_search_files` | Search by glob pattern | Find files by name |
| `mcp_tools_create_entities` | Create knowledge graph entities | Store structured knowledge |
| `mcp_tools_create_relations` | Create entity relations | Link knowledge |
| `mcp_tools_search_nodes` | Search knowledge graph | Query stored knowledge |

**Best Practices:**
- Use `read_multiple_files` for batch operations
- Use `edit_file` for precise line-based changes
- Use knowledge graph for structured information

### 10. Grafana MCP (Observability & Monitoring)

**Purpose:** Comprehensive observability for metrics, logs, traces, and alerts

**Prerequisites:** Requires port-forwards to cluster services:
```bash
kubectl port-forward svc/prometheus-server -n observability 9090:80
kubectl port-forward svc/loki-gateway -n observability 3100:80
kubectl port-forward svc/grafana -n observability 3000:80
```

| Tool Category | Tools | Use Case |
|---------------|-------|----------|
| **Dashboards** | `grafana_search_dashboards`, `grafana_get_dashboard`, `grafana_create_dashboard` | Visualize metrics and logs |
| **Prometheus** | `grafana_query_prometheus`, `grafana_get_prometheus_labels`, `grafana_get_prometheus_metadata` | Query metrics, check service health |
| **Loki** | `grafana_query_loki_logs`, `grafana_get_loki_labels`, `grafana_get_loki_label_values` | Search logs, debug issues |
| **Alerts** | `grafana_list_alert_rules`, `grafana_get_alert_rule` | Monitor alert status |
| **Datasources** | `grafana_list_datasources`, `grafana_get_datasource` | Check data source configuration |
| **Navigation** | `grafana_generate_explore_url` | Create deeplinks to Grafana UI |

**Best Practices:**
- Use Prometheus queries for service health checks: `up{job="cto-controller"}`
- Use Loki for debugging: `{namespace="cto"} |~ "error|ERROR"`
- Check dashboards for visual verification of deployments
- Use alert rules to verify monitoring is active

### 11. ArgoCD & Argo Workflows (GitOps & Orchestration)

**Purpose:** Manage GitOps deployments and workflow executions

**Prerequisites:** Port-forwards required:
```bash
kubectl port-forward svc/argocd-server -n argocd 8080:80
kubectl port-forward svc/argo-workflows-server -n automation 2746:2746
```

| Tool Category | Tools | Use Case |
|---------------|-------|----------|
| **ArgoCD Apps** | `argocd_list_applications`, `argocd_get_application`, `argocd_sync_application` | Verify deployments |
| **ArgoCD Resources** | `argocd_get_application_resource_tree`, `argocd_get_application_events` | Debug sync issues |
| **Workflows** | `argo_workflows_list_workflows`, `argo_workflows_get_workflow`, `argo_workflows_get_workflow_logs` | Monitor Play tasks |
| **Workflow Templates** | `argo_workflows_list_workflow_templates`, `argo_workflows_list_cron_workflows` | Check available templates |
| **Workflow Control** | `argo_workflows_retry_workflow` | Retry failed tasks |

**Best Practices:**
- Check ArgoCD sync status before and after deployments
- Monitor workflow logs for agent CLI output
- Use workflow status to track Play execution progress
- Check resource tree for deployment issues

### 12. MCP Add/Remove/Update Tools

**Purpose:** Manage MCP servers in the platform

| Tool | Description | Use Case |
|------|-------------|----------|
| `mcp_cto_add_mcp_server` | Add MCP server from GitHub | Install new MCP server |
| `mcp_cto_remove_mcp_server` | Remove MCP server | Uninstall MCP server |
| `mcp_cto_update_mcp_server` | Update MCP server config | Refresh server configuration |

**Best Practices:**
- Add servers by GitHub URL
- Use `skip_merge: true` for manual review
- Update servers when README changes

---

## CTO Lifecycle Verification with MCP Tools

This section describes how to use MCP tools to verify each stage of the CTO workflow from PRD to deployed feature.

### Phase 1: Pre-Intake Verification

**Goal:** Ensure platform is ready for intake

```bash
# 1. Check cluster health
mcp_talos-mcp_get_health({ control_planes: ["192.168.1.77"] })

# 2. Verify core services are up
grafana_query_prometheus({
  datasourceUid: "prometheus",
  expr: "up{job=~\"cto-.*\"}",
  queryType: "instant"
})

# 3. Check ArgoCD sync status
argocd_list_applications()
argocd_get_application({ applicationName: "cto-controller" })
argocd_get_application({ applicationName: "cto-tools" })
argocd_get_application({ applicationName: "cto-pm-server" })

# 4. Verify no stuck workflows
argo_workflows_list_workflows({ 
  namespace: "cto", 
  status: "Running",
  limit: 10 
})

# 5. Check for errors in logs
grafana_query_loki_logs({
  datasourceUid: "loki",
  logql: "{namespace=\"cto\"} |~ \"error|ERROR\" | json",
  limit: 20
})
```

### Phase 2: Intake Monitoring

**Goal:** Verify intake workflow is processing correctly

```bash
# 1. Find intake workflow
argo_workflows_list_workflows({
  namespace: "cto",
  status: "Running",
  limit: 5
})

# 2. Monitor workflow progress
argo_workflows_get_workflow({ 
  namespace: "cto", 
  name: "intake-workflow-name" 
})

# 3. Check Morgan's logs
grafana_query_loki_logs({
  datasourceUid: "loki",
  logql: "{namespace=\"cto\", container=\"agent\"} | json | task_id=\"intake\"",
  limit: 100
})

# 4. Monitor resource usage
grafana_query_prometheus({
  datasourceUid: "prometheus",
  expr: "container_memory_usage_bytes{namespace=\"cto\",pod=~\"intake.*\"}",
  queryType: "instant"
})

# 5. Verify Firecrawl usage (URL scraping)
grafana_query_loki_logs({
  datasourceUid: "loki",
  logql: "{namespace=\"cto\"} |~ \"firecrawl|scrape\"",
  limit: 50
})

# 6. Check Context7 lookups
grafana_query_loki_logs({
  datasourceUid: "loki",
  logql: "{namespace=\"cto\"} |~ \"context7|library\"",
  limit: 50
})
```

### Phase 3: Infrastructure Setup (Bolt)

**Goal:** Verify Task 1 infrastructure deployment

```bash
# 1. Monitor Bolt workflow
argo_workflows_get_workflow({ 
  namespace: "cto", 
  name: "play-task-1-bolt" 
})

# 2. Check for created databases/services
mcp_talos-mcp_containers({
  node: "192.168.1.77",
  kubernetes: true
})

# 3. Verify PostgreSQL operator
argocd_get_application({ applicationName: "cloudnative-pg" })

# 4. Check Redis operator
argocd_get_application({ applicationName: "redis-operator" })

# 5. Verify ConfigMap creation
grafana_query_loki_logs({
  datasourceUid: "loki",
  logql: "{namespace=\"cto\"} |~ \"infra-config|ConfigMap\"",
  limit: 20
})

# 6. Check operator logs
grafana_query_loki_logs({
  datasourceUid: "loki",
  logql: "{namespace=~\"postgres-operator|redis-operator\"} |~ \"error|ERROR\"",
  limit: 50
})
```

### Phase 4: Implementation (Rex/Blaze/etc)

**Goal:** Monitor agent implementations and PR creation

```bash
# 1. List active implementation workflows
argo_workflows_list_workflows({
  namespace: "cto",
  status: "Running",
  limit: 10
})

# 2. Monitor specific agent (e.g., Rex)
argo_workflows_get_workflow_logs({
  namespace: "cto",
  workflow_name: "play-task-2-rex"
})

# 3. Check agent CLI output
grafana_query_loki_logs({
  datasourceUid: "loki",
  logql: "{namespace=\"cto\", agent=\"rex\"} | json",
  limit: 200
})

# 4. Verify GitHub PR creation
grafana_query_loki_logs({
  datasourceUid: "loki",
  logql: "{namespace=\"cto\"} |~ \"pull_request|PR created\"",
  limit: 20
})

# 5. Check for build errors
grafana_query_loki_logs({
  datasourceUid: "loki",
  logql: "{namespace=\"cto\"} |~ \"build failed|compilation error|cargo.*error\"",
  limit: 50
})

# 6. Monitor resource usage per agent
grafana_query_prometheus({
  datasourceUid: "prometheus",
  expr: "container_cpu_usage_seconds_total{namespace=\"cto\",pod=~\"play-task.*\"}",
  queryType: "instant"
})
```

### Phase 5: Quality Review (Cleo)

**Goal:** Verify code quality checks pass

```bash
# 1. Monitor Cleo workflow
argo_workflows_get_workflow({ 
  namespace: "cto", 
  name: "play-task-8-cleo" 
})

# 2. Check linter output
grafana_query_loki_logs({
  datasourceUid: "loki",
  logql: "{namespace=\"cto\", agent=\"cleo\"} |~ \"clippy|rustfmt|eslint\"",
  limit: 100
})

# 3. Verify no critical issues
grafana_query_loki_logs({
  datasourceUid: "loki",
  logql: "{namespace=\"cto\", agent=\"cleo\"} |~ \"error|warning|WARN\"",
  limit: 50
})
```

### Phase 6: Security Audit (Cipher)

**Goal:** Verify security analysis completes

```bash
# 1. Monitor Cipher workflow
argo_workflows_get_workflow({ 
  namespace: "cto", 
  name: "play-task-11-cipher" 
})

# 2. Check for vulnerabilities
grafana_query_loki_logs({
  datasourceUid: "loki",
  logql: "{namespace=\"cto\", agent=\"cipher\"} |~ \"vulnerability|CVE|security\"",
  limit: 100
})

# 3. Verify audit passes
grafana_query_loki_logs({
  datasourceUid: "loki",
  logql: "{namespace=\"cto\", agent=\"cipher\"} |~ \"audit.*pass|security.*ok\"",
  limit: 20
})
```

### Phase 7: Testing (Tess)

**Goal:** Verify tests pass and coverage meets requirements

```bash
# 1. Monitor Tess workflow
argo_workflows_get_workflow({ 
  namespace: "cto", 
  name: "play-task-14-tess" 
})

# 2. Check test results
grafana_query_loki_logs({
  datasourceUid: "loki",
  logql: "{namespace=\"cto\", agent=\"tess\"} |~ \"test.*pass|test.*fail|cargo test\"",
  limit: 200
})

# 3. Verify coverage
grafana_query_loki_logs({
  datasourceUid: "loki",
  logql: "{namespace=\"cto\", agent=\"tess\"} |~ \"coverage|%\"",
  limit: 20
})

# 4. Check for test failures
grafana_query_loki_logs({
  datasourceUid: "loki",
  logql: "{namespace=\"cto\", agent=\"tess\"} |~ \"FAILED|test failed\"",
  limit: 50
})
```

### Phase 8: Integration (Atlas)

**Goal:** Verify PRs merge successfully without conflicts

```bash
# 1. Monitor Atlas workflow
argo_workflows_get_workflow({ 
  namespace: "cto", 
  name: "play-task-17-atlas" 
})

# 2. Check merge status
grafana_query_loki_logs({
  datasourceUid: "loki",
  logql: "{namespace=\"cto\", agent=\"atlas\"} |~ \"merge|merged|conflict\"",
  limit: 100
})

# 3. Verify CI passes
grafana_query_loki_logs({
  datasourceUid: "loki",
  logql: "{namespace=\"cto\", agent=\"atlas\"} |~ \"CI.*pass|checks.*pass\"",
  limit: 50
})

# 4. Check for merge conflicts
grafana_query_loki_logs({
  datasourceUid: "loki",
  logql: "{namespace=\"cto\", agent=\"atlas\"} |~ \"conflict|merge.*fail\"",
  limit: 20
})
```

### Phase 9: Deployment Verification

**Goal:** Verify feature deploys successfully

```bash
# 1. Check ArgoCD sync after merge
argocd_get_application({ applicationName: "target-service" })

# 2. Verify pods are running
grafana_query_prometheus({
  datasourceUid: "prometheus",
  expr: "kube_pod_status_phase{namespace=\"target-namespace\",phase=\"Running\"}",
  queryType: "instant"
})

# 3. Check for deployment errors
grafana_query_loki_logs({
  datasourceUid: "loki",
  logql: "{namespace=\"target-namespace\"} |~ \"error|ERROR|crash|panic\"",
  limit: 100
})

# 4. Verify service health
grafana_query_prometheus({
  datasourceUid: "prometheus",
  expr: "up{job=\"target-service\"}",
  queryType: "instant"
})

# 5. Check application logs
grafana_query_loki_logs({
  datasourceUid: "loki",
  logql: "{namespace=\"target-namespace\"} | json",
  limit: 50
})
```

### Phase 10: Post-Deployment Monitoring

**Goal:** Verify feature works in production

```bash
# 1. Check request rate
grafana_query_prometheus({
  datasourceUid: "prometheus",
  expr: "rate(http_requests_total{job=\"target-service\"}[5m])",
  queryType: "instant"
})

# 2. Check error rate
grafana_query_prometheus({
  datasourceUid: "prometheus",
  expr: "rate(http_requests_total{job=\"target-service\",status=~\"5..\"}[5m])",
  queryType: "instant"
})

# 3. Check latency
grafana_query_prometheus({
  datasourceUid: "prometheus",
  expr: "histogram_quantile(0.95, rate(http_request_duration_seconds_bucket{job=\"target-service\"}[5m]))",
  queryType: "instant"
})

# 4. Monitor for alerts
grafana_list_alert_rules()

# 5. Check dashboard
grafana_search_dashboards({ query: "target-service" })
grafana_generate_explore_url({
  datasourceUid: "prometheus",
  queries: [{
    expr: "up{job=\"target-service\"}",
    refId: "A"
  }]
})
```

### Healer Verification

**Goal:** Verify Healer is monitoring and can remediate issues

```bash
# 1. Check Healer is running
grafana_query_prometheus({
  datasourceUid: "prometheus",
  expr: "up{job=\"healer\"}",
  queryType: "instant"
})

# 2. Monitor Healer logs
grafana_query_loki_logs({
  datasourceUid: "loki",
  logql: "{namespace=\"cto\", app=\"healer\"} | json",
  limit: 100
})

# 3. Check for remediation actions
grafana_query_loki_logs({
  datasourceUid: "loki",
  logql: "{namespace=\"cto\", app=\"healer\"} |~ \"remediation|fixing|retry\"",
  limit: 50
})

# 4. Verify acceptance criteria checks
grafana_query_loki_logs({
  datasourceUid: "loki",
  logql: "{namespace=\"cto\", app=\"healer\"} |~ \"acceptance.*criteria|verify\"",
  limit: 50
})
```

### Browser Testing for Frontend Features

**Goal:** Verify frontend features work end-to-end

```bash
# 1. Navigate to deployed app
mcp_cursor-ide-browser_browser_navigate({ url: "https://app.example.com" })

# 2. Take accessibility snapshot
mcp_cursor-ide-browser_browser_snapshot()

# 3. Test user interactions
mcp_cursor-ide-browser_browser_click({ element: "login button", ref: "ref-123" })
mcp_cursor-ide-browser_browser_type({ 
  element: "username input", 
  ref: "ref-456", 
  text: "testuser" 
})

# 4. Capture screenshot for verification
mcp_cursor-ide-browser_browser_take_screenshot({ fullPage: true })

# 5. Check console for errors
mcp_cursor-ide-browser_browser_console_messages()

# 6. Verify network requests
mcp_cursor-ide-browser_browser_network_requests()
```

### Cost & Token Tracking

**Goal:** Monitor resource usage and costs across workflow

```bash
# 1. Track workflow duration
argo_workflows_get_workflow({ namespace: "cto", name: "workflow-name" })

# 2. Monitor CPU/memory costs
grafana_query_prometheus({
  datasourceUid: "prometheus",
  expr: "sum(rate(container_cpu_usage_seconds_total{namespace=\"cto\"}[5m])) by (pod)",
  queryType: "instant"
})

# 3. Check token usage in logs
grafana_query_loki_logs({
  datasourceUid: "loki",
  logql: "{namespace=\"cto\"} |~ \"tokens|cost|USD\"",
  limit: 50
})
```

---

## Context7 for Rust Best Practices

Before implementing significant Rust code, use Context7 to get current documentation.

### When to Query

Always consult Context7 when:
- Setting up async code with Tokio
- Implementing error handling (anyhow context patterns, thiserror enums)
- Using serde attributes or custom serialization
- Configuring Clippy pedantic lints
- Writing HTTP handlers or database queries

## Project Structure & Module Organization
- `mcp/` — Rust MCP server (`cto-mcp`).
- `controller/` — Rust controllers and binaries (e.g., `agent-controller`).
- `sidecar/` — Rust sidecar utilities.
- `infra/` — Helm charts, GitOps manifests, images, and scripts.
- `scripts/` — Bash helpers and validation utilities.
- `docs/` — Architecture, examples, and references.

## Build, Test, and Development Commands
- Build MCP server: `cd mcp && cargo build --release`.
- Build controller: `cd controller && cargo build --release --bin agent-controller`.
- Run tests (Rust): `cargo test -p cto-mcp` and `cargo test -p controller`.
- Lint/format (Rust): `cargo fmt --all --check` and `cargo clippy --all-targets -- -D warnings`.
- GitOps validation: `make -C infra/gitops validate` (or `lint`, `test`).
- Pre-commit checks: `pre-commit install && pre-commit run --all-files`.

## CLI Images & Registry

### Image Registry: GHCR (Standard)

All CLI images use **GHCR** (`ghcr.io/5dlabs/*`) as the canonical source:

| Image | Description |
|-------|-------------|
| `ghcr.io/5dlabs/factory` | Universal CLI image (claude, cursor, code, gemini, opencode) |
| `ghcr.io/5dlabs/controller` | Agent orchestrator with CodeRun templates |
| `ghcr.io/5dlabs/pm-server` | Linear/GitHub integration |
| `ghcr.io/5dlabs/tools` | MCP server proxy |

### Agent-Specific Prompts & Tools

Each agent receives prompts and tools **only for their supported language**:

| Agent | Language | Default Stack | Tools/Hints |
|-------|----------|---------------|-------------|
| **Rex** | Rust | axum | tokio, serde, anyhow, thiserror, sqlx |
| **Blaze** | React/TS | shadcn | Next.js 15, shadcn/ui, Effect, Tailwind, anime.js |
| **Grizz** | Go | chi | chi, grpc, pgx, redis |
| **Nova** | Node.js/Bun | Elysia | Elysia, Effect, Drizzle, Prisma |
| **Tap** | Expo | expo-router | expo, react-native, expo-router, XcodeBuildMCP |
| **Spark** | Electron | electron | electron-builder, react, XcodeBuildMCP (macOS) |

**Effect TypeScript** ([effect.website](https://effect.website)):
Both Nova and Blaze use **Effect** - the missing standard library for TypeScript:
- Type-safe error handling (errors as values)
- Composable, reusable code patterns
- Built-in concurrency (fibers, queues, semaphores)
- Schema validation and transformation
- Observability (tracing, metrics, logging)
- AI integrations (`@effect/ai` for LLM interactions)

**LLM Documentation:** Use `https://effect.website/llms.txt` for AI-assisted development with Effect.

### Agent Memory: OpenMemory

All agents have access to **OpenMemory** ([github.com/CaviraOSS/OpenMemory](https://github.com/CaviraOSS/OpenMemory)) for persistent long-term memory:

| Feature | Description |
|---------|-------------|
| **Multi-sector Memory** | Episodic, semantic, procedural, emotional, reflective |
| **Temporal Knowledge Graph** | Time-aware facts with `valid_from`/`valid_to` |
| **Decay & Salience** | Natural forgetting with coactivation reinforcement |
| **MCP Integration** | Native tools: `openmemory_query`, `openmemory_store`, `openmemory_list` |
| **Waypoint Traces** | Explainable recall paths for debugging |
| **Local-First** | SQLite storage, no cloud dependency |

**MCP Setup:**
```json
{
  "mcpServers": {
    "openmemory": {
      "type": "http",
      "url": "http://openmemory:8080/mcp"
    }
  }
}
```

**Alternative Memory Systems (Under Investigation):**

| System | Type | Best For | URL |
|--------|------|----------|-----|
| **Letta** | Agent framework | Infinite context, human-like memory | [letta.com](https://letta.com) |
| **Mem0** | Memory layer | Lightweight, LLM-agnostic | [mem0.ai](https://mem0.ai) |
| **Zep** | Temporal KG | Enterprise, structured memory | [getzep.com](https://getzep.com) |

### Research Pipeline (Bookmark Ingestion)

The Research crate monitors Twitter/X bookmarks and curates relevant content:

```
Twitter Bookmarks → Browser Automation → AI Analysis → Firecrawl Enrichment → Markdown Storage
```

**Categories:**
- `agents` - AI/LLM agent patterns
- `rust` - Rust ecosystem
- `infrastructure` - Kubernetes, cloud
- `tooling` - MCP, IDEs, dev tools
- `security` - Vulnerabilities, practices
- `research` - Academic papers

**CLI Usage:**
```bash
# Run poll cycle
research poll --output=/data/research --min-relevance=0.5

# Search entries
research search "async runtime"

# List by category
research list --category=rust
```

**Output:** Each entry saved as markdown with YAML frontmatter including relevance score, categories, and enriched link content.

**Frontend Stacks (Blaze):**
| Stack | Best For | Router | Data Layer | UI |
|-------|----------|--------|------------|-----|
| **shadcn** (Default) | Marketing, SEO, SSR | Next.js App Router | Server Actions + React Query | shadcn/ui |
| **tanstack** | Dashboards, real-time | TanStack Router | TanStack DB + Query | shadcn/ui |

**Backend Frameworks (Nova):**
| Framework | Best For | Runtime | Docs |
|-----------|----------|---------|------|
| **Elysia** (Default) | Modern APIs, MCP servers | Bun (fastest) | https://elysiajs.com |
| **Hono** | Edge, multi-runtime | Bun/Node/Deno/CF | https://hono.dev |
| **NestJS** | Enterprise, microservices | Node.js | https://nestjs.com |
| **Fastify** | High-performance APIs | Node.js | https://fastify.dev |

**Elysia MCP Server:** Nova can use [Elysia-mcp](https://github.com/keithagroves/Elysia-mcp) to build MCP servers that expose tools, resources, and prompts to LLMs.

**XcodeBuildMCP** ([github.com/cameroncooke/XcodeBuildMCP](https://github.com/cameroncooke/XcodeBuildMCP)):
Tap and Spark use **XcodeBuildMCP** for iOS/macOS builds with full Xcode integration:

| Workflow | Tools | Use Case |
|----------|-------|----------|
| **simulator** | 18 tools | iOS Simulator builds & testing |
| **device** | 14 tools | Physical iOS device deployment |
| **macos** | 11 tools | macOS app builds |
| **ui-testing** | 11 tools | UI automation & screenshots |
| **swift-package** | 6 tools | Swift Package Manager |
| **project-scaffolding** | 2 tools | Create new Xcode projects |

**Key Features:**
- Session-aware defaults (set simulator/device once, reuse everywhere)
- Dynamic tool loading to optimize context window
- Incremental builds support (experimental)
- Code signing for device deployment
- UI automation and screen capture

**MCP Setup:**
```json
{
  "mcpServers": {
    "XcodeBuildMCP": {
      "command": "npx",
      "args": ["-y", "xcodebuildmcp@latest"]
    }
  }
}
```

Configure in `cto-config.json`:
```json
{
  "agents": {
    "rex": {
      "githubApp": "rex",
      "cli": "claude",
      "tools": {
        "remote": ["context7", "openmemory_*", "mcp_linear_*", "mcp_github_*"],
        "hints": ["tokio", "axum"]
      }
    }
  }
}
```

**Required Linear MCP Tools (all agents):**
- `mcp_linear_update_issue` - Update task status, description
- `mcp_linear_create_comment` - Add comments to task issue
- `mcp_linear_list_comments` - Read existing comments
- `mcp_linear_get_issue` - Fetch task details

**Required GitHub MCP Tools (all agents):**
- `mcp_github_create_pull_request` - Create PR for task branch
- `mcp_github_create_branch` - Create feature branch
- `mcp_github_push_files` - Push code changes

**Required OpenMemory MCP Tools (all agents):**
- `openmemory_query` - Search memories by semantic similarity
- `openmemory_store` - Store new memories with sector classification
- `openmemory_list` - List memories for a user/agent
- `openmemory_get` - Retrieve specific memory by ID
- `openmemory_reinforce` - Boost salience of important memories

### Tools Configuration Flow

The tools server uses `tools-config.json` to configure which MCP tools are available to each agent. The controller renders this config based on `cto-config.json`:

```
Intake Workflow                    Play Workflow
┌──────────────┐                  ┌──────────────┐
│  PRD Input   │                  │  Controller  │
└──────┬───────┘                  └──────┬───────┘
       │ Analyze requirements            │ Read agent config
       ▼                                 ▼
┌──────────────┐                  ┌──────────────┐
│ Generate     │                  │ Render       │
│ cto-config   │─────stored──────►│ tools-config │
│ .json        │   in repo        │ .json        │
└──────────────┘                  └──────┬───────┘
                                         │ Mount in pod
                                         ▼
                                  ┌──────────────┐
                                  │ MCP Client   │
                                  │ (tools-mcp)  │
                                  └──────────────┘
```

**cto-config.json Generated During Intake:**

The intake workflow analyzes the PRD and generates `cto-config.json` with:
- **Required agents** based on languages detected in PRD
- **Remote tools** for each agent (context7, github, openmemory, etc.)
- **Local servers** only when project needs specific URLs (PostgreSQL, Redis)

```json
// Generated cto-config.json
{
  "agents": {
    "rex": {
      "githubApp": "5DLabs-Rex",
      "cli": "factory",
      "model": "claude-opus-4-5-20251101",
      "tools": {
        "remote": [
          "context7_resolve_library_id",
          "context7_get_library_docs",
          "github_create_pull_request",
          "openmemory_openmemory_query"
        ],
        "localServers": {}  // Empty - no project-specific services
      }
    }
  }
}
```

**Remote vs Local Servers:**

| Type | When to Use | Examples |
|------|-------------|----------|
| **Remote** | Most use cases - platform-provided tools | context7, github, openmemory, brave_search, kubernetes |
| **Local** | Project-specific URLs that differ per project | PostgreSQL (custom host), Redis (custom port), custom APIs |

**tools-config.json Format:**
```json
{
  "remoteTools": [
    "openmemory_openmemory_query",
    "openmemory_openmemory_store",
    "openmemory_openmemory_list"
  ],
  "localServers": {
    "postgres": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-postgres"],
      "tools": ["query"],
      "workingDirectory": "project_root",
      "env": {
        "POSTGRES_URL": "${POSTGRES_URL:-postgresql://localhost:5432/postgres}"
      }
    },
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "."],
      "tools": ["read_file", "write_file", "list_directory"],
      "workingDirectory": "project_root",
      "env": {}
    }
  }
}
```

**Configuration Fields:**
- `remoteTools` - Array of tool names from remote MCP servers (openmemory, context7, github, etc.)
- `localServers` - Object defining local MCP servers to spawn per-agent:
  - `command` / `args` - How to run the server (e.g., `npx -y @modelcontextprotocol/server-postgres`)
  - `tools` - Which tools to expose from this server
  - `workingDirectory` - Execution context (`project_root` or absolute path)
  - `env` - Environment variables (supports `${VAR:-default}` syntax)

**Remote vs Local servers:**
- **Remote**: Proxied through tools-server (openmemory, context7, github, brave_search, kubernetes)
- **Local**: Spawned per-agent for project-specific access (PostgreSQL, Redis, filesystem)

**Tools validation pre-check:**
1. `cto-config.json` matches controller rendering
2. CLI only has access to tools in `tools-config.json`
3. Remote tools available from tools-server
4. Local servers (if any) spawned and reachable

### CLI Reference for Non-Interactive Mode

Each CLI supports non-interactive (headless) execution for agent workflows:

| CLI | Non-Interactive Command | Key Flags |
|-----|------------------------|-----------|
| **claude** | `claude -p "query"` | `--output-format stream-json`, `--dangerously-skip-permissions`, `--mcp-config` |
| **code** | `code --no-approval "query"` | `--model`, `--sandbox`, `--config`, `--read-only` |
| **gemini** | `gemini -p "query"` | `--output-format stream-json`, `-m model` |
| **opencode** | `opencode -p "query"` | `--output-format stream-json`, provider-agnostic |
| **cursor** | `cursor -p "query"` | Uses Cursor auth, IDE integration |
| **dexter** | `dexter -p "query"` | Agentic workflows |

### Detailed CLI Reference

#### Claude Code CLI

**Installation:** `npm install -g @anthropic-ai/claude-code`

| Flag | Description |
|------|-------------|
| `-p, --print` | Non-interactive mode, prints response and exits |
| `--output-format json\|stream-json` | Structured output for parsing |
| `--mcp-config` | Load MCP servers from JSON config |
| `--dangerously-skip-permissions` | Skip tool permission prompts (agent mode) |
| `--max-turns` | Limit agentic turns in non-interactive mode |
| `--model` | Model alias: `sonnet` or `opus` |

```bash
# Non-interactive execution for agents
claude -p "Implement the feature" \
  --output-format stream-json \
  --dangerously-skip-permissions \
  --mcp-config ./tools-config.json
```

#### Every Code CLI

**Installation:** `git clone https://github.com/just-every/code.git && cd code && ./build-fast.sh`

| Command/Flag | Description |
|--------------|-------------|
| `code --no-approval "query"` | Non-interactive mode for agents |
| `/plan` | Claude + Gemini + GPT-5 consensus for planning |
| `/solve` | Fastest-first race for problem solving |
| `/code` | Multi-worktree implementation with consensus |
| `/auto` | Auto Drive for multi-step task orchestration |
| `--model` | Override model (e.g., `gpt-5.1`, `gpt-5.2`) |
| `--sandbox` | read-only \| workspace-write \| danger-full-access |
| `--read-only` | Restrict to read-only operations |

```bash
# Non-interactive execution for agents
code --no-approval "Implement the feature" \
  --model gpt-5.1 \
  --sandbox workspace-write
```

#### Google Gemini CLI

**Installation:** `npm install -g @google/gemini-cli`

| Flag | Description |
|------|-------------|
| `-p` | Non-interactive mode (print mode) |
| `--output-format json\|stream-json` | Structured output for parsing |
| `-m` | Model selection (e.g., `gemini-2.5-flash`) |
| `--include-directories` | Add additional working directories |

```bash
# Non-interactive execution for agents
gemini -p "Implement the feature" \
  --output-format stream-json

# Authentication via environment
export GOOGLE_API_KEY="YOUR_API_KEY"
# or for Vertex AI:
export GOOGLE_GENAI_USE_VERTEXAI=true
```

#### OpenCode CLI (SST)

**Installation:** `npm install -g opencode-ai`

| Feature | Description |
|---------|-------------|
| `build` agent | Default, full access for development work |
| `plan` agent | Read-only for analysis and planning |
| Client/server | Can run server locally, drive from mobile app |
| MCP support | Configure in `~/.opencode/settings.json` |
| Provider-agnostic | Claude, OpenAI, Google, or local models |

```bash
# Non-interactive execution
opencode -p "Implement the feature" \
  --output-format stream-json

# Uses ANTHROPIC_API_KEY, OPENAI_API_KEY, or GOOGLE_API_KEY
```

#### Factory CLI (Droid)

**Installation:** Built into `ghcr.io/5dlabs/factory` image

Factory Droid is a multi-CLI image that includes all supported CLIs. The container script selects which CLI to invoke based on the `CLI` environment variable.

```bash
# Container startup script selects CLI
case "$CLI" in
  claude)   claude -p "$PROMPT" --output-format stream-json ... ;;
  code)     code --no-approval "$PROMPT" --model gpt-5.1 ... ;;
  gemini)   gemini -p "$PROMPT" --output-format stream-json ... ;;
  opencode) opencode -p "$PROMPT" --output-format stream-json ... ;;
  cursor)   cursor -p "$PROMPT" ... ;;
  dexter)   dexter -p "$PROMPT" ... ;;
esac
```

### Supported CLIs with Input Methods

| CLI | Input Method | Flag |
|-----|--------------|------|
| claude | stream-json FIFO | `--input-format stream-json` |
| code | JSONL file | `--input-file` |
| cursor | Not supported | N/A |
| dexter | FIFO | `--input` |
| opencode | JSONL file | `--input` |
| gemini | Not supported | N/A |

**Every Code (Recommended):**

We use **Every Code** ([github.com/just-every/code](https://github.com/just-every/code)) - a powerful fork of OpenAI Codex with multi-agent capabilities:

| Command | Description | Agents Used |
|---------|-------------|-------------|
| `/plan` | Plan code changes | Claude + Gemini + GPT-5 consensus |
| `/solve` | Solve complex problems | Fastest-first race |
| `/code` | Write code with worktrees | Consensus implementation |
| `/auto` | Multi-step task orchestration | Auto Drive coordinator |

**Installation:**
```bash
# Install Every Code and companion CLIs
npm install -g @anthropic-ai/claude-code @google/gemini-cli
git clone https://github.com/just-every/code.git && cd code && ./build-fast.sh
```

**Configuration** (`~/.code/config.toml`):
```toml
model = "gpt-5.1"
model_provider = "openai"
approval_policy = "never"  # For CI/CD
model_reasoning_effort = "high"
sandbox_mode = "workspace-write"
```

**External Documentation:**
- Claude Code: [docs.anthropic.com/en/docs/claude-code/cli-reference](https://docs.anthropic.com/en/docs/claude-code/cli-reference)
- Every Code: [github.com/just-every/code](https://github.com/just-every/code)
- Gemini CLI: [github.com/google-gemini/gemini-cli](https://github.com/google-gemini/gemini-cli)
- OpenCode: [opencode.ai/docs](https://opencode.ai/docs)

## Local Development with Tilt + ArgoCD

**Tilt** provides continuous development with **ArgoCD** managing deployments. Changes are built locally, pushed to an in-cluster registry, and deployed via ArgoCD.

### Quick Start

```bash
# One-time setup:
# 1. Install Tilt
curl -fsSL https://raw.githubusercontent.com/tilt-dev/tilt/master/scripts/install.sh | bash

# 2. Deploy local registry (if not already deployed)
./scripts/dev-load.sh --setup

# 3. Enable dev registry in ArgoCD
./scripts/argocd-dev-mode.sh enable

# Start developing (watches files, builds, deploys)
tilt up

# Open the Tilt UI: http://localhost:10350
```

### How It Works

1. **Tilt watches** your `crates/` directory for changes
2. **Builds images** with BuildKit caching (first build ~5-10min, subsequent ~1-2min)
3. **Pushes to local registry** at `192.168.1.77:30500`
4. **Restarts deployments** so pods pull the new images
5. **ArgoCD stays synced** because it's configured to use the same registry

### BuildKit Caching

All Dockerfiles use BuildKit cache mounts:
- **Shared cargo registry**: Downloaded crates cached and shared across all images
- **Per-image target cache**: Compiled artifacts cached per service
- **First build**: ~5-10 minutes (downloads all dependencies)
- **Subsequent builds**: ~1-2 minutes (uses cache)

### ArgoCD Integration

```bash
# Check dev registry status
./scripts/argocd-dev-mode.sh status

# Enable dev registry (use local images)
./scripts/argocd-dev-mode.sh enable

# Disable dev registry (revert to GHCR)
./scripts/argocd-dev-mode.sh disable
```

### Cleanup

```bash
# Stop Tilt
tilt down

# Disable dev registry (ArgoCD reverts to GHCR images)
./scripts/argocd-dev-mode.sh disable
```

## Coding Style & Naming Conventions
- Rust: rustfmt (Edition 2021, `max_width=100`); prefer `tracing::*` over `println!` (enforced by Clippy). Binary names kebab-case (e.g., `agent-controller`); files/modules snake_case (e.g., `src/bin/agent_controller.rs`).
- YAML: 2-space indent; begin docs with `---`; follow `.yamllint.yaml`.
- Markdown: follow `.markdownlint.yaml` (incremental headings, no trailing spaces).
- Shell: keep `bash -euo pipefail`; validate with ShellCheck where applicable.

## Testing Guidelines
- Unit tests live alongside code (`mod tests { ... }`) and in Rust integration tests when needed. Run `cargo test` in crate roots.
- For controllers and workflows, add small, deterministic tests; prefer fixtures under `controller/src/**/tests/`.
- Validate infrastructure changes with `make -C infra/gitops validate` and attach output to PRs when material.

## Commit & Pull Request Guidelines
- Use Conventional Commits: `feat:`, `fix:`, `chore:`, `refactor:`, etc. Keep commits focused and descriptive.
- Before pushing: `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `pre-commit run --all-files`.
- PRs must include: summary, rationale, scope of impact, and verification steps; link issues (e.g., `Closes #123`). Include logs/screenshots for infra or CLI-facing changes.

## Security & Configuration Tips
- Never commit secrets. Use Kubernetes secrets/Helm values under `infra/secret-store/` and local env vars.
- Use `cto-config.json` (see `cto-config-example.json`) and `.cursor/mcp.json` to configure local runs; avoid committing user-specific tokens.
- Large files and generated artifacts should not be committed unless explicitly required.

## Secrets Management

### Local Development: 1Password CLI

For local secrets (API keys, tokens, credentials), use **1Password CLI** (`op`):

```bash
# Sign in to 1Password
op signin

# Read a secret
op read "op://Private/Cloudflare API Token/credential"

# Inject secrets into environment
op run --env-file=.env.local -- cargo run

# List items in a vault
op item list --vault="5DLabs"
```

#### CLI API Keys (1Password)

| CLI | Environment Variable | 1Password Item | Models |
|-----|---------------------|----------------|--------|
| **claude** | `ANTHROPIC_API_KEY` | `Anthropic API Key` | claude-opus-4-5-20250929, claude-sonnet-4-20250514 |
| **code** | `OPENAI_API_KEY` | `OpenAI API Key` | gpt-5.1, gpt-5.2, o3, o4-mini |
| **gemini** | `GOOGLE_API_KEY` | `Google AI API Key` | gemini-2.5-pro, gemini-2.5-flash |
| **dexter** | `ANTHROPIC_API_KEY` | `Anthropic API Key` | claude-sonnet-4-20250514 |
| **opencode** | `ANTHROPIC_API_KEY` | `Anthropic API Key` | claude-opus-4-5-20250929 |
| **cursor** | N/A (uses Cursor auth) | N/A | claude-sonnet-4-20250514 |

**Note:** Every Code (`code`) also supports ChatGPT Plus sign-in as an alternative to API keys.

#### GitHub Apps (1Password)

Each agent has its own GitHub App for repository access. Store in 1Password vault `5DLabs`:

| Agent | GitHub App Name | 1Password Item | Keys |
|-------|----------------|----------------|------|
| **Morgan** | `5DLabs-Morgan` | `GitHub App - Morgan` | `app-id`, `client-id`, `private-key` |
| **Rex** | `5DLabs-Rex` | `GitHub App - Rex` | `app-id`, `client-id`, `private-key` |
| **Blaze** | `5DLabs-Blaze` | `GitHub App - Blaze` | `app-id`, `client-id`, `private-key` |
| **Cleo** | `5DLabs-Cleo` | `GitHub App - Cleo` | `app-id`, `client-id`, `private-key` |
| **Cipher** | `5DLabs-Cipher` | `GitHub App - Cipher` | `app-id`, `client-id`, `private-key` |
| **Tess** | `5DLabs-Tess` | `GitHub App - Tess` | `app-id`, `client-id`, `private-key` |
| **Bolt** | `5DLabs-Bolt` | `GitHub App - Bolt` | `app-id`, `client-id`, `private-key` |
| **Atlas** | `5DLabs-Atlas` | `GitHub App - Atlas` | `app-id`, `client-id`, `private-key` |

#### Linear Agent GitHub App

The Linear agent uses a dedicated GitHub App for webhook processing and repository access:

| Component | GitHub App Name | 1Password Item | Keys |
|-----------|----------------|----------------|------|
| **Linear Agent** | `5DLabs-Talos` | `GitHub App - Talos` | `app-id`, `client-id`, `private-key`, `webhook-secret` |

**Talos App Responsibilities:**
- Receives webhooks from Linear (issue updates, comments)
- Triggers intake/play workflows on label changes
- Syncs Linear issue ↔ GitHub PR status
- Processes merge events to start Play workflows

#### Other Secrets (1Password)

| Secret | 1Password Item | Environment Variable | Usage |
|--------|----------------|---------------------|-------|
| `Cloudflare API Token` | Infrastructure | `CF_API_KEY` | Cloudflare Pages, Tunnels, DNS |
| `Linear API Key` | Integration | `LINEAR_API_KEY` | Linear issue sync |
| `Firecrawl API Key` | Tools | `FIRECRAWL_API_KEY` | Web scraping MCP server |
| `Context7 API Key` | Tools | `CONTEXT7_API_KEY` | Documentation lookup MCP |
| `GitHub PAT` | Tools | `GITHUB_TOKEN` | GitHub MCP operations |
| `Kubeconfig` | Infrastructure | `KUBECONFIG` | K8s MCP tools (file path or base64) |

#### Tools Server Secrets

The MCP tools server requires these secrets mounted:

```bash
# Required for tools-server deployment
FIRECRAWL_API_KEY    # Firecrawl web scraping
CONTEXT7_API_KEY     # Context7 documentation lookup  
GITHUB_TOKEN         # GitHub API operations (PAT with repo scope)
KUBECONFIG           # Kubernetes cluster access (base64 encoded or file)
LINEAR_API_KEY       # Linear integration
```

### Cluster Secrets: OpenBao + External Secrets Operator

For Kubernetes cluster secrets, we use **OpenBao** (Vault fork) with the **External Secrets Operator (ESO)**:

```yaml
# ExternalSecret example - syncs from Bao to K8s Secret
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: github-app-5dlabs-rex
  namespace: cto
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: openbao
    kind: ClusterSecretStore
  target:
    name: github-app-5dlabs-rex
  data:
    - secretKey: app-id
      remoteRef:
        key: github-app-rex
        property: app-id
    - secretKey: private-key
      remoteRef:
        key: github-app-rex
        property: private-key
```

**Bao CLI commands:**

```bash
# Authenticate to Bao
export BAO_ADDR="https://bao.5dlabs.ai"
bao login -method=oidc

# Read a secret
bao kv get github-app-rex

# Write a secret
bao kv put github-app-rex app-id="123456" client-id="..." private-key=@key.pem

# List secrets
bao kv list /
```

#### Bao Secret Paths

| Path | Contents | Used By |
|------|----------|---------|
| `github-app-morgan` | GitHub App credentials | Morgan agent |
| `github-app-rex` | GitHub App credentials | Rex agent |
| `github-app-blaze` | GitHub App credentials | Blaze agent |
| `github-app-cleo` | GitHub App credentials | Cleo agent |
| `github-app-cipher` | GitHub App credentials | Cipher agent |
| `github-app-tess` | GitHub App credentials | Tess agent |
| `github-app-bolt` | GitHub App credentials | Bolt agent |
| `github-app-atlas` | GitHub App credentials | Atlas agent |
| `github-app-talos` | GitHub App credentials | Linear agent (PM server) |
| `linear-sync` | `LINEAR_API_KEY`, `LINEAR_WEBHOOK_SECRET` | PM server |
| `anthropic-api` | `ANTHROPIC_API_KEY` | Claude CLI |
| `openai-api` | `OPENAI_API_KEY` | Code CLI (Every Code) |
| `google-ai-api` | `GOOGLE_API_KEY` | Gemini CLI |
| `firecrawl-api` | `FIRECRAWL_API_KEY` | Web scraping MCP |
| `context7-api` | `CONTEXT7_API_KEY` | Documentation MCP |
| `github-pat` | `GITHUB_TOKEN` | GitHub MCP operations |
| `tools-kubernetes` | `KUBECONFIG` | K8s MCP tools |
| `cloudflare-api` | `CF_API_KEY` | Cloudflare operations |

#### Syncing Secrets from 1Password to Bao

```bash
# Use the sync script to copy secrets from 1Password to Bao
./scripts/sync-credentials-from-1password.sh

# Or manually sync a single secret
op read "op://5DLabs/GitHub App - Rex/private-key" | \
  bao kv put github-app-rex private-key=-
```

## Public Hosting: Cloudflare

For public hosting of static assets, documentation, or tunnels, use **Cloudflare**:

### Cloudflare Pages (Static Sites)

```bash
# Deploy to Cloudflare Pages
npx wrangler pages deploy ./dist --project-name=my-project

# Or use the Cloudflare dashboard for GitHub integration
```

### Cloudflare Tunnels (Local Services)

For exposing local services publicly without port forwarding:

```bash
# List existing tunnels
cloudflared tunnel list

# Create a new tunnel
cloudflared tunnel create my-tunnel

# Run a tunnel to expose local port
cloudflared tunnel run --url http://localhost:3000 my-tunnel

# Configure tunnel routes (in Cloudflare dashboard or config)
# Routes are managed at: https://one.dash.cloudflare.com/
```

**Active tunnels:**
- `cto-main` - Main platform tunnel (active connections)
- `cto-kind-dev` - Development cluster tunnel
- `5dlabs-platform` - Production platform tunnel

**Domain:** All public services use `*.5dlabs.ai` domain via Cloudflare DNS.

## MCP Tools for Observability & GitOps

The platform provides MCP (Model Context Protocol) tools for interacting with observability and GitOps systems. These tools require port-forwards to be active.

### Port-Forward Requirements

```bash
# Prometheus (metrics)
kubectl port-forward svc/prometheus-server -n observability 9090:80

# Loki (logs)
kubectl port-forward svc/loki-gateway -n observability 3100:80

# Grafana (dashboards)
kubectl port-forward svc/grafana -n observability 3000:80

# ArgoCD (GitOps)
kubectl port-forward svc/argocd-server -n argocd 8080:80

# Argo Workflows
kubectl port-forward svc/argo-workflows-server -n automation 2746:2746
```

### Prometheus (Metrics)

Query metrics for performance analysis:

```bash
# Check service health
prometheus_execute_query({ query: "up{job=\"cto-controller\"}" })

# Error rates
prometheus_execute_query({ query: "rate(http_requests_total{status=~\"5..\"}[5m])" })

# Range queries for trends
prometheus_execute_range_query({
  query: "rate(http_requests_total[5m])",
  start: "now-1h", end: "now", step: "1m"
})

# List available metrics
prometheus_list_metrics({ filter_pattern: "cto_" })
```

### Loki (Logs)

Search and analyze application logs:

```bash
# Query logs with filters
loki_query({
  query: "{namespace=\"cto\"} |~ \"error|ERROR\"",
  limit: 100
})

# Get available labels
loki_label_names()
loki_label_values({ label: "pod" })
```

**LogQL Patterns:**
- `{namespace="cto"}` - Label selector
- `|~ "pattern"` - Regex match
- `|= "exact"` - Exact match
- `| json | level="error"` - Parse JSON and filter

### Grafana (Dashboards & Alerts)

Access dashboards, alerts, and query datasources:

```bash
# Search dashboards
grafana_search_dashboards({ query: "CTO" })

# Query metrics via Grafana
grafana_query_prometheus({
  datasourceUid: "prometheus",
  expr: "up",
  queryType: "instant"
})

# Query logs via Grafana
grafana_query_loki_logs({
  datasourceUid: "loki",
  logql: "{namespace=\"cto\"}",
  limit: 50
})

# List alert rules
grafana_list_alert_rules()
```

### ArgoCD (GitOps)

Manage GitOps deployments:

```bash
# List applications
argocd_list_applications()

# Get application status (sync, health)
argocd_get_application({ applicationName: "cto-controller" })

# Check resources and events
argocd_get_application_resource_tree({ applicationName: "app" })
argocd_get_application_events({ applicationName: "app" })

# Trigger sync
argocd_sync_application({ applicationName: "app" })
```

### Argo Workflows

Monitor and manage workflow executions:

```bash
# List workflows
argo_workflows_list_workflows({ namespace: "cto", limit: 20 })

# Get workflow details and logs
argo_workflows_get_workflow({ namespace: "cto", name: "workflow-name" })
argo_workflows_get_workflow_logs({ namespace: "cto", workflow_name: "name" })

# List templates and cron workflows
argo_workflows_list_workflow_templates({ namespace: "cto" })
argo_workflows_list_cron_workflows({ namespace: "automation" })

# Retry failed workflow
argo_workflows_retry_workflow({ namespace: "cto", name: "failed-workflow" })
```

### Healer Monitoring

Healer monitors workflow execution and verifies acceptance criteria. Key areas to watch:

| Category | What to Watch | Log Pattern |
|----------|--------------|-------------|
| Workspace | Working directory resolution | `working_directory\|cwd\|WORKING_DIRECTORY` |
| MCP Tools | Tool availability and routing | `Tool.*not found\|routing.*failed` |
| Prompts | Prompt loading and rendering | `prompt.*error\|template.*failed` |
| Permissions | GitHub, K8s, file access | `permission denied\|unauthorized\|EACCES` |
| Sidecar | Container termination | `sidecar.*timeout\|container.*not.*terminat` |
| CLI Exit | Agent CLI not exiting | `cli.*hang\|timeout.*exit\|stuck` |

**Healer LogQL Queries:**
```bash
# All errors from play workflows
{namespace="cto"} |~ "error|ERROR|failed|Failed" | json

# Workspace issues
{namespace="cto", app="coderun"} |~ "working_directory|cwd|WORKING_DIRECTORY"

# MCP tool problems
{namespace="cto"} |~ "Tool.*not found|routing.*failed|handshake"

# Permission issues
{namespace="cto"} |~ "permission denied|unauthorized|forbidden|EACCES"
```

### Debugging Workflow

When investigating issues:

1. **Metrics** - `prometheus_execute_query({ query: "up{job=\"...\"}" })`
2. **Logs** - `loki_query({ query: "{pod=~\"...\"}|~\"error\"" })`
3. **Deployment** - `argocd_get_application({ applicationName: "..." })`
4. **Workflows** - `argo_workflows_list_workflows({ status: "Failed" })`

### Common Debugging Scenarios

**Workflow Not Starting:**
```bash
# Check workflow status
kubectl get workflow -n cto -l workflow-type=play-orchestration

# Check for errors
kubectl describe workflow <workflow-name> -n cto
```
Common causes: Missing required parameters (repository, service), invalid agent configuration, RBAC permissions missing.

**GitHub Authentication Failing:**
Error: `could not read Username for 'https://github.com': terminal prompts disabled`
```bash
# Check secrets
kubectl get secret github-app-rex -n cto -o yaml
```
Common causes: GitHub App credentials not configured, secret not mounted in pod, app not installed on target repository.

**CodeRun Pod Not Starting:**
```bash
# Check CodeRun status
kubectl get coderuns -n cto
kubectl describe coderun <name> -n cto

# Check controller logs
kubectl logs -n cto -l app.kubernetes.io/name=controller --tail=100
```
Common causes: Image pull errors, PVC mounting issues, resource limits exceeded.

**Agent Timeout (30 min default):**
```bash
# Check pod logs
kubectl logs <pod-name> -n cto --all-containers
```
Common causes: AI rate limiting, complex task taking too long, agent stuck in loop.

**Useful Commands:**
```bash
# List all play workflows
kubectl get workflow -n cto -l workflow-type=play-orchestration

# Watch CodeRuns
kubectl get coderuns -n cto -w

# Get pod logs
kubectl logs <pod-name> -n cto --all-containers

# Check progress ConfigMap
kubectl get configmap -n cto -l play-tracking=true

# Delete stuck workflow
kubectl delete workflow <name> -n cto

# Retry failed workflow
argo retry <workflow-name> -n cto
```

---

## Known Issues & Fixes

### Issue: Sidecar Doesn't Terminate

**Symptoms:** Pod stays in `Running` state after CLI completes. Sidecar container keeps running.

**Root Cause:** Sidecar's `tokio::select!` waits for tasks that never complete (HTTP server, FIFO writer).

**Fix:**
- Watch for `/workspace/.agent_done` sentinel file
- Set shutdown flag when detected → all tasks exit gracefully
- Add timeout after agent completion (e.g., 30s grace period)

```rust
// In status_sync main loop
if fs::metadata("/workspace/.agent_done").await.is_ok() {
    info!("Agent completed, initiating shutdown");
    shutdown.store(true, Ordering::SeqCst);
}
```

### Issue: Docker Sidecar Doesn't Terminate

**Symptoms:** Docker-in-Docker sidecar keeps running after build completes.

**Root Cause:** Docker daemon process doesn't receive SIGTERM propagation.

**Fix:**
- Use `shareProcessNamespace: true` in pod spec
- Add preStop hook to signal Docker daemon
- Set `terminationGracePeriodSeconds` appropriately

### Issue: Agent CLI Doesn't Exit

**Symptoms:** CLI process hangs after completing work, never returns exit code.

**Root Cause:** Some CLIs wait for stdin EOF or explicit quit signal.

**Fix:**
- Use `--print` or non-interactive flags where available
- Close stdin after sending prompt (`stdin(Stdio::null())`)
- Send explicit `/exit` command for interactive CLIs
- Add timeout with SIGTERM → SIGKILL escalation

```bash
# In container script
timeout --signal=TERM --kill-after=60s 3600s claude --print "$PROMPT"
```

---

## Healer Monitoring Details

### Acceptance Criteria Verification

Healer probes each stage's acceptance criteria:

- **Intake** - Tasks created? Dependencies valid? Linear project exists?
- **Implementation** - Code compiles? Tests pass? PR created?
- **Quality** - Lints pass? No code smells? Documented?
- **Security** - No vulnerabilities? Input validated?
- **Testing** - Tests written? Coverage acceptable?
- **Integration** - No conflicts? CI passes? PR merged?

### Common Issues to Detect

**Workspace Location Issues:**
```
Symptoms: Tools can't find files, relative paths resolve incorrectly
Log patterns: WORKING_DIRECTORY not set, cwd: /workspace (should be /workspace/repo)
Fix: Ensure MCP_CLIENT_CONFIG points to correct working directory
```

**MCP Tools Not Available:**
```
Symptoms: "Tool not found" errors, tools list empty, handshake failures
Log patterns: [Bridge] Filtering out remote tool, Tool.*not available, handshake failed
Fix: Check tools-config.json, verify tools-server is running
```

**Prompt Loading Failures:**
```
Symptoms: Agent doesn't understand task, generic responses, missing context
Log patterns: prompt.xml not found, template rendering failed
Fix: Verify .tasks/ directory exists, check prompt files are valid
```

**Permission Denied:**
```
Symptoms: Can't push to GitHub, can't create K8s resources
Log patterns: permission denied, 403 Forbidden, EACCES
Fix: Check GitHub App installation, verify ServiceAccount RBAC
```

**Sidecar Not Terminating:**
```
Symptoms: Pod stuck in "Running" after agent completes
Log patterns: sidecar waiting for, status.json not found
Fix: Ensure CLI writes completion status, check sidecar shutdown handling
```

**CLI Not Exiting:**
```
Symptoms: Agent appears stuck, no new output but process running
Log patterns: waiting for input, interactive mode, stdin blocked
Fix: Ensure --dangerously-skip-permissions flag, use -p flag
```

