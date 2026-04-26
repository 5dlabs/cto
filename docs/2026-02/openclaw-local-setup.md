# Local OpenClaw Gateway Setup

This guide gets the **OpenClaw gateway** running on your machine so the **CTO desktop toolbar app** (cto-lite) can connect to it.

## 1. Install OpenClaw CLI

Install the latest OpenClaw (or a pinned version):

```bash
npm install -g openclaw@latest
# or pin to a specific version, e.g.:
# npm install -g openclaw@2026.2.12
openclaw --version
```

Requires **Node.js 20+**.

## 2. Allow the gateway to run locally

The gateway refuses to start unless it’s configured for local use. Either:

**Option A – Config file (recommended)**

Create or edit `~/.openclaw/openclaw.json` and set gateway mode and optional port:

```json
{
  "gateway": {
    "mode": "local",
    "port": 18789
  }
}
```

**Option B – Ad‑hoc run without config**

```bash
openclaw gateway --allow-unconfigured --port 18789
```

## 3. Start the gateway

In a terminal, run:

```bash
openclaw gateway
```

Default port is **18789**. To use another port (e.g. 3100):

```bash
openclaw gateway --port 3100
```

If something is already using the port:

```bash
openclaw gateway --port 18789 --force
```

Leave this terminal running. You should see the gateway listening (e.g. on 18789).

## 4. Connect the desktop toolbar app

The CTO desktop app (cto-lite) talks to the gateway over HTTP. It uses:

- **Default URL:** `http://localhost:18789` (OpenClaw’s default port)
- **Override:** set `OPENCLAW_GATEWAY_URL` to your gateway URL before starting the app, e.g.  
  `export OPENCLAW_GATEWAY_URL=http://localhost:3100` if you ran the gateway with `--port 3100`.

Start the toolbar app after the gateway is running. It will call `/api/status` (and other `/api/*` endpoints) to check connectivity.

## 5. If the toolbar still won’t connect

- **Gateway not running:** Ensure `openclaw gateway` is running and shows it’s listening (e.g. on 18789).
- **Wrong port:** The app defaults to 18789. If you use a different port, set `OPENCLAW_GATEWAY_URL` (see above).
- **API shape:** The toolbar expects HTTP endpoints such as `/api/status`, `/api/chat`, `/api/workflows`. OpenClaw’s gateway may expose different paths (e.g. WebSocket RPC or `/v1/...`). If the gateway is up but the app reports “not connected”, check [OpenClaw gateway HTTP API docs](https://docs.openclaw.ai/cli/gateway) and that your OpenClaw version matches what the app expects.
- **Auth:** If your gateway uses token or password auth, you may need to set `OPENCLAW_GATEWAY_TOKEN` (or the appropriate env) when starting the desktop app so it can authenticate.

## 6. "Transaction process failed" in the Control UI

If the **OpenClaw Control UI** (browser or in-app view of the gateway) shows **"Transaction process failed"**, that message comes from OpenClaw's own UI when an internal operation (e.g. processing an incoming Discord message) fails. To find the real cause:

1. **Run the gateway in the foreground** so you can see logs:
   - If the gateway is already running in another terminal (or as a background process), stop that process first (Ctrl+C in that terminal, or `pkill -f "openclaw gateway"` if needed).
   - If you previously ran `openclaw gateway install` (launchd service), run `openclaw gateway stop` so the service isn’t running, then start in the foreground.
   - In a terminal, run:
   ```bash
   openclaw gateway        # run in foreground, leave it open
   ```
   **Note:** `openclaw gateway stop` loads config and plugins (you may see “Discord poster logged in” etc.) then checks for an installed *service* (e.g. launchd). “Gateway service not loaded” means no service is installed—nothing to stop. The process then exits; it does not leave the gateway running.
2. **Reproduce the issue** (e.g. send Henry a Discord DM).
3. **Watch the terminal** where the gateway is running. The stack trace or error line that appears at the same time as the UI toast is the underlying failure (e.g. session path, NATS, or Discord handling).

Common causes after a token or config change: session directory validation, plugin errors, or Discord gateway reconnecting. Fix the error shown in the gateway logs; the UI message will stop once the operation succeeds.

## 7. Paths and heartbeat

**`[heartbeat] failed: Session file path must be within sessions directory`**  
OpenClaw requires session files to live under the agent’s **workspace** (e.g. `~/.openclaw/workspace`). Ensure in `~/.openclaw/openclaw.json`:

- `agents.defaults.workspace` is set to `~/.openclaw/workspace`
- Each agent (e.g. `agents.list[].workspace`) uses that path or one under it

Create the sessions directory if missing: `mkdir -p ~/.openclaw/workspace/sessions`.

**Known issue: `[discord] handler failed: Session file path must be within sessions directory`**  
This is a known OpenClaw bug when the gateway uses a **combined** session store (multiple agents): internally `storePath` becomes `"(multiple)"`, and code that does `path.dirname(storePath)` gets `"."` as the sessions directory, so real session paths are rejected. Two workarounds:

1. **Normalize `sessionFile` in the store**  
   Open `~/.openclaw/agents/henry/sessions/sessions.json` and change any `sessionFile` value that is a full path to just the filename (e.g. `abc123.jsonl`). Restart the gateway. This can reduce the failure but may not fix it if the bug is in the combined-store path.

2. **Force a single session store (recommended)**  
   In `~/.openclaw/openclaw.json`, set an explicit **non-template** session store path so the gateway never uses `"(multiple)"`. For a single-agent (Henry) setup, add under the top-level `session` key:
   ```json
   "session": {
     "store": "~/.openclaw/agents/henry/sessions/sessions.json"
   }
   ```
   Use a literal path (no `{agentId}`). Restart the gateway. If you add more agents later, you can remove this and rely on the default template; the upstream fix is to use `agentId` for session dir resolution when `storePath === "(multiple)"`.

**`[tools] exec failed: ls: /Users/jonathonfritz/cto/docs/research/: No such file or directory`**  
Cron jobs or prompts may reference an old CTO path. Your real repo is likely `.../5d-labs-workspace/cto/0-main`. Either:

- **Edit the cron job:** In `~/.openclaw/cron/jobs.json`, change any `.../cto/docs/research/` (or `/Users/jonathonfritz/cto/...`) to your actual repo path, e.g. `/Users/jonathonfritz/5d-labs-workspace/cto/0-main/docs/research/`.
- **Or add a symlink** so the old path works:  
  `ln -snf /Users/jonathonfritz/5d-labs-workspace/cto/0-main /Users/jonathonfritz/cto`

**`tools.profile (coding) allowlist contains unknown entries (group:memory)`**  
The coding tool profile references a `group:memory` tool group that isn’t enabled. Either enable the memory plugin in config or remove `group:memory` from the profile allowlist.

**`exec failed: error: unknown option '--limit'`**  
A tool or skill is invoking a CLI (e.g. `ls`) with a `--limit` flag that your system’s version doesn’t support. Update the skill or tool to use a supported invocation.

## Quick reference

| What              | Value / command |
|-------------------|------------------|
| Default gateway port | 18789 |
| Config file       | `~/.openclaw/openclaw.json` |
| Local mode        | `gateway.mode: "local"` |
| Start gateway     | `openclaw gateway` |
| Start on 3100     | `openclaw gateway --port 3100` |
| App URL override  | `OPENCLAW_GATEWAY_URL=http://localhost:18789` |

## Links

- [OpenClaw CLI: gateway](https://docs.openclaw.ai/cli/gateway)
- [OpenClaw gateway configuration](https://docs.openclaw.ai/gateway/configuration)
