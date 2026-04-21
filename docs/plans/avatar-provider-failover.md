# Avatar Provider Failover — Coder Plan (DO + OVH)

> **Audience:** OpenClaw Coder agent. User will hand-deliver this plan; do not
> start work until the user explicitly says "go".
>
> **Out of scope for you (Coder):** Lemon Slice integration — Jonathon owns
> `avatar_providers/lemonslice.py` end-to-end. You implement the shared
> interface and the OVH + DigitalOcean providers only.

## TL;DR

We are building a **three-tier avatar failover chain** on top of the voice
bridge that shipped in PR #4769 (Hermes multi-agent routing):

| Priority | Provider | Kind | Owner | Why |
|---|---|---|---|---|
| 1 | **Lemon Slice** | Hosted SaaS (Video-Only mode, no hosted agent) | Jonathon | Subscribed today; instant availability; reference-quality lipsync |
| 2 | **OVH AI Deploy** | Self-hosted custom container on H100/L40S | **Coder** | ~€15k startup credits; EU; long-term home |
| 3 | **DigitalOcean GPU Droplet** | Self-hosted custom container on L40S | **Coder** | Quota just granted (10 droplets + 2 GPUs); ~$5k Hatch credits |

Each provider implements a single `AvatarProvider` abstraction exported from
`infra/images/voice-bridge/app/avatar_providers/`. The voice bridge's WebSocket
layer hands off a per-turn audio stream to the highest-priority **healthy**
provider; a circuit breaker demotes providers on consecutive failures.

The voice-bridge (STT / LLM / TTS / Hermes routing) is already built — your
work lives exclusively in the **avatar provider layer** plus the OVH/DO
container images and the provisioning scripts.

---

## Context you need

**Already in flight (read before starting):**
- PR #4769 `feat/voice-bridge-hermes` — Hermes routing, auth, rate limit,
  per-agent voice + `turn_metrics`. Based on `feat/coder-welcome-gitlab-swap`.
  **Branch your work off `feat/voice-bridge-hermes`** (or off whatever branch
  it lands on after merge — check with the user).
- `infra/images/voice-bridge/app/voice_agents.py` — `AgentSpec` frozen
  dataclass + registry loader. Your provider registry should follow the same
  pattern.
- `infra/images/voice-bridge/app/main.py` — current WS endpoint; you will add
  a video-stream fork, not rewrite anything.

**Background (session artifacts the human drafted; summaries below):**
- Lemon Slice API: Daily.co WebRTC room OR `livekit-agents[lemonslice]`
  plugin. "Video-Only mode" means Lemon Slice renders the avatar only — we
  own STT + LLM + TTS. Reference image is **368 × 560**, auto center-crop.
  Idle timeout default 60 s; GPU hard timeout 30 min.
- OVH AI Deploy: CaaS — bring your own Docker image, attach weights from S3
  Object Storage, `ovhai app run` returns a managed HTTPS endpoint. H100 80GB
  = €2.80/hr, L40S 48GB = €1.40/hr. Scale to 0 when idle.
- DigitalOcean: `doctl compute droplet create --size gpu-l40sx1-48gb` now
  allowed per support ticket (Shubham, 2026-04-21). L40S $1.57/hr. No
  Gradient AI; custom containers only.

**Hard guardrails — identical to the voice-bridge prompt you already ran:**
- **NEVER** `kubectl delete` node / pv / pvc / namespace.
- **NEVER** force-push protected branches (`main`, `release-please--*`).
- **NEVER** modify Cloudflare DNS without an explicit user "yes".
- Payment-affecting actions (provisioning a GPU droplet, launching an
  `ovhai app run`, bulk Object Storage upload) **require the user's explicit
  "yes"** in-session. Dry-run first, print the estimated burn rate, then
  wait.
- **Stop at each D-boundary, summarise what landed, await go-ahead** before
  starting the next D.
- Do **not** resume the abandoned MuseTalk / EchoMimic v3 / hunyuan-worker
  in-cluster spike. That's archived.

---

## Architecture

### 1. Shared `AvatarProvider` abstraction

