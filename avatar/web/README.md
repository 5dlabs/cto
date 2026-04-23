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

Optional (TalkingHead runtime):

- `NEXT_PUBLIC_AVATAR_RUNTIME` — set to `talkinghead` to render the 3D WebGL avatar via
  [met4citizen/TalkingHead](https://github.com/met4citizen/TalkingHead). Omit or set to any other value
  to fall back to the deterministic portrait.
- `NEXT_PUBLIC_AVATAR_GLB_URL` — URL to a Mixamo-compatible humanoid `.glb` with ARKit 52 + Oculus 15
  viseme blendshapes. Recommended sources:
    1. **Ready Player Me** (free; non-commercial / commercial tiers available): create an avatar at
       <https://readyplayer.me> and append
       `?morphTargets=ARKit,Oculus%20Visemes&textureAtlas=1024&lod=1` to the generated `.glb` URL.
    2. **TalkingHead demo sample** (quick start, non-commercial only):
       `https://met4citizen.github.io/TalkingHead/avatars/brunette.glb`.

  Leaving the variable empty disables the 3D view and the client falls back to the deterministic
  portrait with a helper message.

### Lip-sync architecture (TalkingHead runtime)

Lip-sync is driven client-side by [met4citizen/HeadAudio](https://github.com/met4citizen/HeadAudio), a
browser-side AudioWorklet that classifies visemes in real time from any audio stream:

```
LiveKit RemoteAudioTrack → <audio> element (user hears Morgan) + MediaStreamAudioSourceNode
                                                                     ↓
                                                        HeadAudio worklet (viseme detection)
                                                                     ↓
                                                        head.mtAvatar[viseme_xx].newvalue = n
```

No voice-bridge text or ElevenLabs alignment frames are involved in the TalkingHead path — the
agent speaks directly through LiveKit and HeadAudio infers visemes from the audio. The worklet and
pre-trained model ship in `@met4citizen/headaudio`; a `postinstall` / `copy:headaudio` script
places them under `public/headaudio/` for the browser to fetch.

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

> **Note:** `npm run dev` / `npm run build` use `next --webpack` rather than Turbopack.
> TalkingHead's bundled module performs a string-constructed `import()` for
> per-language lipsync plugins, which Turbopack 16 cannot statically resolve.
> Webpack handles the dynamic import cleanly. This can be reverted once
> [vercel/next.js#85238](https://github.com/vercel/next.js/issues/85238) ships.

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
