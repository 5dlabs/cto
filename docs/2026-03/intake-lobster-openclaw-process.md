# Intake process: Lobster, OpenClaw, and chain of command

This document summarizes how the **intake pipeline** is supposed to run, with emphasis on **Lobster workflows**, **OpenClaw**, and **who actually executes** the workflow files. It is written for review (e.g. toward repos such as [`5dlabs/sigma-1`](https://github.com/5dlabs/sigma-1)).

---

## 0. Current testing scope (explicit)

**In scope right now:** validating **Lobster** workflows executed by **OpenClaw running locally** on a developer machine—same workflow YAML under `intake/workflows/`, same `intake` agent tool allowlist (`lobster`, `llm-task`), **without** requiring a cluster **CodeRun** or Morgan’s shell template.

**Not in scope yet (for this testing phase):** end-to-end **Morgan CodeRun** (Linear → PM → `intake.sh.hbs` → clone → `openclaw.invoke`). That path is the intended **production** trigger; it should match the **same** `pipeline.lobster.yaml` inputs once you wire it, but it is **not** what we’re proving when we say “testing locally.”

---

## 1. Goals in one sentence

**Turn a PRD (plus optional architecture and codebase context) into a structured task breakdown, per-task docs/prompts, commits, and a PR—using a single declarative Lobster pipeline executed by OpenClaw, with the `intake` agent as the tool-capable runtime for Lobster + LLM steps.**

---

## 2. Chain of command

### 2a. Local testing (what we’re doing now)

| Layer | Role |
|--------|------|
| **Human** | Runs OpenClaw gateway locally, points workspace at a checkout (e.g. sigma-1), supplies PRD text and flags as JSON stdin to `openclaw.invoke`. |
| **`openclaw.invoke`** | Starts `pipeline.lobster.yaml` with that JSON (from repo root or configured workflow path). See [`docs/../2026-02/openclaw-local-setup.md`](../2026-02/openclaw-local-setup.md). |
| **`intake` OpenClaw agent** | Runtime identity allowed to run **Lobster** + **llm-task** per [`intake/config/openclaw-llm-task.json`](../../intake/config/openclaw-llm-task.json). |
| **Lobster** | Executes steps in `intake/workflows/*.yaml` (subprocesses, `llm-task`, `intake-util`, …). |
| **Helpers** | `intake-util`, `intake-agent`, local Discord bridge, etc., as invoked by workflow steps—only what the YAML needs on your machine. |

**Takeaway:** *Intake runs the Lobster file* means **OpenClaw’s `intake` agent + Lobster tool**, whether the shell that called `openclaw.invoke` is Morgan in CI or **you in a terminal**.

### 2b. Production (later): Morgan CodeRun

| Layer | Role |
|--------|------|
| **Morgan (`intake.sh.hbs`)** | **Bootstrap only**: repo prep, clone, PRD/arch staging, Linear metadata, logging. |
| **Then** | Same **`openclaw.invoke --workflow pipeline.lobster.yaml`** as in [`templates/agents/morgan/intake.sh.hbs`](../../templates/agents/morgan/intake.sh.hbs)—inputs built from the workspace. |

Morgan does **not** re-implement the pipeline in bash; it delegates to the **same Lobster graph** after setup.

---

## 3. Top-level workflow file

- **File:** [`intake/workflows/pipeline.lobster.yaml`](../../intake/workflows/pipeline.lobster.yaml)  
- **Name:** `pipeline`  
- **Inputs (conceptual):** PRD text, project name, task count, `deliberate`, `include_codebase`, repo URL/org, Linear metadata, base branch, optional infra context, etc.

**Phases (high level):**

1. **Config / environment** — Load model tiers and committee settings from `cto-config.json` (and related wiring in the YAML).
2. **Parallel setup** — Repo setup, Linear project/issue visibility, infra context, optional tool discovery, optional **codebase-analysis** when `include_codebase=true`.
3. **Deliberation (conditional)** — If `deliberate=true`: research + Optimist/Pessimist + committee voting + design brief; otherwise PRD flows straight to intake.
4. **Intake** — Invokes `intake.lobster.yaml`: parse PRD → complexity → approval gates as configured → expand/refine → multi-model voting → fan-out docs/prompts → validation → sync Linear → commit → open PR.

Exact step names live in the YAML; a diagram lives in [`templates/skills/workflow/intake-pipeline/SKILL.md`](../templates/skills/workflow/intake-pipeline/SKILL.md).

---

## 4. How OpenClaw and Lobster fit together

- **Lobster** plugin enabled in [`intake/config/openclaw-llm-task.json`](../../intake/config/openclaw-llm-task.json).
- **LLM steps** use `llm-task` with `intake/prompts/` and `intake/schemas/`.
- **Guardrails:** `allowedModels` in that JSON.

**Lobster = orchestration DSL; OpenClaw = host + tools; `intake` = principal that may call `lobster` and `llm-task`.**

### 4a. Pinned runtime (verify locally)

| Piece | Expectation |
|--------|-------------|
| **Node** | **≥ 22** (OpenClaw current trees). |
| **OpenClaw** | Install current stable globally, e.g. `npm install -g openclaw@latest`; optional `openclaw update --channel dev` for prerelease. This repo was smoke-tested with **OpenClaw 2026.3.13**. |
| **`lobster` CLI** | [`@clawdbot/lobster`](https://www.npmjs.com/package/@clawdbot/lobster) on the **same host** as the gateway (e.g. `npx @clawdbot/lobster` or a global install). **npm `@latest`** was **2026.1.24** — there is no newer standalone CLI on npm today; keep it aligned with whatever OpenClaw bundles or documents. |
| **Workflow file shape** | The parser in `@clawdbot/lobster` requires every step to have string **`id`** and **`command`**. Top-level **`inputs:`** lists are **not** substituted; use **`args:`** with `{ name: { default: … } }` and **`${arg_name}`** in commands. Sub-workflows are invoked with **`lobster run --mode tool "$WORKSPACE/.../child.lobster.yaml" --args-json "$(jq -n …)"`**. Prefer YAML literal **`command: \|`** (or explicit `\` continuation) for multi-line shell — folded **`command: >`** turns newlines into spaces and can break `python3 multiline -c`, line-broken CLIs, and `if`/`fi` blocks. |
| **Bulk migration** | [`intake/scripts/lobster_native_convert.py`](../../intake/scripts/lobster_native_convert.py) can migrate CTO-style templates (`${{ steps.* }}`, `workflow:` + `inputs:`) toward the native shape; **nested workflows** should pass prior-step JSON via **step `env:`** (e.g. `CTO_*`) so Lobster does not splice raw JSON into `/bin/sh -lc` (see [`pipeline.lobster.yaml`](../../intake/workflows/pipeline.lobster.yaml)). LLM steps should build `openclaw.invoke --args-json` with **`jq --rawfile`** from `$WORKSPACE/intake/prompts` (see [`voting.lobster.yaml`](../../intake/workflows/voting.lobster.yaml)). |
| **`intake-util` on PATH** | Prefer a **fresh** build from [`apps/intake-util`](../../apps/intake-util) (`bun run build`) and prepend that directory to `PATH` so shell steps do not pick up a stale global binary. See [`intake-local-prereqs.md`](intake-local-prereqs.md). |
| **Linear secrets** | `LINEAR_API_KEY` may be a **`lin_api_` key** or an **OAuth access token** (injected via 1Password, etc.); see [`sync-linear.ts`](../../apps/intake-util/src/sync-linear.ts) for the Authorization header rule. Ensure the gateway child inherits the variable. |

Official references: [Lobster / workflow files](https://docs.openclaw.ai/tools/lobster#workflow-files-lobster), [LLM task](https://docs.openclaw.ai/tools/llm-task).

**Quick verify:** run [`intake/scripts/verify-lobster-openclaw.sh`](../../intake/scripts/verify-lobster-openclaw.sh) on the gateway host (Node, OpenClaw, lobster PATH, and `openclaw-llm-task.json` shape).

---

## 5. Deliberation vs intake vs committee (conceptual)

- **Deliberation:** architecture-oriented debate and brief before task decomposition (when enabled).
- **Intake:** PRD → tasks → subtasks → artifacts.
- **Committee / voting:** multi-model quality gating on task output.

These are **stages inside** `pipeline.lobster.yaml`, not separate one-off scripts.

---

## 6. Local run vs sigma-1-shaped repo

**Environment and secrets:** see [`intake-local-prereqs.md`](intake-local-prereqs.md) (`WORKSPACE`, `LINEAR_API_KEY`, `LINEAR_BRIDGE_URL`, optional skip flags, Argo CD). For **`kubectl`** steps in the pipeline, use the **OVH cluster** context where CTO and OpenClaw agents run—not an arbitrary kube context.

For local testing toward [`5dlabs/sigma-1`](https://github.com/5dlabs/sigma-1):

1. Clone sigma-1 (or use mono-repo paths) so the workflow’s working directory and `repository_url` / PRD paths match what you pass in JSON.
2. Start **OpenClaw gateway** locally (see [`docs/../2026-02/openclaw-local-setup.md`](../2026-02/openclaw-local-setup.md)).
3. From the **CTO repo** (or wherever workflow files resolve), run **`openclaw.invoke --workflow pipeline.lobster.yaml`** with stdin JSON analogous to the `jq -n` block in Morgan’s template: at minimum `prd_content`, `project_name`, `num_tasks`, `deliberate`, `include_codebase`, `repository_url`, `pr_base_branch`, and optional `intake_metadata` / Linear fields if steps need them.
4. **Sigma-1 / full surface:** after the graph runs cleanly with `deliberate: false`, re-invoke the intake agent with **`deliberate: true`** (and the same PRD/repo inputs). Watch **#intake** / NATS / Discord if those bridges are enabled; expect longer runtimes and stricter env (see [`intake-local-prereqs.md`](intake-local-prereqs.md)).
5. Confirm artifacts under `.tasks/` (and any PR/commit steps your local git auth allows).

**Verify:** step completion, model allowlist, fan-out—**not** “did CodeRun schedule.”

---

## 7. What this document is *not* claiming

- It does not fully specify **ACP** vs **NATS** vs **Discord**—adjacent to orchestration.
- It does not replace reading **`pipeline.lobster.yaml`** and **`intake.lobster.yaml`** line by line.

---

## 8. Review checklist

**Local (current phase)**

- [ ] OpenClaw local config includes **`intake`** with **`lobster`** + **`llm-task`** (merge or point at `intake/config/openclaw-llm-task.json` as your source of truth).
- [ ] `pipeline.lobster.yaml` resolves from your cwd or OpenClaw workflow path.
- [ ] PRD + flags JSON matches what the workflow expects for sigma-1.
- [ ] **`kubectl`** context is the **OVH / CTO** cluster (see prereqs doc), if you rely on `build-infra-context` / `discover-tools`.
- [ ] Optional: deliberation / Discord / NATS only if you are testing those steps locally.

**Production (later)**

- [ ] Morgan template still ends with **`openclaw.invoke --workflow pipeline.lobster.yaml`** and equivalent inputs.
- [ ] Cluster/runtime injects the same agent tool policy for **`intake`**.

---

*Update when CodeRun comes into scope or local vs prod wiring changes.*