New package: `infra/images/voice-bridge/app/avatar_providers/`

```
avatar_providers/
├── __init__.py
├── base.py          # Protocol + AvatarSession, shared types
├── registry.py      # loads AVATAR_PROVIDERS_JSON, priority-sorted
├── failover.py      # circuit breaker + try-next-on-failure
├── lemonslice.py    # STUB ONLY (Jonathon owns the impl)
├── ovh_worker.py    # HTTP/WS client → our container on OVH AI Deploy
└── do_worker.py     # HTTP/WS client → our container on DO GPU droplet
```

`base.py` — signatures you MUST NOT change without the user's sign-off (these
are contract boundaries for the Lemon Slice impl Jonathon is writing in
parallel):

```python
from __future__ import annotations

from dataclasses import dataclass
from typing import AsyncIterator, Protocol, runtime_checkable

from .voice_agents import AgentSpec  # reuse existing dataclass


@dataclass(frozen=True, slots=True)
class VideoFrame:
    data: bytes          # H.264 NAL unit OR JPEG bytes (see content_type)
    content_type: str    # "video/h264" | "image/jpeg"
    pts_ms: int          # presentation timestamp, ms since session start
    is_keyframe: bool


@dataclass(frozen=True, slots=True)
class AvatarProviderConfig:
    name: str            # "lemonslice" | "ovh-musetalk" | "do-musetalk"
    kind: str            # "hosted" | "self-hosted-ovh" | "self-hosted-do"
    priority: int        # 1 = try first, N = try last
    endpoint: str        # https://... for self-hosted; scheme-specific otherwise
    token: str           # bearer token; "" if unauthenticated
    extras: dict         # provider-specific knobs (voice_id overrides, etc.)


@runtime_checkable
class AvatarSession(Protocol):
    """One turn's worth of video streaming. NOT reused across turns."""

    async def push_audio(self, chunk: bytes, *, is_final: bool = False) -> None: ...
    def __aiter__(self) -> AsyncIterator[VideoFrame]: ...
    async def close(self) -> None: ...


@runtime_checkable
class AvatarProvider(Protocol):
    config: AvatarProviderConfig

    async def healthz(self) -> bool:
        """Cheap liveness probe. <=500 ms budget. Called by failover layer."""

    async def start_session(
        self,
        *,
        agent: AgentSpec,
        reference_image_url: str,
        session_id: str,
    ) -> AvatarSession: ...

    async def aclose(self) -> None:
        """Tear down persistent connections (pools, etc.). Idempotent."""
```

**Contract rules** (call these out in `base.py` as module-level docstring):

1. `push_audio` accepts raw PCM16 mono @ 16 kHz OR Opus-in-WebM (the voice
   bridge will tell the session which via the `extras["audio_mime"]` field at
   provider construction time — do not renegotiate per turn).
2. `VideoFrame.pts_ms` MUST be monotonic within a session; the client uses
   it for lipsync drift detection.
3. `close()` MUST drain and return within 2 s; hard-cancel any upstream WS
   if that budget is blown.
4. Providers MUST be safe to instantiate at bridge startup even if their
   backend is down. `healthz()` is the gate, not the constructor.

### 2. Registry + env config

```
VOICE_AGENTS_JSON           (existing; AgentSpec registry)
AVATAR_PROVIDERS_JSON       (NEW; AvatarProviderConfig list)
AVATAR_FAILOVER_DISABLED    (NEW; "1" to force first-priority only, for tests)
AVATAR_CIRCUIT_FAILURES     (NEW; default 3 — consecutive fails → demote)
AVATAR_CIRCUIT_COOLDOWN_S   (NEW; default 60 — seconds before a demoted
                             provider is retried)
```

`AVATAR_PROVIDERS_JSON` example:

