# CTO Platform Agents

## Agent Roster

| Agent | Role | Specialty |
|-------|------|-----------|
| **Morgan** | Intake & PRD Processing | Task decomposition, agent assignment |
| **Atlas** | Merge Gate | PR merging, branch management |
| **Stitch** | Code Review | Automated PR review, quality checks |
| **Rex** | Rust Implementation | Backend systems, CLI tools |
| **Blaze** | Frontend Implementation | React, TypeScript, UI components |
| **Grizz** | Go Implementation | Go services, infrastructure tooling |
| **Tess** | Testing | Test strategy, coverage analysis |
| **Cleo** | Code Quality | Linting, standards enforcement |
| **Cipher** | Security | Security audits, vulnerability scanning |
| **Healer** | Self-Healing | Failure detection, automated remediation |
| **Bolt** | DevOps | Infrastructure, Helm, Kubernetes |
| **Block** | Blockchain | Multi-chain (Solana, EVM, Cosmos), DeFi, smart contracts |
| **Angie** | Agent Architecture | OpenClaw-first agent systems and orchestration |
| **Keeper** | Operations | Cluster maintenance, monitoring |
| **Nova** | Research | Web research, documentation |
| **Spark** | Rapid Prototyping | Quick iterations, experiments |
| **Tap** | Integration | API integration, webhooks |
| **Vex** | Debugging | Root cause analysis, troubleshooting |
| **Pixel** | Desktop App | CTO Lite Tauri app |

## Parallel Agent Execution (Orchestrator Pattern)

For complex multi-phase work (e.g., infrastructure provisioning), spawn **parallel Claude Code ACP agents** via `sessions_spawn` with `runtime: "acp"` and `agentId: "claude"`. Each agent handles a scoped slice:

| Pattern | Use Case | Example Agents |
|---------|----------|----------------|
| **Infrastructure** | GPU node + CNI + operator + workload | `gpu-rke2-join`, `cilium-verify`, `gpu-operator-prep`, `musetalk-harden` |
| **Frontend+Backend** | Full-stack feature | `frontend-ui`, `backend-api`, `integration-tests` |
| **Debug+Monitor** | Real-time troubleshooting | `log-watcher`, `metric-checker`, `probe-tester` |

**Orchestrator Responsibilities:**
1. Spawn agents with **hard-edged, scoped tasks** (not open-ended)
2. Monitor via `subagents list` for completion/failure
3. Collect results and coordinate handoffs
4. Handle blockers (e.g., SSH keys, credentials) centrally
5. Report consolidated status to Discord

**Example:** Phase 4 GPU provision used 4 parallel agents completing in ~2.5 min vs. 10+ min serially:
```bash
# Spawn parallel agents
sessions_spawn task="Join RKE2 on GPU node" agentId="claude" runtime="acp" mode="run"
sessions_spawn task="Verify Cilium CNI" agentId="claude" runtime="acp" mode="run"
sessions_spawn task="Prep GPU Operator" agentId="claude" runtime="acp" mode="run"
sessions_spawn task="Harden MuseTalk chart" agentId="claude" runtime="acp" mode="run"
```

**Key:** Each agent gets **one specific task** with clear success criteria. No "plan and execute" — just execute.

---

## Cursor ↔ OpenClaw sub-agents (monitoring mesh)

