# Morgan Avatar PoC — Handoff for Claude

This document summarizes the Morgan talking-avatar proof of concept so another Claude session (or a human) can continue the work without re-reading every transcript. It is intended as the single entry point for context.

---

## 1. Goal and scope

- **Goal**: Two-way, low-latency voice conversation with **Morgan** in the browser (LiveKit room + LemonSlice avatar + OpenClaw persona).
- **In scope**: Browser demo at `http://localhost:3000`, Python agent, Next.js token/room UI, latency tooling, Morgan routing and tunnel (CTO repo).
- **Out of scope for now**: Desktop/cto-lite app, in-cluster avatar worker, Discord (mentioned in skills but not part of this PoC).

---

## 2. Architecture (high level)

```
Browser → Next.js /api/token → LiveKit Cloud
                                    |
                             Python Avatar Agent
                            /        |         \
                    OpenClaw   LemonSlice   Latency Logs
                 (morgan.5dlabs.ai)  (avatar video)
```

- **LiveKit**: WebRTC transport (rooms, tokens, agent dispatch).
- **LemonSlice**: Lip-synced avatar video from a still image (or pre-built agent ID).
- **OpenClaw**: Morgan persona LLM at `morgan.5dlabs.ai` (requires `x-openclaw-agent-id: morgan`).
- **STT**: LiveKit Inference Deepgram Flux (default); optional direct Deepgram Flux/Nova.
- **TTS**: ElevenLabs (default); optional Cartesia / LiveKit Inference.

The token API **creates the room and dispatches** the `morgan-avatar` agent before returning a token so the agent always joins.

---

## 3. How to run (two terminals)

**Terminal 1 — Agent**

```bash
cd agent
source .venv/bin/activate
SSL_CERT_FILE="$(python -c 'import certifi; print(certifi.where())')" \
REQUESTS_CA_BUNDLE="$(python -c 'import certifi; print(certifi.where())')" \
python agent.py dev
```

**Terminal 2 — Web**

```bash
cd web
npm run dev
```

Then open `http://localhost:3000` and click **Talk to Morgan**.

**Env**: Create `agent/.env` and fill LiveKit, LemonSlice, OpenClaw (or inference fallback), and any STT/TTS keys for the modes you use. See `docs/provider-spikes.md` for swap combinations.

---

## 4. What’s been done

- **Avatar repo** (`/Users/jonathon/avatar`):
  - Python LiveKit agent: `agent/agent.py`, `agent/morgan_avatar_agent/` (config, providers, latency).
  - Next.js app and token API that **create room + dispatch** `morgan-avatar` before issuing the token.
  - Room UI and client-side latency panel.
- **Morgan in CTO repo** (separate repo): `cto/infra/gitops/agents/morgan-values.yaml`, `cto/infra/manifests/morgan-support/` (ClusterTunnel, TunnelBinding, skills ConfigMap), ArgoCD ApplicationSet, dedicated Cloudflare tunnel for `morgan.5dlabs.ai` (e.g. PR from branch `morgan-agent-cto`).
- **Agent config**: LemonSlice via `MORGAN_LEMONSLICE_AGENT_ID`; LLM via `MORGAN_LLM_BASE_URL=https://morgan.5dlabs.ai` and `MORGAN_LLM_AGENT_ID=morgan`; `providers.py` sends `x-openclaw-agent-id: morgan`.
- **Latency**: `agent/morgan_avatar_agent/latency.py` and `agent/scripts/summarize_latency.py` with partial sums, greeting vs conversational turns, per-component stats (EOU, STT, LLM TTFT, TTS TTFB).
- **Git**: Avatar repo initialized and committed; CTO Morgan changes committed and pushed on `morgan-agent-cto`.
- **Docs**: Root and agent READMEs (quick start, routing header, tunnel secret, DNS, manual cluster state, follow-up); `docs/decision-review.md`, `docs/provider-spikes.md`, `docs/morgan-image.md`.
- **Smoke tests**: Three greeting-only runs were done successfully (see transcript [02aa3f84](agent-transcripts/02aa3f84-83de-458b-870c-5dd73ea7325e)); greeting and avatar stable; “First speaking state” showed some variance (e.g. ~6.2s vs ~9.2s).

---

## 5. Known issues and mitigations

### “He can’t hear me” / Morgan doesn’t respond

- **Cause**: The agent **does** receive your speech (STT transcripts appear in logs). OpenClaw often takes **~10 seconds** to respond. If you speak again before that reply completes, the in-flight LLM request is **cancelled** and Morgan never replies.
- **Mitigation**: Say **one short sentence**, then **stay silent for 10–15 seconds** so the reply can complete.
- **Optional**: To validate the loop without OpenClaw latency, temporarily use inference: `MORGAN_LLM_BACKEND=inference`, `MORGAN_LLM_MODEL=openai/gpt-4.1-mini` (see `docs/provider-spikes.md`).

### Agent not joining room

- **Fix**: Token route must create the room and create a dispatch for `morgan-avatar`. This is implemented in `web/app/api/token/route.ts` via `ensureRoomAndDispatch`.

### Generic persona from gateway / no reply from Morgan

- **Cause**: Requests to `morgan.5dlabs.ai` without routing fall back to a generic assistant. Or the OpenClaw pod that serves that hostname does not have an agent with `id: "morgan"` in its config, so requests hang or return generic behavior.
- **Fix**: Agent must send `x-openclaw-agent-id: morgan`. Set `MORGAN_LLM_AGENT_ID=morgan` in `.env`; `providers.py` adds the header. On the OpenClaw side, the gateway that receives traffic for morgan.5dlabs.ai must have an agent with `agent.id: "morgan"` (see `docs/morgan-openclaw-setup.md`). Check that pod’s logs: `kubectl logs -n openclaw <pod> -c agent --tail=200`.