```json
{
  "providers": [
    {
      "name": "lemonslice",
      "kind": "hosted",
      "priority": 1,
      "endpoint": "https://api.lemonslice.com",
      "token_env": "LEMONSLICE_API_KEY",
      "extras": {"audio_mime": "audio/webm;codecs=opus"}
    },
    {
      "name": "ovh-musetalk",
      "kind": "self-hosted-ovh",
      "priority": 2,
      "endpoint": "https://abcd1234.app.gra.ai.cloud.ovh.net",
      "token_env": "AVATAR_OVH_TOKEN",
      "extras": {"audio_mime": "audio/pcm;rate=16000"}
    },
    {
      "name": "do-musetalk",
      "kind": "self-hosted-do",
      "priority": 3,
      "endpoint": "https://avatar-nyc2.5dlabs.ai",
      "token_env": "AVATAR_DO_TOKEN",
      "extras": {"audio_mime": "audio/pcm;rate=16000"}
    }
  ]
}
```

### 3. Failover behaviour (`failover.py`)

- Wrap the sorted provider list. For each turn:
  1. Pick the highest-priority provider whose circuit is **closed**.
  2. Call `healthz()` with a 500 ms timeout; if False → trip breaker, move on.
  3. Call `start_session()`; on exception → trip breaker, move on.
  4. If `push_audio` / iteration raises mid-turn, surface a `provider_failed`
     event to the client and DO NOT retry mid-turn (partial video is worse
     than a clean fail-fast).
- Circuit breaker: after `AVATAR_CIRCUIT_FAILURES` consecutive failures,
  demote for `AVATAR_CIRCUIT_COOLDOWN_S`. On cooldown expiry, `healthz()`
  probe — success → close circuit.
- Emit structured log line on every state transition:
  `{"event": "avatar_provider", "name": ..., "state": "healthy|tripped|recovered", "reason": ...}`
- Emit `turn_metrics` extension: `{"avatar_provider": <name>, "avatar_start_ms": 123, "avatar_first_frame_ms": 456}`.

### 4. Voice-bridge WS integration

- Client opts in to video via `{"type": "start", "session_id": "...", "want_video": true}`.
- If `want_video=false` OR `AVATAR_PROVIDERS_JSON` empty → current audio-only
  path unchanged.
- If `want_video=true`: bridge opens an avatar session via the failover
  layer, forks every audio chunk it gets from the client into both (a) the
  STT path and (b) `session.push_audio(chunk)`. Video frames are relayed
  back to the client as binary WS frames prefixed with a 1-byte opcode:

  ```
  0x01  [N-byte JSON header]  [M-byte payload]   # text frame
  0x02  [payload]                                 # audio mp3 (existing)
  0x03  [4-byte BE length][header JSON][frame bytes]  # NEW: video frame
  ```

  The JSON header is `{"content_type": "video/h264", "pts_ms": 123, "is_keyframe": true}`.

  **Do not** break the existing `0x02` audio contract. Client fallback path
  must still work when the provider layer is unavailable.

---

## Deliverables (stop-and-review at each boundary)

### D1 — Shared abstraction + failover core

**Files to create:**
- `infra/images/voice-bridge/app/avatar_providers/__init__.py`
- `infra/images/voice-bridge/app/avatar_providers/base.py` (verbatim from the
  "Architecture §1" block above; any change requires user sign-off)
- `infra/images/voice-bridge/app/avatar_providers/registry.py`
- `infra/images/voice-bridge/app/avatar_providers/failover.py`
- `infra/images/voice-bridge/app/avatar_providers/lemonslice.py` — **stub
  only**: `class LemonSliceProvider: ...` that raises `NotImplementedError`
  from every method EXCEPT the dataclass wiring. Jonathon fills this in.

**Tests:** `infra/images/voice-bridge/tests/test_avatar_providers_core.py`
- Registry loads from env, sorts by priority, resolves `token_env` to secret.
- Failover demotes after `AVATAR_CIRCUIT_FAILURES`; recovers after cooldown.
- Unknown provider name in `AVATAR_PROVIDERS_JSON` is logged and skipped,
  not fatal.

**Exit criteria:** pytest green; no real network calls.

### D2 — Avatar worker container (shared by OVH and DO)

Single image, two hosts. Model is env-selected.

**Directory:** `infra/images/avatar-worker/`

