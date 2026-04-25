# Morgan Avatar Web Client

Next.js frontend for the Morgan talking-avatar proof of concept.

## What it does

- Calls `POST /api/token` to create a short-lived LiveKit access token.
- Ensures the target room exists and dispatches the `morgan-avatar` worker.
- Connects to LiveKit in-browser for mic capture + audio playback.
- Shows basic session telemetry (connect timing, media readiness, first speaking state).

## Prerequisites

- Node.js 18+
- Valid self-hosted LiveKit credentials
- The Python avatar worker running from `../agent` with agent name `morgan-avatar`

## Environment

Create `.env.local` from `.env.local.example`:

```bash
cp .env.local.example .env.local
```

Required variables:

- `LIVEKIT_URL` (WebSocket URL, e.g. `wss://lk.5dlabs.ai`)
- `LIVEKIT_API_KEY`
- `LIVEKIT_API_SECRET`

These are read only on the server route (`app/api/token/route.ts`) and are not exposed to the browser.

Optional:

- `NEXT_PUBLIC_AVATAR_RUNTIME=derived-text` keeps the deterministic fallback
  portrait while deriving rough text cues from bridge frames. The active product
  path is remote LiveKit video from LemonSlice or EchoMimic whenever the worker
  publishes a video track.

## Run locally

```bash
npm install
npm run dev
```

If you're using a self-hosted LiveKit that isn't publicly reachable (the default for our cluster),
open the port locally in another terminal:

```bash
kubectl -n cto port-forward svc/livekit-server 7880:80
```

And set `LIVEKIT_URL=ws://localhost:7880` in `.env.local`.

Open `http://localhost:3000` and click **Talk to Morgan**.

### EchoMimic turn demo

Open `http://localhost:3000/echo-turn` to test the batch EchoMimic path in a
web page:

1. Stream Morgan's text reply through `/api/echo-turn/chat`.
2. Generate or fall back to MP3 audio through `/api/echo-turn/tts`.
3. Send that audio plus Morgan's reference image to EchoMimic through
   `/api/echo-turn/avatar`.
4. Play the returned MP4 in the browser.

Set `ECHOMIMIC_APP_URL` to the active EchoMimic FastAPI app URL. Optional
`MORGAN_GATEWAY_*` and `ELEVENLABS_*` variables enable real model and TTS calls;
without them, the page still runs with deterministic streamed text and the
checked-in `voice_clone_sample.mp3`. For shared OpenClaw gateways, the chat
route also accepts `MORGAN_LLM_BASE_URL`, `MORGAN_LLM_AGENT_ID`, and
`OPENCLAW_TOKEN`, and forwards `x-openclaw-agent-id` server-side. This is a
turn-based bridge, not true low-latency video streaming: LiveKit/WebRTC remains
the target transport for live mic/audio, while EchoMimic currently returns
complete MP4 files per turn.

## Token + dispatch flow

`POST /api/token` (`app/api/token/route.ts`) performs:

1. Validate LiveKit environment variables.
2. Create room if missing (`emptyTimeout=60`, `departureTimeout=15`, `maxParticipants=2`).
3. Ensure a dispatch exists for agent `morgan-avatar`.
4. Mint a 10-minute user token with room join/publish/subscribe grants.

Response payload:

```json
{
  "token": "<jwt>",
  "serverUrl": "wss://...",
  "roomName": "morgan-<suffix>",
  "identity": "user-<suffix>",
  "participantName": "optional"
}
```

## Operational constraints

- Worker identity is currently coupled: token route dispatches `morgan-avatar` and the Python worker registers the same agent name.
- The worker env must include `LIVEKIT_URL`, `LIVEKIT_API_KEY`, `LIVEKIT_API_SECRET`, and one of the allowed avatar modes: `lemonslice`, `echomimic`, or `disabled` for audio-only validation.
- If room creation/dispatch/token mint fails, the UI surfaces the route error message directly.
- OpenClaw reply latency can be ~10 seconds; use the in-app transcript/telemetry panel to verify progress.

## Troubleshooting

- `Server misconfigured...` in UI: check `.env.local` has all three LiveKit vars.
- Connect succeeds but no agent response: confirm the Python worker is running and registered as `morgan-avatar`.
- Mic text never appears in "Heard you": verify browser microphone permissions and active mic selection.
