# Morgan Talking Avatar PoC

A proof of concept for Morgan as a two-way conversational talking avatar.

## Quick start

### Prerequisites

- Python 3.11
- Node.js 18+
- API keys for: LiveKit, LemonSlice, ElevenLabs (via LiveKit Inference)
- Access to `morgan.5dlabs.ai` (OpenClaw gateway with Morgan persona)

### 1. Agent setup (one time)

```bash
cd agent
python3.11 -m venv .venv
source .venv/bin/activate
pip install -e '.[dev]'
cp .env.example .env
# Fill in .env with your credentials
```

### 2. Frontend setup (one time)

```bash
cd web
npm install
cp .env.local.example .env.local
# Add LIVEKIT_URL, LIVEKIT_API_KEY, LIVEKIT_API_SECRET
```

### 3. Run the demo (two terminals)

**Terminal 1 -- Agent:**

```bash
cd agent
source .venv/bin/activate
SSL_CERT_FILE="$(python -c 'import certifi; print(certifi.where())')" \
REQUESTS_CA_BUNDLE="$(python -c 'import certifi; print(certifi.where())')" \
python agent.py dev
```

**Terminal 2 -- Web:**

```bash
cd web
npm run dev
```

Open `http://localhost:3000` and click **Talk to Morgan**.

### Before you test (checklist)

- **Only one agent process** — If you’ve run the agent before, make sure no other `python agent.py dev` is running (e.g. in another terminal or from a previous run). Multiple agents can fight for the same room.
- **Env set** — `agent/.env` has at least: LiveKit URL/keys, `MORGAN_LEMONSLICE_AGENT_ID` (or image URL), `MORGAN_LLM_BASE_URL` and `MORGAN_LLM_AGENT_ID=morgan` for OpenClaw. `web/.env.local` has LiveKit credentials for the token API.
- **Wait after speaking** — OpenClaw can take ~10 s to reply. Say one short sentence, then stay silent for 10–15 seconds so the reply isn’t cancelled. Use the “Heard you” panel to confirm your words are being received.
- **Optional: faster loop** — To validate the full loop with a quick reply, set in `agent/.env`: `MORGAN_LLM_BACKEND=inference` and `MORGAN_LLM_MODEL=openai/gpt-4.1-mini`, then restart the agent. Switch back to OpenClaw for the real Morgan persona.

### Expected behavior

1. Room connects in ~1-2 seconds.
2. Morgan's avatar appears and greeting plays within ~5-9 seconds.
3. Agent state transitions: `connecting` -> `speaking` -> `listening`.
4. Speak into your mic; Morgan responds with lip-synced video.

## Architecture

```
Browser -> Next.js /api/token -> LiveKit Cloud
                                     |
                              Python Avatar Agent
                             /        |         \
                     OpenClaw    LemonSlice   Latency Logs
                  (morgan.5dlabs.ai)  (avatar video)
```

- **LiveKit** -- real-time WebRTC transport (rooms, tokens, tracks)
- **LemonSlice** -- lip-synced avatar video from a still image
- **OpenClaw** -- Morgan persona LLM backend
- **Deepgram** (via LiveKit Inference) -- speech-to-text
- **ElevenLabs** (via LiveKit Inference) -- text-to-speech

## Project layout

| Path | Purpose |
|------|---------|
| `agent/` | Python LiveKit agent with latency logging |
| `web/` | Next.js browser client with LiveKit room UI |
| `docs/HANDOFF.md` | Handoff doc for Claude / continuing work (context, issues, next steps) |
| `docs/morgan-image.md` | Asset prep guidance for Morgan's portrait |
| `docs/provider-spikes.md` | Zero-code provider swap instructions |
| `docs/decision-review.md` | Architecture decision record |
| `docs/morgan-openclaw-setup.md` | Morgan OpenClaw deployment runbook (CTO manifests, validation commands, troubleshooting) |

## Latency instrumentation

The agent writes turn-level latency logs to `agent/runs/` and computes timing breakdowns for:

- **Greeting path**: session start to first TTS audio
- **Conversational turns**: end-of-utterance to first audio (EOU + STT + LLM TTFT + TTS TTFB)
- **Per-component**: individual p50/p95 for each stage

Summarize the latest run:

```bash
cd agent
source .venv/bin/activate
python scripts/summarize_latency.py        # human-readable table
python scripts/summarize_latency.py --json  # raw JSON
```

## Known constraints

- **Morgan routing requires a header.** `morgan.5dlabs.ai` serves a shared
  OpenClaw gateway. Without `x-openclaw-agent-id: morgan` the response falls
  back to a generic assistant. The Python agent sends this header automatically
  when `MORGAN_LLM_AGENT_ID=morgan` is set in `.env`.
- **Cloudflare tunnel credential is manual.** The `morgan-avatar-tunnel-credentials`
  secret lives in the cluster only and is not checked into git.
- **DNS CNAME is manual.** The `morgan.5dlabs.ai` CNAME was pointed at the
  dedicated tunnel ID via the Cloudflare API.

## Manual cluster state (not in git)

These items exist in the OVH Kubernetes cluster and were configured manually:

| Item | Location | Notes |
|------|----------|-------|
| `morgan-avatar-tunnel-credentials` | `cloudflare-operator-system` namespace | Cloudflare API credentials for the dedicated tunnel |
| DNS CNAME `morgan.5dlabs.ai` | Cloudflare DNS | Points to tunnel ID `6967368c-...cfargotunnel.com` |
| ElevenLabs API key | `openclaw` namespace, via ExternalSecrets | Synced from 1Password/OpenBao |

## Repo boundaries

| Repo | What lives there |
|------|-----------------|
| **This repo** (`avatar/`) | Python agent, Next.js frontend, latency tooling |
| **CTO** (`5dlabs/cto`) | Morgan Helm values, Cloudflare tunnel manifests, ArgoCD ApplicationSet |
| **openclaw-platform** (`5dlabs/openclaw-platform`) | Upstream OpenClaw Helm chart source consumed by CTO ArgoCD apps |

## Follow-up work

- **Hostname-only routing**: Platform-layer change so `morgan.5dlabs.ai` implies the Morgan persona without requiring the `x-openclaw-agent-id` header.
- **In-cluster avatar agent**: Time-boxed experiment placing the Python avatar worker inside the same cluster as OpenClaw to reduce network hop latency.
- **Desktop app integration**: Embed the avatar in the `cto-lite` Tauri app with KinD-based local OpenClaw (phase 2, gated on PoC success).
- **Morgan image refresh**: The LemonSlice agent currently uses an uploaded image; a proper Morgan portrait aligned to the persona would improve demo quality.