```
Dockerfile                       # CUDA 12.1, python 3.10, ffmpeg
requirements.txt                 # fastapi, uvicorn, httpx, torch, model deps
app/
  __init__.py
  main.py                        # FastAPI: GET /healthz; POST /v1/sessions; WS /v1/sessions/{id}/stream
  models/
    base.py                      # TalkingHeadModel Protocol
    musetalk.py                  # MuseTalk v1.5 adapter (PRIMARY for MVP)
    hunyuan.py                   # HunyuanVideo-Avatar adapter (STRETCH)
  encoder.py                     # h264 NAL unit chunking via av (PyAV) / ffmpeg-python
entrypoint.sh
README.md                        # env vars, model selection, weight paths
```

**Env vars:**
- `AVATAR_MODEL` — `musetalk` (default) | `hunyuan`
- `AVATAR_WEIGHTS_DIR` — `/opt/ml/model` (OVH bucket mount)
- `AVATAR_DEFAULT_IMAGE_URL` — fallback reference portrait
- `AVATAR_MAX_CONCURRENT_SESSIONS` — default 1
- `AVATAR_SHARED_SECRET` — bearer token; required, no default

**HTTP API:**

```
GET  /healthz
  → 200 {"status": "ok", "model": "musetalk", "gpu": "L40S"}

POST /v1/sessions
  headers: Authorization: Bearer $AVATAR_SHARED_SECRET
  body: {"session_id": "...", "reference_image_url": "https://...", "audio_mime": "audio/pcm;rate=16000"}
  → 200 {"session_id": "...", "ws_url": "wss://<host>/v1/sessions/<id>/stream"}

WS /v1/sessions/{id}/stream
  client → server: binary audio chunks OR {"type":"end_utterance"}
  server → client: binary framed as described in "Voice-bridge WS
                   integration" §4 opcode 0x03 (so the provider client can
                   forward frames verbatim). Sends {"type":"done"} text
                   frame when render completes.
```

**Model selection rationale (justify in README.md):**
- **MuseTalk v1.5 = MVP primary.** Proven, runs in 16 GB VRAM on L40S,
  human lipsync only. Fastest to ship.
- HunyuanVideo-Avatar = stretch. Higher quality, ~50 GB weights, needs H100
  80GB. Swap in via env once the pipeline is proven.
- Do NOT ship EchoMimicV3 in this PR — DD in
  `session:files/echomimicv3-dd-findings.md` says it's not real-time on any
  realistic GPU and the anime/animal-face gate is open on issue #33.

**Tests:** `infra/images/avatar-worker/tests/`
- Unit: encoder produces monotonic `pts_ms`, emits at least one keyframe per
  2 s window.
- Integration (behind `pytest -m gpu`): 5-second reference image + 5-second
  audio → > 100 frames returned, first frame < 3 s wall-clock.

**Exit criteria:** `docker build` green; `docker run --gpus all` serves
`/healthz` locally on a machine with an NVIDIA GPU; unit tests pass on CPU.

### D3 — OVH AI Deploy provisioning

**Directory:** `infra/avatar/ovh/`

```
Makefile                         # build-image, upload-weights, deploy, status, logs, teardown
scripts/
  ovhai-login.sh                 # sources OVH_AI_ENDPOINTS_TOKEN from 1Password
  ovhai-deploy.sh                # wraps `ovhai app run`
  ovhai-upload-weights.sh        # aws-cli → OVH Object Storage S3
cloud-config/
  app-config.yaml                # static flavor / replica config
README.md                        # step-by-step runbook + credit burn math
```

**Makefile targets (all idempotent, print dry-run plan before mutating):**

```make
build-image:          # docker build + push to ghcr.io/5dlabs/avatar-worker:$(GIT_SHA)
upload-weights:       # aws s3 sync ./weights s3://5dlabs-avatar/musetalk-v1.5/
deploy:               # ovhai app run ... --gpu 1 --flavor l40s-1-gpu --unsecure-http=false
status:               # ovhai app get <name> --output json
logs:                 # ovhai app logs <name> --follow
teardown:             # ovhai app delete <name>  (PROMPTS for confirmation)
```

