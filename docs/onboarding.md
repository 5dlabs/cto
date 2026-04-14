# CTO Platform — Onboarding Guide

## What We're Building

The **Cognitive Task Orchestrator (CTO)** is an AI-powered software development platform. You feed it a PRD (product requirements document), and it runs a multi-stage pipeline that:

1. **Researches** requirements (via Exa/Perplexity/Tavily/Firecrawl)
2. **Deliberates** on architecture using a committee of AI models that vote
3. **Generates tasks** — broken down, refined, deduplicated, with scaffolds and docs
4. **Dispatches work** to specialized AI agents running in Kubernetes pods
5. **Tracks everything** through Discord threads, Linear issues, and CRDs

## Architecture Layers

| Layer | What | Tech |
|-------|------|------|
| **Config** | Agent roster, model assignments, tools, skills | `cto-config.json` (JSON, consumed by TS/Rust/Lobster) |
| **Pipelines** | Multi-stage intake/deliberation/voting workflows | Lobster YAML (`intake/workflows/*.lobster.yaml`) |
| **Controller** | Watches CRDs, spawns agent pods, manages lifecycle | Rust (`crates/controller/`) |
| **Apps** | Discord bridge, intake agent, Linear bridge, NATS messenger | TypeScript (`apps/`) |
| **Infra** | K8s + ArgoCD + Helm, Talos Linux, Tauri desktop app w/ Kind | `infra/`, `charts/` |
| **Agents** | OpenClaw-based agents running Claude/Copilot/Gemini/GPT with MCP tools | Pod-per-agent in cluster |

## The Agent Roster

22 specialized agents: morgan, rex, grizz, nova, viper, blaze, tap, spark, cleo, cipher, tess, stitch, atlas, bolt, block, vex, angie, glitch, lex, hype, tally, chase.

Each has assigned model providers, tools, and skills defined in `cto-config.json`.

## Current Focus

**Getting Lobster flows executing end-to-end through the Play.** We're currently setting up the OpenClaw architecture with test repos to validate the full loop: PRD in → pipeline runs → tasks generated → agents dispatched.

## What's Been Actively Worked On

- Per-agent Discord MCP servers (no more cross-talk between agents)
- Task deduplication mutex
- CRD schema overhaul (multi-agent fields, debug mode, parameter passing)
- OpenClaw model routing and provider qualification
- Neon Gauntlet E2E test suite
- Qdrant/Mem0 vector DB for agent memory
- ArgoCD sync workflow for auto-deploy on merge

## Outstanding Work

### Pipeline (Lobster flows)

- Full `pipeline.lobster.yaml → intake.lobster.yaml` execution with real test repos
- Workflow resumption (recovering interrupted runs)
- `"provider": "auto"` mode for credit-aware model routing (deferred)

### Controller (Rust)

- PostgreSQL storage backend (~14 TODO items for SQL queries)
- Healer tracking (remediation tracking, task status queries)
- Label controller (cleanup, override storage, conflict detection — deferred)
- OAuth per-agent, session tracking, attempt recovery

### Apps (TypeScript)

- Linear bridge cascade updates, dependency-aware "next task"
- Intake util `sync-linear` refinement

### Infra

- OpenTelemetry/Datadog integration completion
- Server availability pinging
- Cluster hardening and observability

---

## Resources — Getting Up to Speed

### OpenClaw (Agent Orchestration Runtime)

OpenClaw is the runtime that hosts agents, manages tool execution, and provides the gateway for agent communication. Agents run inside OpenClaw with access to MCP tool servers.

**Official docs:**

- Main site: https://docs.openclaw.ai
- Gateway setup: https://docs.openclaw.ai/cli/gateway
- Gateway config reference: https://docs.openclaw.ai/gateway/configuration
- LLM Task plugin: https://docs.openclaw.ai/tools/llm-task

**Source repo:** https://github.com/5dlabs/openclaw-platform

**In-repo docs (start here):**

- [`docs/intake-lobster-openclaw-process.md`](intake-lobster-openclaw-process.md) — how Lobster + OpenClaw + intake fit together (read this first)
- [`docs/openclaw-local-setup.md`](openclaw-local-setup.md) — running the gateway locally
- [`docs/intake-local-prereqs.md`](intake-local-prereqs.md) — env vars, secrets, workspace setup
- `intake/scripts/verify-lobster-openclaw.sh` — runtime verification script (run this to check your setup)

### Lobster (Workflow Engine)

Lobster is a declarative YAML-based workflow DSL that OpenClaw agents invoke as a tool. It orchestrates multi-step pipelines with subworkflows, conditionals, parallel steps, and environment variable threading.

**Official docs:** https://docs.openclaw.ai/tools/lobster

**npm package:** `@clawdbot/lobster` (version 2026.1.24)

**In-repo workflows:** `intake/workflows/*.lobster.yaml` (8 files, ~6500 lines total)

The key workflow files, in execution order:

1. `pipeline.lobster.yaml` — top-level orchestrator (PRD → tasks → artifacts)
2. `codebase-analysis.lobster.yaml` — optional repo analysis preprocessing
3. `deliberation.lobster.yaml` — architecture deliberation phase
4. `design-deliberation.lobster.yaml` — design brief generation
5. `intake.lobster.yaml` — main task generation pipeline
6. `task-refinement.lobster.yaml` — task expansion and refinement
7. `voting.lobster.yaml` — multi-model voting on task quality
8. `decision-voting.lobster.yaml` — committee voting mechanism

### Infrastructure (Familiar Territory)

You'll feel at home here. The infra stack is:

- **Helm charts:** `charts/` — cto, openclaw-agent, tenant-agents, argocd, argo-workflows, buildkit, gitlab, twingate-operator, etc.
- **GitOps:** `infra/gitops/` — ArgoCD application definitions
- **Talos Linux:** `crates/talos/` — OS-level config for bare metal nodes
- **Desktop app:** Tauri-based CTO App that runs a Kind cluster locally with agent pods + Cloudflared tunnel

## Key Files to Start With

| Priority | File | Why |
|----------|------|-----|
| 1 | `docs/intake-lobster-openclaw-process.md` | How all the pieces connect — the architectural Rosetta Stone |
| 2 | `cto-config.json` | Master config: agents, models, tools, skills |
| 3 | `docs/openclaw-local-setup.md` | Get a local gateway running |
| 4 | `intake/workflows/pipeline.lobster.yaml` | Top-level orchestrator — trace the execution flow |
| 5 | `charts/` | Helm charts — start contributing here |
| 6 | `crates/controller/` | The Rust brain — CRD lifecycle, pod spawning |
| 7 | `infra/gitops/` | ArgoCD app definitions |

## Suggested Onboarding Path

1. **Read** `docs/intake-lobster-openclaw-process.md` to understand the architecture
2. **Read** `cto-config.json` to see how agents are configured
3. **Set up** a local OpenClaw gateway using `docs/openclaw-local-setup.md`
4. **Run** `intake/scripts/verify-lobster-openclaw.sh` to validate your environment
5. **Trace** a workflow by reading `pipeline.lobster.yaml` top to bottom
6. **Explore** `charts/` and `infra/gitops/` — familiar ground to build confidence
7. **Pick up** an infra task from the outstanding work list above