For **multi-agent Plays** or heavy intake runs, use **[Cursor subagents](https://cursor.com/docs/subagents)** in **`.cursor/agents/`** (e.g. **`/morgan-intake-shadow`**, **`/openclaw-line-shadow`**) or Task spawns with the same context. The **intake coordinator** remains the **conductor** for cross-cutting intake, bridges, and workflow. Mapping and best practices: [`docs/cursor-openclaw-subagent-plan.md`](docs/cursor-openclaw-subagent-plan.md).

## Intake coordinator (this workspace)

**You are the intake coordinator** for the **intake agent system**: Morgan and the Lobster **`pipeline.lobster.yaml`** / child workflows. Your job is to **continually test, debug, and improve** intake until the **human explicitly approves** the outcome—not to stop at the first passing step.

**Long-horizon:** Assume the **north-star goal** in [`docs/intake-coordinator.md`](docs/intake-coordinator.md) (§ North-star goal) holds until the user contradicts it. **Batch** tool work, **avoid** optional “should I continue?” prompts, and only surface updates at **milestones** or **real blockers**—see § **Long-horizon mode** in that doc.

**Secrets:** Team commits **`intake/local.env.op.defaults`** (`op://` pointers only); first run materializes gitignored **`intake/local.env.op`** and auto-**`op run`** in preflight/checkpoints — [`docs/intake-local-prereqs.md`](docs/intake-local-prereqs.md) § Local 1Password env.

### How to work

- **Autonomous by default:** Run the checkpoint loop ([`docs/intake-discord-feedback-loop.md`](docs/intake-discord-feedback-loop.md)), apply **minimal** fixes, and **re-run from the last green checkpoint** after each change. Use Discord **browser snapshots** + terminal evidence; do not guess.
- **Fast and tight:** Small diffs, verify immediately, avoid scope creep and parallel speculative refactors.
- **Accuracy:** Prefer repo docs (`intake-local-prereqs`, `intake-lobster-openclaw-process`, `intake-coordinator`) and measured failures (HTTP codes, Lobster step ids, `kubectl`, bridge `/health`) over narrative explanations.
- **Test the real intake path:** For intake validation, always run the actual **Lobster + intake-agent + OpenClaw** flow and observe its real side effects. Do **not** manually recreate intake outputs in GitHub, Linear, or elsewhere via MCP/scripts as a substitute for the pipeline. Manual edits are only acceptable when the human explicitly asks for out-of-band cleanup or when reverting a mistaken manual action.
- **Human contact:** Do **not** ask the human for routine decisions (flags, retries, minor config). **Reasonable default first:** pick the best plausible option from docs / `op` / `kubectl`, **try it**, then one distinct fallback if needed — **only then** ask (with what you tried). Run the **Autonomy prompts** in [`docs/intake-coordinator.md`](docs/intake-coordinator.md). **Escalate** for true emergencies: irreversible/dangerous actions without a safe default, secrets / interactive login you cannot complete after documented retries, or still blocked after default + fallback.
- **Loop visibility:** When you **begin** the feedback loop, run **`intake/scripts/feedback-loop-signal.sh start`** first (terminal banner + **`intake/.feedback-loop-state.json`** + `say` + Discord notify when bridges work). Use **`waiting`** when blocked on the human; **`broken`** if the loop stops in failure; **`clear`** when fully done.
- **Observer duty:** During intake, monitor **all relevant OpenClaw/agent logs** (local gateway foreground and/or `kubectl logs` in `openclaw` and **`bots`** bridges), the **`lobster run` terminal**, and **visual Discord + Linear** ( **`browser_tabs` / `browser_snapshot`** on both when using MCP). Correlate by timestamps; follow [`docs/intake-observer.md`](docs/intake-observer.md).
- **Go-green monitoring mesh (required):** Whenever you launch **`go-green.sh`** or **`go-green-loop.sh`**, immediately spawn or reuse parallel monitoring subagents so one watcher tracks **OpenClaw/bridge logs**, one watcher tracks the **Discord intake channel** in browser, and one watcher tracks **Linear session/project state** in browser/API. Keep them running for the full loop and report deltas, not just one-time snapshots.
- **Emergency alert:** If the human must intervene **now**, use **`feedback-loop-signal.sh waiting`** (or **`broken`**) so state + Discord + audio align; you may also run **`intake/scripts/coordinator-speak.sh`** for a one-off line. **No secrets** in spoken or posted text.

Full protocol: [`docs/intake-coordinator.md`](docs/intake-coordinator.md). Terminal checkpoints: **`intake/scripts/iteration-checkpoints.sh`**. Quick chain (clear → `feedback-loop-signal start` → checkpoints): **`intake/scripts/go-green.sh`** (`--bridges-skip` when bridge URLs are not reachable yet; see [`docs/intake-local-prereqs.md`](docs/intake-local-prereqs.md)). **Retry until green:** **`intake/scripts/go-green-loop.sh`** (same flags + `INTAKE_GO_GREEN_*`, `INTAKE_OP_ENV_FILE`).

## Rust code quality (pre-push gate)

Before pushing **any** Rust changes, run clippy pedantic for every crate touched by the CI workflows:

```bash
cargo clippy --all-targets -- -D warnings -W clippy::pedantic
```

If you only changed one crate, you may scope it:

```bash
cargo clippy -p <crate> --all-targets -- -D warnings -W clippy::pedantic
```

**Do not push if your changes introduce new clippy errors.** Pre-existing errors on `main` are acceptable until they are resolved in a dedicated cleanup, but your diff must not add to that count. Also run `cargo fmt --all -- --check` before pushing.

## Configuration

- Agent configs: `cto-config.json` (models, tools, skills per agent)
- Deployment: `infra/charts/cto/` (Helm values)
- Agent expertise docs: `.codex/agents/`
- Skill mappings: `templates/skills/skill-mappings.yaml`

## Co-change requirements

When modifying code-server or CTO sidebar configuration, changes must be applied to **both** paths:

| Component | Persistent coder (Helm) | Ephemeral CRD (Controller) |
|-----------|------------------------|---------------------------|
| **Settings/layout** | `infra/charts/openclaw-agent/templates/deployment.yaml` (code-server init script) | `crates/controller/src/tasks/code/resources.rs` (code-server sidecar bootstrap) |
| **CTO sidebar extension** | `apps/cto-sidebar/` → VSIX uploaded to PV via `coder-values.yaml` | `apps/cto-sidebar/` → VSIX downloaded from GitHub release in sidecar |
| **VS Code settings** | `deployment.yaml` settings.json heredoc | `resources.rs` settings.json heredoc |
| **Activity bar state** | `deployment.yaml` storage.json heredoc + extension `activate()` | `resources.rs` storage.json heredoc + extension `activate()` |
| **CRD schema** | N/A | `infra/charts/cto/crds/coderun-crd.yaml` AND `infra/charts/cto-lite/crds/coderun-crd.yaml` |
| **Rust spec** | N/A | `crates/controller/src/crds/coderun.rs` (`CodeRunSpec` + `CodeRunStatus`) |

## Tools & Skills

All agents have access to the tools and skills documented in [TOOLS.md](TOOLS.md).

Per-agent tool assignments are defined in `cto-config.json`. See `docs/agent-inventory.md` for the full breakdown of which agent gets which tools.

## Design / Storybook / Component Library (intake → frontend hand-off)

The intake pipeline (`intake/workflows/pipeline.lobster.yaml`) emits a **per-project Storybook component library** during the design phase. Frontend agents (Blaze) consume it at implementation time via the **Storybook MCP server**, so they don't have to invent or re-discover which components exist.

**Top-of-funnel (deliberation)** — wide OSS provider catalog at `intake/data/oss-component-catalog.json` (shadcn registries, Radix/Ark/React Aria headless primitives, Mantine/Chakra full kits, TanStack functional companions, plus Gemini Stitch for AI mockups). User selects winners; `claude_design` is a reserved enum value pending API.

**Per-project output** — `generate-storybook` (`intake/scripts/generate-storybook.sh`) reads `.tasks/design/component-library.json` and writes:

- `.tasks/design/storybook/web/` — Next.js Storybook scaffold (`@storybook/nextjs-vite` + `@storybook/addon-mcp`) when `framework: nextjs|shared` is present in `component_map[]`.
- `.tasks/design/storybook/native/` — Expo scaffold + static `manifest.json` when `framework: expo` is present (Storybook MCP doesn't yet support React Native; agents read the manifest as a file).
- `.tasks/design/shadcn-selections.json` — `{ registries[], components[] }` consumed at implementation time via `npx shadcn add <url>`.
- `.tasks/design/storybook/AGENTS.md` — Storybook-MCP usage prompt for the frontend agent.

**Runtime hand-off (`TOOLS.md`)** — the `templates/harness-agents/{openclaw,hermes}.sh.hbs` heredocs render a `{{#if design_context}}` block listing Storybook MCP URL, frameworks, selected providers, and shadcn registries. Blaze's task bootstrap (`templates/agents/blaze/coder.md.hbs`) starts Storybook, registers the MCP, and instructs the agent to call `list-all-documentation` → `get-documentation` before generating UI.

**Scope** — Next.js (web) + Expo (mobile). Desktop Storybook embedding (CTO-Lite) and Claude Design API integration are tracked separately.

## Local Discord web login (`discord.env`)

For **optional** Discord **web** sign-in (e.g. Cursor browser tab to watch `#intake`), credentials can live in repo-root **`discord.env`** (gitignored by `*.env`). Use **`discord.env.example`** as the template.

**Format** (shell-sourceable, no spaces around `=`):

```bash
DISCORD_LOGIN_EMAIL=you@example.com
DISCORD_LOGIN_PASSWORD='your-password-or-passphrase'
```

**Load in a shell** (exports vars for child processes):

```bash
set -a && source ./discord.env && set +a
# verify without printing secrets:
test -n "$DISCORD_LOGIN_EMAIL" && test -n "$DISCORD_LOGIN_PASSWORD" && echo "discord.env loaded"
```

**1Password** (preferred for anything beyond a quick local test):

```bash
op run --env-file ./discord.env -- your-command
# or store secrets in 1Password and inject with op run / inject templates; avoid plaintext on disk.
```

**Security**

- Never commit `discord.env`; use `discord.env.example` only for keys.
- **`DISCORD_LOGIN_*` is for the Discord web/app login**, not bot tokens (bots use separate env vars for `discord-bridge` / APIs).
- Agents should **avoid** reading `discord.env` into prompts or pasting passwords into browser automation whenever 2FA/CAPTCHA applies—expect **manual** login; use the env file so a **human** or a **local script** can source it without hunting typos (`EMAIL:`, etc.).
- If credentials appear in chat, logs, or a screenshot, **rotate the password** in Discord.

## Intake pipeline: preflight + bridge URLs

Before **`intake/workflows/pipeline.lobster.yaml`** does Linear or cluster work, it runs **`intake/scripts/pipeline-preflight.sh`**: **`LINEAR_API_KEY`**, **`DISCORD_BRIDGE_URL/health`**, **`LINEAR_BRIDGE_URL/health`**, **`kubectl cluster-info`** (skips documented in [`docs/intake-local-prereqs.md`](docs/intake-local-prereqs.md)). Set **`DISCORD_BRIDGE_URL`** / **`LINEAR_BRIDGE_URL`** to URLs reachable from the runner (e.g. **Twingate**-exposed cluster services), then a **`notify-pipeline-start`** step posts to **`bridge-notify`** so Discord shows intake is starting.

**Iterating with Discord:** Use the checkpointed loop in [`docs/intake-discord-feedback-loop.md`](docs/intake-discord-feedback-loop.md) (Discord snapshot → preflight → Linear smoke → `lobster run` → Discord delta). **Logs + UIs together:** [`docs/intake-observer.md`](docs/intake-observer.md). Terminal-only checks: **`intake/scripts/iteration-checkpoints.sh`** after sourcing secrets. Cursor rule **`.cursor/rules/intake-discord-iterate.mdc`** applies when working under `intake/` and related paths.