**Deployment flavor default:** `l40s-1-gpu` (€1.40/hr). H100 is gated behind
`make deploy FLAVOR=h100-1-gpu` and prints a red warning showing the €2.80/hr
rate before proceeding.

**Secrets (documented in README):**
- `OVH_AI_ENDPOINTS_TOKEN` — already in 1Password per prior session.
- `AVATAR_OVH_BUCKET_ACCESS_KEY` / `SECRET_KEY` — create during D3 if missing.
- `AVATAR_SHARED_SECRET` — generated per deploy, stored in Kubernetes secret
  `cto/avatar-ovh-token` (you create the secret manifest in `infra/charts/...`
  as part of D5, not D3).

**Tests:** `infra/avatar/ovh/tests/test_makefile.bats` (bash + bats-core)
- `make deploy --dry-run` prints the `ovhai` command without executing it.
- `make teardown` refuses if `CONFIRM_TEARDOWN=yes` is not set.

**Exit criteria:** `make build-image` + `make upload-weights --dry-run` both
pass in CI on a runner with no OVH credentials (dry-run only); real
provisioning happens only when the user runs `make deploy` locally AFTER D5
integration is proven.

### D4 — DigitalOcean GPU Droplet provisioning

**Directory:** `infra/avatar/digitalocean/`

```
Makefile                         # provision, snapshot, destroy, ssh
scripts/
  doctl-provision.sh             # doctl compute droplet create
  doctl-snapshot.sh              # doctl compute image create-from-droplet
  doctl-destroy.sh               # PROMPTS for confirmation
cloud-init.yaml                  # nvidia-container-toolkit, docker, pull image, systemd unit
terraform/                       # OPTIONAL alternative (gated behind user ack)
  main.tf
  variables.tf
  outputs.tf
README.md                        # runbook + quota notes
```

**Makefile defaults:**
- Region: `nyc2` (user requested; L40S available per support)
- Size: `gpu-l40sx1-48gb`
- Image: `gpu-h100x1-base-ubuntu-2204` (correct base for GPU droplets)
- Volume: 100 GB attached, tagged `avatar-weights`

**Cloud-init duties:**
1. Install `nvidia-container-toolkit` and `docker`.
2. `docker login ghcr.io` (read-only deploy token).
3. `docker pull ghcr.io/5dlabs/avatar-worker:$TAG`.
4. Write systemd unit `avatar-worker.service` that runs the container with
   `--gpus all` + env vars + the attached volume mounted at `/opt/ml/model`.
5. Open firewall: 22/tcp from our IP allowlist ONLY, 443/tcp from Cloudflare
   ranges ONLY. No `0.0.0.0/0`.

**Security rails:**
- `doctl-destroy.sh` requires both `--yes` and `CONFIRM_DROPLET_ID=<id>` to
  proceed. Never delete by name.
- `doctl-snapshot.sh` tags the image with `created-by=avatar-failover` so
  the user can audit.

**Tests:** `infra/avatar/digitalocean/tests/test_scripts.bats`
- `make provision --dry-run` prints the `doctl` command and expected $/hr.
- `make destroy` without `CONFIRM_DROPLET_ID` errors out.

**Exit criteria:** same as D3 — dry-run only in CI, live provisioning
gated on user "yes".

### D5 — Voice-bridge integration + Kubernetes plumbing

**Voice-bridge diff (`infra/images/voice-bridge/app/main.py`):**
1. Import the registry + failover layer.
2. On WS `start` with `want_video=true`, call
   `failover.start_session(agent=agent_spec, reference_image_url=agent_spec.extras.get("avatar_image"), session_id=session_id)`.
3. Fork audio: the existing `push_stt(audio)` gets a sibling
   `push_avatar(audio)` (fire-and-forget task).
4. On video frame from the provider's `__aiter__`, encode as opcode `0x03`
   framed binary and `ws.send_bytes(...)` it.
5. `turn_done` waits for both TTS completion AND `session.close()`.

