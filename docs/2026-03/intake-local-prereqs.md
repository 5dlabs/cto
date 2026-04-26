# Local prerequisites: sigma-1 / Lobster intake pipeline

Short checklist before running [`pipeline.lobster.yaml`](../../intake/workflows/pipeline.lobster.yaml) via OpenClaw (e.g. first [`5dlabs/sigma-1`](https://github.com/5dlabs/sigma-1) test). See also [`intake-lobster-openclaw-process.md`](intake-lobster-openclaw-process.md).

---

## Workspace and repos

| Requirement | Notes |
|-------------|--------|
| **`WORKSPACE`** | Export to the **CTO** checkout path so `load-config` / `discover-tools` resolve [`cto-config.json`](../../cto-config.json). Example: `export WORKSPACE=/path/to/cto`. |
| **Workflow cwd** | Run `openclaw.invoke` from a context where `intake/workflows/pipeline.lobster.yaml` is found (usually CTO repo or configured workflow root). |
| **`repository_url` + `project_name`** | For sigma-1: set `repository_url` to that GitHub URL and a short `project_name` for branches/PR titles. |

**PRD body:** Prefer combining `prd.txt` with `architecture.md` from the target repo (concatenate markdown) so deliberation/intake match Morgan’s optional architecture block.

---

## Local 1Password env (automatic `op run`)

**Why it ever looked “human”:** 1Password cannot infer *which* item is “the Linear key for CTO”—that’s **org mapping**, not crypto. Only **someone with context** can point `op://Vault/Item/field` at the right row once. **`op://` references are not secret values** (they’re stable pointers), so that mapping can live in git.

**Default for the team (no per-developer `cp`):**

1. Maintain **[`intake/local.env.op.defaults`](../../intake/local.env.op.defaults)** only as a temporary local fallback while intake bootstrap migrates. The preferred path is now: mint a runtime token via PM from the per-agent item (for example **Linear Morgan OAuth** `client_id` / `client_secret`) and read the runtime token from Kubernetes.
2. On first preflight/checkpoints run, **[`intake/scripts/ensure-local-env-op.sh`](../../intake/scripts/ensure-local-env-op.sh)** copies defaults → gitignored **`intake/local.env.op`** if the latter is missing.
3. **[`intake/scripts/intake-op-auto.sh`](../../intake/scripts/intake-op-auto.sh)** then **`op run`** as before.

**Solo / ad hoc:** set **`INTAKE_BOOTSTRAP_LINEAR_OP_REF=op://…`** once, or copy [`intake/local.env.op.example`](../../intake/local.env.op.example).

When **`intake/local.env.op`** exists and the **1Password CLI** (`op`) is on your PATH, these scripts **re-exec once** under `op run --env-file=…` so secrets resolve **without** you prefixing every command:

| Script | Effect |
|--------|--------|
| [`intake/scripts/pipeline-preflight.sh`](../../intake/scripts/pipeline-preflight.sh) | Injected env before bridge/Linear/kubectl checks |
| [`intake/scripts/iteration-checkpoints.sh`](../../intake/scripts/iteration-checkpoints.sh) | Same, then Linear GraphQL viewer check |

Implementation: [`intake/scripts/intake-op-auto.sh`](../../intake/scripts/intake-op-auto.sh) (sourced by the scripts above).

| Override | Meaning |
|----------|---------|
| **`INTAKE_OP_ENV_FILE`** | Use another env file path instead of `intake/local.env.op` |
| **`INTAKE_OP_AUTO_DISABLE=1`** | Never auto-wrap (e.g. CI, or debugging raw env) |
| **`INTAKE_OP_WRAPPED=1`** | Already running under `op run` — do not wrap again (set automatically after wrap; **`go-green-loop.sh`** sets it when you pass **`INTAKE_OP_ENV_FILE`**) |
| **`INTAKE_OP_BOOTSTRAP_DISABLE=1`** | Do not auto-create `local.env.op` from defaults |
| **`INTAKE_BOOTSTRAP_LINEAR_OP_REF`** | If no defaults file, write this single `op://…` into `local.env.op` when missing |

Long-lived processes (**OpenClaw gateway**, etc.) still need to be **started** under `op run` (or export vars in that shell) so the **running** process sees `LINEAR_API_KEY`; the auto-wrap above fixes **checkpoints and preflight** invoked from your repo checkout.

---

## Preflight, bridges, and Twingate (before `pipeline.lobster.yaml`)

The pipeline runs **`intake/scripts/pipeline-preflight.sh`** immediately after `load-config`. It fails fast unless:

| Check | Why |
|--------|-----|
| **`LINEAR_API_KEY`** | Non-empty runtime token (prefer PM-minted Kubernetes token; `lin_api_…` still works as a fallback). |
| **`DISCORD_BRIDGE_URL/health`** | HTTP 200 (default base `http://discord-bridge.bots.svc:3200` — override when not on-cluster). |
| **`LINEAR_BRIDGE_URL/health`** | HTTP 200 (default `http://linear-bridge.bots.svc:3100`). |
| **`kubectl cluster-info`** | Reaches the **CTO** API server (`INTAKE_PREFLIGHT_KUBECTL_SKIP=true` only if you accept missing cluster context). |

**Twingate (or other Zero Trust):** From a laptop, in-cluster hostnames will not resolve until connectors and resource definitions expose the bridge services and Kubernetes API. Set:

- **`DISCORD_BRIDGE_URL`** — e.g. `https://discord-bridge.<your-internal-dns>` (no trailing slash).
- **`LINEAR_BRIDGE_URL`** — same pattern for linear-bridge.
- **`KUBECONFIG`** / context pointed at the cluster reachable via Twingate.

Then from the same shell you will run `lobster` / OpenClaw:

```bash
curl -sf --max-time 10 "${DISCORD_BRIDGE_URL%/}/health"
curl -sf --max-time 10 "${LINEAR_BRIDGE_URL%/}/health"
kubectl cluster-info --request-timeout=15s
```

**Escape hatch (dev only):** `INTAKE_PREFLIGHT_SKIP=true` skips all checks — not for production intake.

**Bridge-only skip (laptop without routable bridge URLs):** `INTAKE_PREFLIGHT_BRIDGES_SKIP=true` skips **only** the Discord + Linear bridge `/health` curls; `LINEAR_API_KEY` and `kubectl` checks still run (unless you also set `INTAKE_PREFLIGHT_KUBECTL_SKIP`). Pipeline steps that call `bridge-notify` still need reachable bridge URLs at runtime.

**One-shot local check:** from repo root, `WORKSPACE=$PWD ./intake/scripts/go-green.sh` (full preflight + Linear viewer). Use `./intake/scripts/go-green.sh --bridges-skip` when bridges are not on localhost/Twingate yet but you want **preflight (minus bridge HTTP) + Linear GraphQL** green.

**Retry loop (e.g. while you fix Twingate / `op` / port-forward):** `WORKSPACE=$PWD ./intake/scripts/go-green-loop.sh` — one `feedback-loop-signal start`, then repeats **`iteration-checkpoints.sh`** only (avoids spamming Discord each retry). Tune with **`INTAKE_GO_GREEN_MAX_ATTEMPTS`** (default 60), **`INTAKE_GO_GREEN_SLEEP_SEC`** (default 15), **`INTAKE_GO_GREEN_FOREVER=1`**, and **`INTAKE_OP_ENV_FILE=/path/to.env`** for **`op run`** each attempt. Same flags as one-shot: `--bridges-skip`, `--max-attempts N`, `--sleep SEC`.

**Discord “starting intake” message:** After preflight succeeds, the pipeline sends **`notify-pipeline-start`** via `intake-util bridge-notify --from intake --to deliberation` so the same discord-bridge routing as deliberation shows a **pipeline kickoff** embed (even when `deliberate=false`).

**Timeouts:** `INTAKE_PREFLIGHT_CURL_TIMEOUT` (seconds, default `10`).

**Local both bridges (OAuth + team id + OpenClaw Discord token):**

1. Ensure **`intake/local.env.op`** exists (from **`local.env.op.defaults`**) with:
   - **`LINEAR_API_KEY`** → runtime Linear token. Prefer a PM-minted token sourced from Kubernetes; the `developer_token` pointer is only a temporary local fallback while intake bootstrap is migrated.
   - **`DISCORD_BRIDGE_TOKEN`** → **`op://Automation/OpenClaw Discord Tokens/DISCORD_TOKEN_INTAKE`** (intake bot).
2. **`LINEAR_TEAM_ID`** is **not** required in the file for local runs: **`intake/scripts/linear-resolve-team-id.sh`** resolves **`defaults.linear.teamId`** in **`cto-config.json`** (e.g. `CTOPA`) to a UUID via GraphQL (same as finding the team in the Linear UI).
3. Run **`./intake/scripts/run-local-bridges.sh`** — starts both bridges ( **`ACP_ACTIVITY_ENABLED=false`** by default). **One terminal:** logs go to **`intake/.bridge-logs/`** and the script runs **`tail -f`** on both files so you monitor Discord + Linear together. Use **`--detach`** to spawn in the background and print the `tail -f` command; **`--no-tail`** if you only want the processes and will read logs elsewhere. Then export **`DISCORD_BRIDGE_URL`** / **`LINEAR_BRIDGE_URL`** to **`http://127.0.0.1:3200`** / **`:3100`** and run **`go-green.sh`** without **`--bridges-skip`**.

For **Discord-visible iteration** (snapshot → fix → re-run checkpoints), see [`intake-discord-feedback-loop.md`](intake-discord-feedback-loop.md) and [`intake-coordinator.md`](intake-coordinator.md). **Logs + Discord + Linear together:** [`intake-observer.md`](intake-observer.md). Terminal checks 2–3: **`intake/scripts/iteration-checkpoints.sh`**. Emergency / loop-state alerts: **`intake/scripts/feedback-loop-signal.sh`** (`start` | `broken` | `waiting` | `show` | `clear`) and **`intake/scripts/coordinator-speak.sh`** for a one-off line. State file: **`intake/.feedback-loop-state.json`** (gitignored).

---

## OpenClaw / intake agent

- Local OpenClaw gateway and **`intake`** agent with **`lobster`** + **`llm-task`** enabled (see [`intake/config/openclaw-llm-task.json`](../../intake/config/openclaw-llm-task.json)).
- Details: [`../2026-02/openclaw-local-setup.md`](../2026-02/openclaw-local-setup.md).
- **Node ≥ 22**; **OpenClaw** current stable (or dev channel if you need it); **`@clawdbot/lobster`** on PATH on the gateway host (often **`2026.1.24`** from npm `@latest` — verify with `lobster version` vs `openclaw --version`). See §4a in [`intake-lobster-openclaw-process.md`](intake-lobster-openclaw-process.md).

---

## `intake-util` binary (PATH)

Workflow steps call **`intake-util`** on `PATH`. A stale copy (e.g. `~/bin/intake-util`) may be **older than** [`apps/intake-util`](../../apps/intake-util) in this repo and **miss** newer flags or behaviors (for example `INTAKE_REGISTER_RUN_SKIP` handling in [`run-registry-client.ts`](../../apps/intake-util/src/run-registry-client.ts)).

| Approach | Notes |
|-----------|--------|
| **Prefer repo build** | From `apps/intake-util`: `bun install` (if needed) then **`bun run build`** — produces `./intake-util` in that directory. |
| **PATH** | Prepend **`/path/to/cto/apps/intake-util`** so `lobster`-spawned shells resolve the freshly built CLI. |
| **Verify** | `which intake-util` and test skip: `INTAKE_REGISTER_RUN_SKIP=true intake-util register-run --run-id test --agent intake` should exit **0** and print a skip warning on stderr. |

---

## Linear (Phase 0: `create-linear-project`)

| Variable / config | Required | Notes |
|-------------------|----------|--------|
| **`LINEAR_API_KEY`** | **Yes** for `intake-util sync-linear init` / project creation | Must be visible in the **same environment** as the OpenClaw gateway / `lobster` process (e.g. inject with **1Password** `op run --env-file=…` when starting the gateway, or export in the parent shell). **Naming:** historically `LINEAR_API_KEY`; value may be a Linear **personal API key** (`lin_api_…`) **or** an **OAuth access token** (see header logic in [`sync-linear.ts`](../../apps/intake-util/src/sync-linear.ts): `lin_api_` prefix is sent as-is; otherwise `Authorization: Bearer …`). Use the **access token**, not only the OAuth client secret. Tokens can **expire**; this repo does not refresh OAuth for you. |
| **`defaults.linear.teamId`** in `cto-config.json` | Yes (via `load-config`) | Passed as `--team-id` to `sync-linear init`. Value may be a Linear **team key** (e.g. `CTOPA`) **or** a team **UUID**; `intake-util` resolves keys via the API (`resolveTeamId` in `apps/intake-util/src/sync-linear.ts`). If the key is wrong, the CLI errors with available teams. |

---

## linear-bridge (`register-run` / `deregister-run`)

Default URL is in-cluster: `http://linear-bridge.bots.svc:3100`. That **will not resolve** on a laptop unless you tunnel or override.

**Verify the bridge:** build, unit tests, and HTTP smoke — see [`linear-bridge-verify.md`](linear-bridge-verify.md) and `scripts/2026-03/verify-linear-bridge.sh`.

| Approach | When to use |
|----------|-------------|
| **`LINEAR_BRIDGE_URL`** | Point at a reachable endpoint: e.g. `kubectl port-forward svc/linear-bridge -n bots 3100:3100` then `export LINEAR_BRIDGE_URL=http://127.0.0.1:3100`. |
| **`INTAKE_REGISTER_RUN_SKIP=true`** (or `1`) | Local dev only: **`register-run` exits 0** with a **warning**; the run **does not** appear in the bridge registry. Use when you are not testing bridge integration. |
| **`INTAKE_DEREGISTER_RUN_SKIP=true`** | Same idea for **`deregister-run`** so teardown does not fail if the bridge is unreachable. |

**Policy:** For a **real** sigma-1 integration test that expects registry visibility, use **tunnel + `LINEAR_BRIDGE_URL`**. Use **skip flags** only when you explicitly accept missing registry rows.

CLI shape (after fixes):

```bash
intake-util register-run --run-id my-run --agent intake [--linear-session-id …] [--issue-id …]
```

Successful stdout includes **`run_id`** for Lobster `jq` steps.

---

## Kubernetes / Argo CD (`build-infra-context`)

**Which cluster?** If you have **more than one** kubeconfig context, point **`kubectl`** at the **OVH cluster**—the environment that actually runs **CTO platform components** and the **OpenClaw agents** (operators, services, and MCP inventory the pipeline queries). A random dev or staging cluster will run the steps but the **context text will be wrong** for CTO.

| Requirement | Notes |
|-------------|--------|
| **`kubectl`** | Use the **OVH / CTO** context (`kubectl config get-contexts`, `kubectl config use-context …`). `build-infra-context` runs `kubectl cluster-info` and fails if there is no reachable API; `discover-tools` also queries that cluster. |
| **Argo CD CLI** | **Optional.** If `argocd` is missing, **`ARGOCD_SERVER` unset**, or login fails, the workflow prints an **ArgoCD Applications** stub line instead of failing the whole step. For real Argo output against the same environment, set **`ARGOCD_SERVER`** to the Argo CD instance that manages **that** OVH cluster and run **`argocd login`**. |

---

## Optional first-run scope

To reduce moving parts on the first invocation:

- Set **`deliberate`: `false`** if you are not testing deliberation/NATS yet.
- Set **`include_codebase`: `true`** only when Repomix/OctoCode paths are ready for the target repo.

---

## Lobster + OpenClaw + `llm-task` check

On the **gateway host** (same machine that runs OpenClaw and invokes workflows):

```bash
./intake/scripts/verify-lobster-openclaw.sh
```

This asserts **Node ≥ 22**, **`openclaw`** and **`lobster`** on PATH, prints npm’s published **`@clawdbot/lobster`** version, and validates [`intake/config/openclaw-llm-task.json`](../../intake/config/openclaw-llm-task.json) (**`llm-task`** enabled + **`intake`** allowlist per [LLM Task](https://docs.openclaw.ai/tools/llm-task)). It does **not** verify `intake-util`; use **`which intake-util`** and the table in **§ `intake-util` binary** above.

Install / refresh CLIs:

```bash
npm install -g openclaw@latest @clawdbot/lobster@latest
```

**Note:** **`lobster --version`** (npm **`@clawdbot/lobster`**) is often **`2026.1.x`** while **`openclaw --version`** is **`2026.3.x`** — different package versions; keep both current via npm and follow [Lobster docs](https://docs.openclaw.ai/tools/lobster) if OpenClaw starts bundling a different binary.

---

## Quick preflight commands

```bash
# From CTO repo, with bridge reachable OR skip set:
export WORKSPACE="$(pwd)"
export LINEAR_API_KEY=...   # if exercising Linear
# export LINEAR_BRIDGE_URL=http://127.0.0.1:3100
# export INTAKE_REGISTER_RUN_SKIP=true   # if no bridge

intake-util register-run --run-id preflight --agent intake --linear-session-id test-session
```

Then run your `openclaw.invoke --workflow intake/workflows/pipeline.lobster.yaml` (or equivalent path) with JSON inputs for sigma-1.
