# morgan-voice-bridge

FastAPI + WebSocket sidecar for two-way voice with the in-cluster Morgan
OpenClaw StatefulSet (`cto/openclaw-morgan-openclaw`).

**Status: scaffold only.** Nothing is deployed. Building and running is opt-in.

## Architecture

```
┌──── CTO desktop app (Tauri) ─────┐
│  MorganView.tsx                  │
│    ├── LemonSlice widget (face)  │   ← Path Z, shipped
│    └── WS client → voice-bridge  │   ← Path B, this package
└───────────────┬──────────────────┘
                │  wss://morgan-voice.5dlabs.ai/ws  (cloudflared)
                ▼
┌──── openclaw-morgan-openclaw pod (StatefulSet) ──────┐
│  morgan container (unchanged)                        │
│  voice-bridge sidecar (this image) :8090             │
│    ├── STT (ElevenLabs Scribe)                       │
│    ├── TTS (ElevenLabs Flash v2.5, streaming MP3)    │
│    └── agent_client.py → Morgan (NATS, TBC)          │
└──────────────────────────────────────────────────────┘
```

## Local build

```
cd infra/images/voice-bridge
docker build -t voice-bridge:dev .
docker run --rm -p 8090:8090 \
  -e ELEVENLABS_API_KEY=... \
  -e MORGAN_VOICE_ID=... \
  voice-bridge:dev
```

Health: `curl localhost:8090/readyz`

## Outstanding work (do not deploy until these are settled)

- [ ] Confirm Morgan's actual input transport (NATS subject pair assumed
      in `agent_client.py`; could be Linear/Discord bridge or ACP).
- [ ] Pick audio container for STT uploads (opus-in-webm vs pcm16 wav).
- [ ] Provision Cloudflare tunnel hostname `morgan-voice.5dlabs.ai`
      (destructive — needs explicit approval).
- [ ] Patch `openclaw-morgan-openclaw` chart to add the sidecar
      (destructive — needs explicit approval).
- [ ] Wire the CTO desktop app voice/text modes to this WebSocket
      (currently only the video tab is live via LemonSlice).