### SSL errors (Python)

- **Fix**: Run the agent with certifi: `SSL_CERT_FILE` and `REQUESTS_CA_BUNDLE` as in the run commands above.

### Latency “0 measured turns”

- **Fix**: Latency reporting was updated to use partial sums, greeting path, and per-component stats so short or greeting-only sessions still produce useful metrics.

---

## 6. Key paths (this repo)

| Path | Purpose |
|------|---------|
| `agent/agent.py` | Entrypoint; runs agent with LemonSlice, STT/LLM/TTS, latency recorder |
| `agent/morgan_avatar_agent/config.py` | All env-driven config (STT/TTS/LLM modes, timeouts, etc.) |
| `agent/morgan_avatar_agent/providers.py` | Builds LLM (OpenClaw vs inference), STT, TTS; injects `x-openclaw-agent-id` |
| `agent/morgan_avatar_agent/latency.py` | Turn records, greeting/conversational breakdown, per-component stats |
| `agent/scripts/summarize_latency.py` | Summarizes latest run (table or JSON) |
| `agent/.env` | Credentials and mode overrides (not in git) |
| `web/app/api/token/route.ts` | Mints token; ensures room exists and `morgan-avatar` is dispatched |
| `web/components/Room.tsx` | Room UI and client latency display |
| `docs/decision-review.md` | Decision to stay on LiveKit + LemonSlice; decision gates for provider swaps |
| `docs/provider-spikes.md` | Env-only provider swap recipes (STT, TTS, inference fallback) |
| `docs/morgan-image.md` | Morgan image prep for LemonSlice |

---

## 7. Repo boundaries and manual cluster state

| Repo | What lives there |
|------|------------------|
| **This repo** (`avatar/`) | Python agent, Next.js frontend, latency tooling, this handoff |
| **CTO** (`5dlabs/cto`) | Morgan Helm values, Cloudflare tunnel manifests, ArgoCD ApplicationSet |
| **openclaw-platform** | Reference for agent deployment patterns (not modified for this PoC) |

**Manual cluster state (not in git):**

- `morgan-avatar-tunnel-credentials` in `cloudflare-operator-system` (Cloudflare API credentials for dedicated tunnel).
- DNS CNAME `morgan.5dlabs.ai` in Cloudflare pointing to the tunnel.
- ElevenLabs API key in `openclaw` namespace (e.g. via ExternalSecrets).

---

## 8. OpenClaw pod logs and agent config

- **Which pod serves morgan.5dlabs.ai?** The Cloudflare tunnel for morgan.5dlabs.ai targets a specific service/pod. There is currently **no** `openclaw-morgan-*` pod in the `openclaw` namespace; other agents (intake, metal, keeper, etc.) exist. If the tunnel points at one of them, that pod must have `morgan` in its `agents.list` or requests will not be handled as Morgan.
- **Read logs**: `kubectl logs -n openclaw <pod-name> -c agent --tail=200`. Look for gateway startup (`agent model: ...`, `listening on ... 18789`), errors, or timeouts when the avatar sends a request.
- **Reference setup**: See `docs/morgan-openclaw-setup.md` (pattern from openclaw-platform: agent.id, values, configmap, and how to align Morgan).

---

## 9. Before the next test

- **Single agent**: Only one `python agent.py dev` process; kill any stale runs.
- **Env**: Agent has LiveKit, LemonSlice, OpenClaw (or inference); web has LiveKit in `.env.local`.
- **Wait after speaking**: One sentence, then 10–15 s silence so OpenClaw can reply.
- **UI**: Use “Heard you” to confirm speech is received; in-room tip reminds about waiting.
- **Optional**: Use `MORGAN_LLM_BACKEND=inference` for a faster validation loop, then switch back to OpenClaw.

---

## 10. Suggested next steps

1. **Latency baseline**: Run real multi-turn sessions with OpenClaw and record p50/p95 “end-of-turn to first audio” using `summarize_latency.py`; use `docs/decision-review.md` and `docs/provider-spikes.md` to decide STT/TTS/inference swaps.
2. **Optional inference test**: Use `MORGAN_LLM_BACKEND=inference` briefly to confirm the full loop is fast when LLM is local; then switch back to OpenClaw for persona.
3. **Hostname-only routing**: Platform change so `morgan.5dlabs.ai` implies Morgan persona without requiring `x-openclaw-agent-id` (currently documented as follow-up in README).
4. **In-cluster avatar worker**: Time-boxed experiment to run the Python avatar worker in the same cluster as OpenClaw to reduce network hop latency.
5. **Desktop / cto-lite**: Phase 2, gated on PoC success; embed avatar in Tauri app with KinD-based local OpenClaw.

---

## 11. Transcript references

- [Morgan Avatar Implementation](f5bbf952-7eca-45f9-bc68-0f9f43730acc) — Main implementation and debugging (agent, web, token, dispatch, routing, SSL, latency, “can’t hear me” diagnosis).
- [Greeting-only smoke tests](02aa3f84-83de-458b-870c-5dd73ea7325e) — Three sequential greeting-only runs; success; latency variance noted.

Cite by parent UUID only; do not cite subagent transcripts.

---

*Last updated from conversation summary and repo state; intended for Claude or any handoff reader.*
