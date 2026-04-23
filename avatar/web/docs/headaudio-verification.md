# HeadAudio lip-sync verification checklist (WS-A)

End-to-end smoke test for the HeadAudio viseme classifier pipeline introduced in
[PR #4790](https://github.com/5dlabs/cto/pull/4790). Target: a human can verify
a working lip-sync path locally in **under 5 minutes**.

## 0. Architecture recap

```
Python agent TTS
   └── LiveKit RemoteAudioTrack
         ├── <audio>  (user hears Morgan; see AssistantAudioRenderer)
         └── MediaStreamAudioSourceNode
                └── TalkingHead audioCtx (AudioContext)
                     └── HeadAudio AudioWorklet
                          └── head.mtAvatar[viseme_xx].newvalue = n
```

No voice-bridge alignment frames, no ElevenLabs timestamps. The agent's audio
alone drives visemes client-side.

Key files:

- `avatar/web/components/LiveKitAudioBridge.tsx` — subscribes to the agent
  track via `useVoiceAssistant()` and calls `attachAudio` / `detachAudio`.
- `avatar/web/components/TalkingHeadView.tsx` — owns the `TalkingHead`
  instance, registers `headworklet.min.mjs` on `head.audioCtx`, loads
  `model-en-mixed.bin`, routes `onvalue(key, value)` into
  `head.mtAvatar[key].newvalue`.
- `avatar/web/components/Room.tsx` — mounts `<LiveKitAudioBridge>` and
  `<TalkingHeadView>` with a shared `talkingHeadRef`.
- `avatar/web/public/headaudio/{headworklet.min.mjs,model-en-mixed.bin}` —
  copied from `node_modules/@met4citizen/headaudio/dist/` via the `postinstall`
  / `copy:headaudio` script.
- `infra/manifests/voice-bridge/deployment.yaml` — `VOICE_BRIDGE_ENABLE_ALIGNMENT=0`
  (alignment path is off in this deployment).

## 1. Local dev setup

From `avatar/web/`:

```bash
# Dependencies (postinstall copies worklet + model into public/headaudio/)
pnpm install

# Self-hosted LiveKit is not publicly reachable; port-forward in another shell.
kubectl -n cto port-forward svc/livekit-server 7880:80
```

`.env.local` must contain at minimum:

```bash
LIVEKIT_URL=ws://localhost:7880
LIVEKIT_API_KEY=...
LIVEKIT_API_SECRET=...

# Enables the 3D TalkingHead view (without this you get the flat portrait fallback
# and HeadAudio never runs).
NEXT_PUBLIC_AVATAR_RUNTIME=talkinghead
NEXT_PUBLIC_AVATAR_GLB_URL=https://models.readyplayer.me/<id>.glb?morphTargets=ARKit,Oculus%20Visemes&textureAtlas=1024&lod=1
```

Confirm the Python agent worker (`avatar/agent`) is running and registered as
`morgan-avatar`.

Then:

```bash
pnpm dev
# open http://localhost:3000, click "Talk to Morgan"
```

## 2. DevTools observations

Open Chrome DevTools **before** clicking "Talk to Morgan" so you can watch the
bootstrap.

### Network tab

Filter on `headaudio`:

- `GET /headaudio/headworklet.min.mjs` → **200**, `text/javascript` (or `application/javascript`).
- `GET /headaudio/model-en-mixed.bin` → **200**, `application/octet-stream`,
  size ~several MB. A **404** here is the most common failure mode — the
  worklet will still register but viseme detection silently stays flat.

### Console

Expect **no** warnings from `[talkinghead]`. Specifically these lines from
`TalkingHeadView.tsx` should not appear:

- `[talkinghead] failed to attach pending MediaStream`
- `[talkinghead] createMediaStreamSource failed`

### AudioContext state

In the DevTools console once connected:

```js
// Grab the TalkingHead-owned AudioContext via any <audio> element backed by it,
// or easier: inspect the HeadAudio worklet node via performance timeline.
// Simplest check — use the Media panel (More tools → Media) to confirm:
//   1. An AudioContext exists and is in state "running" (not "suspended").
//   2. It has an AudioWorkletNode with a MediaStreamAudioSourceNode upstream.
```

If `AudioContext.state === "suspended"`, browsers require a user gesture first
— clicking **Talk to Morgan** counts, so a stuck "suspended" after clicking is
a bug.

### Morph target values (the actual signal)

Run this in the console after Morgan starts speaking:

```js
// TalkingHead keeps the morph table on the instance; pull it from window if
// you exposed it for debugging, otherwise use the performance profiler
// timeline and watch for the ScriptProcessor/AudioWorklet "onvalue" calls.
const t = window.__talkingHead; // only available if you add it for debug
if (t) {
  ["viseme_sil","viseme_PP","viseme_FF","viseme_TH","viseme_DD","viseme_kk",
   "viseme_CH","viseme_SS","viseme_nn","viseme_RR","viseme_aa","viseme_E",
   "viseme_I","viseme_O","viseme_U"].forEach(k => {
    const v = t.mtAvatar[k];
    console.log(k, v?.newvalue);
  });
}
```

(If `window.__talkingHead` isn't wired, a quick temporary patch in
`TalkingHeadView.tsx` after `headRef.current = head;` — **do not commit** —
helps spot-check values.)

## 3. Expected viseme behaviour

HeadAudio outputs the Oculus 15 viseme set. Rough guide:

| Condition | Expected pattern |
|---|---|
| **Silence** (before Morgan speaks) | `viseme_sil` ≈ 1.0, all others ≈ 0. No flicker. |
| **Agent speaking** | `viseme_sil` drops; several others oscillate between 0 and ~0.6–1.0 as phonemes land. |
| **/p/, /b/, /m/** (bilabials — "Morgan", "problem") | Short peaks on `viseme_PP`. |
| **/f/, /v/** ("five", "voice") | Peaks on `viseme_FF`. |
| **/s/, /z/, /ʃ/** ("system", "sure") | Peaks on `viseme_SS` and `viseme_CH`. |
| **/t/, /d/, /n/, /l/** | Peaks on `viseme_DD` / `viseme_nn`. |
| **Open vowels** (/ɑ/, /æ/) | Peaks on `viseme_aa`. |
| **/i/, /iː/** ("feed") | Peaks on `viseme_I`. |
| **/oʊ/, /u/** ("go", "two") | Peaks on `viseme_O` / `viseme_U`. |

Exact values depend on `vadGateActiveDb=-45` / `vadGateInactiveDb=-60` (set in
`TalkingHeadView.tsx`); thresholds were loosened because LiveKit jitter-buffered
audio is attenuated.

**Pass criterion:** during a multi-word utterance the lips visibly move and at
least 3 distinct non-`sil` visemes cross ~0.3 at some point. Pure "mouth opens
and closes uniformly" is a fail (likely amplitude-driven fallback, not viseme
classification).

## 4. Working vs failing — triage table

| Symptom | Likely cause | Fix |
|---|---|---|
| Viseme values stuck at 0, `viseme_sil` also 0 | HeadAudio never attached a source. | Check `LiveKitAudioBridge` — `audioTrack?.publication?.track?.mediaStreamTrack` was null. Usually means the agent hasn't published audio yet. Reload after agent responds. |
| Only `viseme_sil` ever non-zero, others flat while Morgan clearly speaks | Worklet loaded, model didn't. | Network tab: `model-en-mixed.bin` 404 or CORS. Re-run `pnpm install` (postinstall copies the file) or `pnpm run copy:headaudio`. |
| Console: `The user aborted a request.` on `addModule` | Worklet URL returned non-JS content-type or 404. | Confirm `public/headaudio/headworklet.min.mjs` exists and Next is serving it. Hard-refresh to bypass SW/cache. |
| `[talkinghead] createMediaStreamSource failed` | AudioContext in forbidden state (suspended without gesture, or already closed). | The user must click **Talk to Morgan** before audio arrives; a component remount mid-session can also close the ctx. |
| Lips move but completely out of sync with audio | Not HeadAudio's fault in practice — more likely a GLB with the wrong blendshape names. | Confirm GLB was exported with ARKit 52 + Oculus 15 viseme morph targets. Ready Player Me requires the `morphTargets=ARKit,Oculus%20Visemes` query param. |
| No `<audio>` element audible but visemes animate | Playback renderer not mounted. | `AssistantAudioRenderer` in `Room.tsx` must be present; don't remove it when refactoring — HeadAudio does not play audio itself. |
| Agent audio audible, HeadAudio worklet registered, visemes still flat | VAD gates too strict for this audio level. | Temporarily lower `vadGateActiveDb` / `vadGateInactiveDb` in `TalkingHeadView.tsx`. |

## 5. Rollback criteria → when to enable alignment mode (WS-D)

The alignment-frame path (ElevenLabs timestamp alignment via voice-bridge) is
currently **disabled** in deployment: `VOICE_BRIDGE_ENABLE_ALIGNMENT=0` in
`infra/manifests/voice-bridge/deployment.yaml`.

Re-enable the alignment path (and prioritise WS-D) **only if**:

1. **Multiple** GLB avatars (at least two distinct Ready Player Me heads) show
   flat or obviously wrong visemes despite HeadAudio loading cleanly and
   `model-en-mixed.bin` being served. (Rules out a single-avatar blendshape
   naming issue.)
2. Native-speaker reviewers rate HeadAudio lip-sync qualitatively worse than
   the deterministic portrait on > 30% of utterances.
3. HeadAudio introduces unacceptable latency (> ~100 ms beyond audio) that
   cannot be tuned via `modelFPS` / VAD thresholds.
4. Worklet loading is unreliable across supported browsers (Safari ≥ 16,
   Firefox ≥ 120, Chrome ≥ 116) — i.e. a class of users can't get lip-sync at
   all.

If **none** of the above are observed during WS-A verification, WS-D can be
**dropped** or deferred indefinitely. The alignment path costs ElevenLabs
timestamp API calls and adds a server-side synchronisation surface we don't
need if HeadAudio works.

## 6. Fast smoke (before full manual run)

```bash
avatar/web/scripts/smoke-headaudio.sh
```

Checks static assets exist, the bridge is wired in `Room.tsx`, the worklet is
addressed to `head.audioCtx`, and `pnpm build` succeeds. Passing this is
necessary but not sufficient — a human must still eyeball the avatar.
