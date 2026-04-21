# voice-bridge manifests

In-cluster FastAPI WebSocket bridge that powers Morgan's voice loop:

- **Inbound**: CTO app (or any client) opens `wss://morgan-voice.5dlabs.ai/ws`,
  streams mic audio + optional text addenda.
- **STT**: ElevenLabs Scribe (`scribe_v1`) via `POST /v1/speech-to-text`.
- **Agent**: OpenAI-compatible HTTP gateway on
  `http://openclaw-morgan.cto.svc.cluster.local:18789/v1/chat/completions`,
  streamed via SSE (`model=openclaw/morgan`, `Authorization: Bearer openclaw-internal`).
- **TTS**: ElevenLabs Flash v2.5 streaming (`voice_id=iP95p4xoKVk53GoZ742B`).
- **Outbound**: reply text deltas as JSON frames + MP3 audio as binary frames.

## Deploy

```sh
# 1. Image: pushed by .github/workflows/build-voice-bridge.yml on changes
#    under infra/images/voice-bridge/ to ghcr.io/5dlabs/voice-bridge:latest.

# 2. Apply manifests
kubectl apply -f infra/manifests/voice-bridge/

# 3. Wait for ready
kubectl -n cto rollout status deploy/voice-bridge

# 4. Smoke
curl -sSf https://morgan-voice.5dlabs.ai/healthz
curl -sSf https://morgan-voice.5dlabs.ai/readyz
```

## Files

| File | Purpose |
|---|---|
| `deployment.yaml` | 1-replica Deployment, `ELEVENLABS_API_KEY` from secret `openclaw-api-keys`. |
| `service.yaml` | ClusterIP `voice-bridge.cto.svc:8090`. |
| `tunnel-binding.yaml` | Cloudflare `TunnelBinding` on `ClusterTunnel/morgan-avatar` exposing `morgan-voice.5dlabs.ai`. |
