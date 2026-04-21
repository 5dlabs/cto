# morgan-voice-bridge

FastAPI + WebSocket sidecar for two-way voice with in-cluster OpenClaw agents.

This branch adds Hermes-compatible multi-agent routing on top of the existing
voice bridge so the client can select an agent with `?agent=` and get the
correct upstream model and ElevenLabs voice without code changes.

## What it does

- clean WebSocket disconnect handling, no noisy stack trace on normal client exits
- multi-agent registry from `VOICE_AGENTS_JSON`
- `?agent=morgan|hermes` routing with hard reject for unknown agents
- shared-secret auth for `/ws`
- simple per-client turn rate limiting
- structured `turn_metrics` log lines per completed turn
- pytest smoke coverage for routing, auth, unknown agents, and rate limiting

## Environment

### Core

- `ELEVENLABS_API_KEY`: ElevenLabs API key for Scribe STT and Flash TTS
- `VOICE_AGENTS_JSON`: optional JSON object defining available agents
- `VOICE_BRIDGE_SHARED_SECRET`: optional shared secret required on `/ws`
- `VOICE_BRIDGE_MAX_TURNS`: max turns per client per window, default `20`
- `VOICE_BRIDGE_RATE_WINDOW_S`: rate-limit window in seconds, default `60`
- `VOICE_BRIDGE_AUDIO_MIME`: upload mime type for STT, default `audio/webm`
- `VOICE_BRIDGE_AUDIO_NAME`: upload filename for STT, default `turn.webm`

### Default per-agent fallbacks

If `VOICE_AGENTS_JSON` is unset, the service exposes built-in `morgan` and
`hermes` agents from env fallbacks:

- `MORGAN_GATEWAY_URL`, `MORGAN_GATEWAY_TOKEN`, `MORGAN_MODEL`, `MORGAN_VOICE_ID`
- `HERMES_GATEWAY_URL`, `HERMES_GATEWAY_TOKEN`, `HERMES_MODEL`, `HERMES_VOICE_ID`

## VOICE_AGENTS_JSON format

```json
{
  "morgan": {
    "model": "openclaw/morgan",
    "voice_id": "voice-id-morgan",
    "gateway_url": "http://openclaw-morgan.cto.svc.cluster.local:18789",
    "gateway_token": "openclaw-internal"
  },
  "hermes": {
    "model": "openclaw/hermes",
    "voice_id": "voice-id-hermes",
    "gateway_url": "http://openclaw-hermes.cto.svc.cluster.local:18789",
    "gateway_token": "openclaw-internal"
  }
}
```

## WebSocket contract

Connect to `/ws?agent=<name>`.

### Auth

If `VOICE_BRIDGE_SHARED_SECRET` is set, provide it by one of:

- query param: `?token=...`
- header: `x-voice-bridge-token: ...`
- header: `authorization: Bearer ...`

JWT mode is not implemented yet.

### Unknown agent behavior

Unknown agents are rejected with WebSocket close code `4404` and reason
`unknown_agent`.

## Local run

```bash
cd infra/images/voice-bridge
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
uvicorn app.main:app --host 0.0.0.0 --port 8090 --reload
```

Health:

```bash
curl localhost:8090/readyz
```

## Local smoke test

Run tests:

```bash
cd infra/images/voice-bridge
pytest -q
```

Manual WebSocket smoke flow:

1. connect to `ws://localhost:8090/ws?agent=hermes&token=<secret>`
2. send `{"type":"start","session_id":"demo"}`
3. send audio bytes and optional `{"type":"text","text":"hello"}`
4. send `{"type":"end_utterance"}`
5. expect `transcript`, `reply_delta`, `reply_text`, MP3 bytes, then `turn_done`

## Adding a new voice agent

1. add an entry to `VOICE_AGENTS_JSON`
2. set that agent's `model`, `voice_id`, `gateway_url`, and `gateway_token`
3. connect with `?agent=<new-name>`
4. if the name is missing from the registry, the service closes with `4404 unknown_agent`

## Notes

- rate limiting rejects excess turns with `{"type":"error","error":"rate_limited"}`
- each completed turn emits one JSON log line with `type=turn_metrics`
- the bridge currently uses simple shared-secret auth only
