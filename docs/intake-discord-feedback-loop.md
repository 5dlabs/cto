# Intake pipeline ↔ Discord: iterative fix loop

This is the **operator protocol** for running the Lobster intake pipeline while **watching the intake Discord surface** (Cursor browser or desktop app). The agent should treat Discord as a **primary signal** alongside terminal output, then **fix → re-run from the last green checkpoint**—not guess from memory.

**Full observation** also includes **Linear’s UI** and **OpenClaw/bridge logs**; see [`intake-observer.md`](intake-observer.md).

---

## Loop status: you should always “feel” the state

The coordinator is **not** “running” in the background silently. When the feedback loop is active, **signal state** so the human can tell **started vs broken vs waiting** without reading the whole chat.

| Phase | Command (from CTO repo root) | What fires |
|--------|------------------------------|------------|
| **Loop started** | `./intake/scripts/feedback-loop-signal.sh start [--message "…"]` | **Terminal banner** (bold), **`intake/.feedback-loop-state.json`** (`status: running`), **macOS `say`** (or `spd-say` / stderr), **`bridge-notify`** to Discord (if `intake-util` + bridges reachable; set `INTAKE_LOOP_NO_DISCORD=1` to skip). |
| **Loop broken** | `./intake/scripts/feedback-loop-signal.sh broken [--message "…"]` | Same surfaces with `status: broken` — unrecoverable error or coordinator giving up after bounded tries. |
| **Waiting on human** | `./intake/scripts/feedback-loop-signal.sh waiting [--message "…"]` | Same surfaces with `status: awaiting_human` — approval gate, MFA you must complete, or emergency. |
| **Show / clear** | `./intake/scripts/feedback-loop-signal.sh show` / `clear` | Print JSON state or remove the file. |
| **Mechanical retry loop** | `./intake/scripts/go-green-loop.sh` [`--bridges-skip`] | **One** `start` signal, then repeats **`iteration-checkpoints.sh`** until green or max attempts (`INTAKE_GO_GREEN_*`, `INTAKE_OP_ENV_FILE` — see [`intake-local-prereqs.md`](intake-local-prereqs.md)). |

**Visual without Discord:** open **`intake/.feedback-loop-state.json`** in the IDE (updated on each transition). **Audio:** disable with `INTAKE_LOOP_NO_SPEAK=1` if `say` is noisy.

**Rule:** Call **`feedback-loop-signal.sh start` as the first action** when entering the feedback loop for a session; call **`waiting`** before blocking on human approval; call **`broken`** when stopping unsuccessfully; **`clear`** when the session is done or you are fully idle.

---

## Roles

| Role | Responsibility |
|------|----------------|
| **Intake coordinator (agent)** | Owns the full loop: autonomous fixes, tight re-runs, **no routine human pings**—see [`intake-coordinator.md`](intake-coordinator.md). Emergency only: **`intake/scripts/coordinator-speak.sh`**. |
| **Human** | **Approval gate:** signals satisfaction / ship decision. Also: MFA / captcha / Twingate when automation cannot; explicit OK on destructive or irreversible actions. |

---

## Checkpoints (strict order)

Re-run **from checkpoint 1** after any change to **kube context, bridge URLs, secrets, or preflight script**. After a **pure workflow YAML** fix inside a child graph, you may restart from **checkpoint 4** if checkpoints 1–3 were green in this session.

| # | Check | Command / action | Pass criteria |
|---|--------|------------------|---------------|
| 1 | **Discord visual baseline** | `browser_tabs` → select Discord tab; `browser_snapshot` on the intake-related channel (or the bridge room that mirrors intake). | Snapshot succeeds; note **latest embed/text** and timestamp if visible. |
| 2 | **Bridge + kube preflight** | `WORKSPACE` set to CTO root; run `intake/scripts/pipeline-preflight.sh`. | Exit 0; stderr ends with `preflight OK`. |
| 3 | **Linear token smoke** | `curl` GraphQL `viewer { id }` with `Authorization: Bearer $LINEAR_API_KEY` (never print the key). | HTTP  **200** and JSON without `AUTHENTICATION_ERROR`. |
| 4 | **Pipeline** | `lobster run --mode tool --file intake/workflows/pipeline.lobster.yaml --args-json '…'` with real args (see [`intake-local-prereqs.md`](intake-local-prereqs.md)). | Lobster envelope `ok: true`, or capture failing **step id** and stderr. |
| 5 | **Discord visual delta** | Snapshot Discord again **after** the run (wait a few seconds for bridge latency). | New message or embed for `pipeline-start` / deliberation / errors, consistent with terminal outcome. |

**Mismatch protocol:** If the terminal says the pipeline failed but Discord shows a “starting” success, treat **trust order** as: terminal + Lobster step id first; then investigate **bridge delivery** (discord-bridge logs, wrong `DISCORD_BRIDGE_URL`). If Discord is silent but the pipeline advanced past `notify-pipeline-start`, treat **notify** as best-effort and verify `bridge-notify` stdout JSON (`discord: true`).

---

## Failure → remediation → resume

1. **Classify** the failure:
   - **Infra:** preflight, `kubectl`, `DISCORD_BRIDGE_URL` / `LINEAR_BRIDGE_URL`, Twingate, port-forward.
   - **Auth:** `LINEAR_API_KEY` 401, OAuth expiry, wrong token type.
   - **Workflow:** shell / `jq` / Lobster `env:` + `CTO_*` (see [`intake-lobster-openclaw-process.md`](intake-lobster-openclaw-process.md)).
2. **Minimal fix:** one logical change per iteration; no drive-by refactors.
3. **Resume** from the **first checkpoint** invalidated by that change (see table above).
4. **Log a one-line note** in the session (or PR description): what failed, what changed, which checkpoint was re-run.

---

## Browser MCP (when available)

1. `browser_tabs` with `action: "list"` — locate Discord tab.
2. `browser_lock` → `browser_snapshot` (and `take_screenshot_afterwards: true` if layout matters).
3. After edits, repeat snapshot to close the loop.

If Discord is **only** on the desktop app, the human describes new posts; the agent still runs checkpoints 2–4.

---

## Related docs

- [`intake-local-prereqs.md`](intake-local-prereqs.md) — env, Twingate, preflight, `notify-pipeline-start`.
- [`AGENTS.md`](../AGENTS.md) — `discord.env`, preflight summary.
