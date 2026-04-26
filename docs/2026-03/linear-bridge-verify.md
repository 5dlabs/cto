# linear-bridge: verify and wire for intake

The intake pipeline’s **`intake-util register-run`** talks to **linear-bridge** over HTTP (`POST /runs/:runId/register`). This doc covers **tests**, **local smoke**, **cluster deployment**, and **`LINEAR_BRIDGE_URL`**.

---

## 1. Automated verification (repo)

From the CTO repo root:

```bash
chmod +x scripts/2026-03/verify-linear-bridge.sh
export LINEAR_API_KEY=…           # required for HTTP smoke
export LINEAR_TEAM_ID=CTOPA       # or your team UUID / key (matches `defaults.linear.teamId` in `cto-config.json`)
./scripts/2026-03/verify-linear-bridge.sh
```

The script runs **`npm ci`**, **`npm test`**, **`npm run build`** in `apps/linear-bridge`, then (if `LINEAR_API_KEY` is set) starts the server with **`ACP_ACTIVITY_ENABLED=false`** and **`AGENT_SESSIONS_ENABLED=false`** so Loki/Discord/agent-session paths are not required for the smoke check. It asserts:

- `GET /health` → JSON with `status: ok`
- `POST /runs/verify-smoke/register` with `{"agent":"intake","linearSessionId":"…"}` → success

Override the smoke port with **`VERIFY_LINEAR_BRIDGE_PORT`** if needed.

---

## 2. Pointing the intake pipeline at the bridge

When the bridge listens on your machine:

```bash
export LINEAR_BRIDGE_URL=http://127.0.0.1:3100   # or whatever port you use
```

Match the port to **`WEBHOOK_PORT`** (default **3100**) in the bridge process.

---

## 3. Kubernetes (OVH / CTO cluster)

Manifests live under **`infra/manifests/linear-bridge/`** (namespace **`bots`**, Service **`linear-bridge`**:3100). An Argo CD Application is defined at **`infra/gitops/applications/workloads/linear-bridge.yaml`**.

**Reality check:** If `kubectl get ns bots` returns **NotFound** and there is **no** `linear-bridge` Argo Application, the bridge is **not deployed** on that cluster yet. Sync/apply the Application (and ensure **`openclaw-linear`** and related secrets exist per the ExternalSecret in the deployment manifest) before relying on in-cluster `http://linear-bridge.bots.svc:3100`.

**Local shell → in-cluster bridge:**

```bash
kubectl port-forward -n bots svc/linear-bridge 3100:3100
export LINEAR_BRIDGE_URL=http://127.0.0.1:3100
```

---

## 4. Related docs

- Cloudflare tunnel / public URL: [`cloudflare-tunnel-intake-agent.md`](cloudflare-tunnel-intake-agent.md)
- Intake env checklist: [`intake-local-prereqs.md`](intake-local-prereqs.md)