**Helm chart diff (`infra/charts/openclaw-agent/`):**
- Add `voice-bridge-avatar-providers` ConfigMap with `AVATAR_PROVIDERS_JSON`.
- Add 3 Secrets references: `LEMONSLICE_API_KEY`, `AVATAR_OVH_TOKEN`,
  `AVATAR_DO_TOKEN`. Do NOT commit real values — reference existing
  External Secrets or document the `kubectl create secret` commands.
- New values block `avatar.enabled=false` by default. Set true in a
  follow-up commit only after the user approves the deploy.

**AgentSpec extension (`infra/images/voice-bridge/app/voice_agents.py`):**
- Add optional field: `avatar_image: str | None = None` (URL to 368×560
  reference portrait).
- Extend `AgentSpec` without breaking the existing frozen+slots contract:
  ensure default value handling works on both Morgan and Hermes presets.

**Tests:** `infra/images/voice-bridge/tests/test_avatar_integration.py`
- Smoke: `want_video=true` with all providers tripped → bridge sends
  `{"type":"error","error":"avatar_unavailable"}` and continues audio-only.
- Smoke: `want_video=true` with one healthy mock provider → at least one
  0x03 frame received by the test client.
- Back-compat: `want_video=false` OR field absent → no avatar session
  created; existing tests from PR #4769 still pass.

**Exit criteria:** pytest green; helm lint + template on `openclaw-agent`
chart clean.

### D6 — Runbook + ops

**New doc:** `docs/avatar-provider-failover-runbook.md`
- Provider selection decision tree.
- How to trip a circuit manually (for testing).
- How to scale to 0 on OVH and DO (save credits).
- Credit burn dashboards / alerts (promote from scratch, not required to
  wire to Grafana in this PR).
- Escalation: who does what when Lemon Slice goes down mid-demo.

**Exit criteria:** doc lands; user explicitly acks "I can run this without
pinging you" before we call D6 done.

---

## Branching + PR strategy

- Base branch: **`feat/voice-bridge-hermes`** (PR #4769 head). If that
  merges into `feat/coder-welcome-gitlab-swap` before you start, base off
  that instead — ask the user.
- Working branch: `feat/avatar-provider-failover` off the base above.
- Open **ONE PR per D** against the base branch. Do not stack PRs on each
  other — each D is reviewable standalone.
- PR titles: `avatar-failover(D1): shared abstraction` etc.
- PR body must include:
  - What landed vs. the D deliverable list
  - Which env vars / secrets / configs are net-new
  - Tests run + output (paste pytest summary)
  - Any deviation from the plan (and why)

---

## What Jonathon is doing in parallel

- Implementing `avatar_providers/lemonslice.py` using
  `livekit-agents[lemonslice]` Video-Only mode (no hosted agent).
- Client-side Tauri WebRTC rendering (WKWebView video element).
- Cloudflare tunnel for `morgan-voice.5dlabs.ai` (pending his explicit
  "yes" — do not provision DNS).

---

## Known unknowns (flag these back to the user as you hit them)

1. **L40S in NYC2 vs TOR1** — earlier OVH/DO correspondence said L40S is
   TOR1-only on DO. Confirm with `doctl compute size list` before D4.
2. **Weights hosting** — MuseTalk v1.5 weights are ~1.5 GB; HunyuanVideo is
   ~50 GB. For D3/D4 MVP stick with MuseTalk (S3 push <5 min). Push
   HunyuanVideo only if user asks for the stretch.
3. **CORS + auth on WS** — OVH `--unsecure-http` defaults allow public
   reachability; we want bearer auth enforced even with the managed HTTPS
   endpoint. Implement in `app/main.py` of the worker image (D2), not at
   the OVH layer.
4. **Cost ceiling** — please surface estimated $/hr at every deploy
   invocation. User will reject anything that exceeds $5/hr without
   explicit opt-in.

---

## Rollback

Every piece of this plan is reversible:
- D1–D2 are additive; they don't change the audio-only path.
- D3–D4 are infrastructure scripts; `make teardown` destroys the resource.
- D5 is feature-flagged behind `avatar.enabled=false`.
- D6 is documentation.

If any D blocks on reality (e.g., HunyuanVideo-Avatar won't fit on L40S),
**stop, summarise the wall, propose alternatives** — do not improvise.
