# Intake observer: logs + Discord + Linear

The **intake coordinator** acts as **observer** during intake runs: correlate **terminal/pipeline output**, **OpenClaw and bridge logs**, and **live UIs** (Discord + Linear). No single pane shows everything—this doc is the checklist. For **per–OpenClaw-agent** Cursor shadows (parallel monitors), see [`cursor-openclaw-subagent-plan.md`](cursor-openclaw-subagent-plan.md).

---

## Visual surfaces (browser MCP or desktop)

| Surface | What to watch | How (Cursor browser MCP) |
|---------|----------------|----------------------------|
| **Discord `#intake`** (and bridge rooms) | Pipeline **`notify-pipeline-start`**, deliberation **`bridge-notify`** embeds, bot errors | `browser_tabs` → `browser_snapshot` on the `#intake` tab; re-snapshot after each pipeline phase or failure. |
| **Linear** | Project/issues for the run, Agent session activity, comments from **`linear-activity`** / bridge | Separate tab; snapshot after major pipeline steps; compare issue/comment timestamps to local logs. |

**Rule:** Prefer **fresh snapshots** over assumptions. If the embedded browser hits **“incompatible browser”** on Linear, use **desktop Linear** and have the human confirm anomalies—the coordinator still monitors **Discord** via MCP when possible.

---

## Log sources (local + cluster)

### 1. OpenClaw gateway (local dev)

When the gateway runs on the laptop ([`openclaw-local-setup.md`](openclaw-local-setup.md)):

- Run **`openclaw gateway` in the foreground** in a dedicated terminal and **leave it visible** (or tee to a file: `openclaw gateway 2>&1 | tee /tmp/openclaw-gateway.log`).
- Errors that show as toasts in the Control UI usually appear **first** in this stream.

### 2. OpenClaw workloads (Kubernetes)

If agents run **in-cluster** (typical `openclaw` namespace on CTO clusters):

```bash
# Pods (names vary by release)
kubectl get pods -n openclaw -o wide

# Example: follow conductor / agent pods (adjust names from get pods)
kubectl logs -n openclaw deploy/openclaw-conductor -f --tail=200

# All pods in namespace (short)
kubectl logs -n openclaw -l app.kubernetes.io/name=openclaw --all-containers=true --tail=100
```

**Intake-specific:** filter for **`intake`**, **`lobster`**, **`llm-task`**, **`invoke`**, **`workflow`** in log lines; align timestamps with Discord/Linear.

### 3a. Bridges (local laptop)

When running **`intake/scripts/run-local-bridges.sh`**, follow **`intake/.bridge-logs/discord-bridge.log`** and **`linear-bridge.log`** (the script **`tail -f`** both by default — one terminal for both processes).

### 3. Bridges (Discord + Linear plumbing)

When deployed (often `bots` namespace—confirm in your cluster):

```bash
kubectl logs -n bots deploy/discord-bridge --tail=200 -f
kubectl logs -n bots deploy/linear-bridge --tail=200 -f
```

If namespaces differ, discover with:

```bash
kubectl get deploy -A | grep -E 'discord-bridge|linear-bridge'
```

### 4. Pipeline / Lobster (local shell)

The terminal where **`lobster run`** executes is the **source of truth** for step failures (step id, stderr). Keep it in view alongside gateway logs.

### 5. Optional: Loki / Grafana

If your environment ships logs to **Loki** ([`docs/tools-catalog.md`](tools-catalog.md) Grafana MCP / `grafana_query_loki_logs`), query for:

- `namespace="openclaw"` + pod/agent name  
- `namespace="bots"` + bridge components  

Use the same **time window** as a failed pipeline step.

---

## Observer loop (during an active intake)

1. **Baseline:** Snapshot Discord + note Linear issue/session if applicable.  
2. **Start pipeline / checkpoints** — note wall-clock time **T0**.  
3. **While running:** Every major step or failure:
   - refresh **gateway** (or conductor) logs around **T0**;
   - **snapshot Discord** (new embed?);
   - **snapshot Linear** (new comments/activity?).
4. **On failure:** Capture **Lobster step id** + log excerpt + one Discord/Linear snapshot for the triage artifact.  
5. **After fix:** Re-run from the **first invalidated checkpoint** ([`intake-discord-feedback-loop.md`](intake-discord-feedback-loop.md)).

---

## tmux observer grid (recommended)

For continuous tails outside UI snapshots, launch the local tmux observer session:

```bash
./intake/scripts/observer-tmux.sh --attach
```

What it gives you:
- `observer` window: `discord-bridge` log, `linear-bridge` log, latest pipeline log, OpenClaw gateway log (if tee'd to `/tmp/openclaw-gateway.log`)
- `cluster` window: pod watch + `openclaw-conductor` log stream (disable via `--no-cluster`)

Useful flags:
- `--session <name>` custom tmux session name
- `--no-cluster` local-only monitoring
- `--attach` attach immediately after launch

---

## Related

- [`intake-coordinator.md`](intake-coordinator.md) — autonomy and approval gate.  
- [`intake-discord-feedback-loop.md`](intake-discord-feedback-loop.md) — checkpoint ordering.  
- [`cloudflare-tunnel-intake-agent.md`](cloudflare-tunnel-intake-agent.md) — bridge/tunnel path.  
- [`openclaw-local-setup.md`](openclaw-local-setup.md) — local gateway logging.
