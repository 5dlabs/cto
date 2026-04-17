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

## Run locally

```bash
npm install
npm run dev
```

Open `http://localhost:3000` and click **Talk to Morgan**.

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
- Phase 2 expects the worker env to include `LIVEKIT_URL`, `LIVEKIT_API_KEY`, `LIVEKIT_API_SECRET`, and `MORGAN_AVATAR_MODE=disabled` for the audio-only gate.
- If room creation/dispatch/token mint fails, the UI surfaces the route error message directly.
- OpenClaw reply latency can be ~10 seconds; use the in-app transcript/telemetry panel to verify progress.

## Troubleshooting

- `Server misconfigured...` in UI: check `.env.local` has all three LiveKit vars.
- Connect succeeds but no agent response: confirm the Python worker is running and registered as `morgan-avatar`.
- Mic text never appears in "Heard you": verify browser microphone permissions and active mic selection.
