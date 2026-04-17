# PRD — Full Self-Hosted Avatar Pipeline (Coder, Autonomous)

> **Scope change (2026-04-17):** This PRD previously covered only Phase 4 (GPU provisioning). It now covers the **entire** self-hosted avatar pipeline, Phases 1–6. Coder owns the full implementation and runs **autonomously to completion** across reboots. Nothing below is out of scope. If you think it's out of scope, you're wrong — read "Autonomous Operation" and keep going.

## Mission
Replace LemonSlice + LiveKit Cloud with a fully self-hosted avatar stack on our OVH RKE2 cluster. Only acceptable external dependency is ElevenLabs TTS. Target: Morgan's talking avatar rendered by MuseTalk 1.5 on a V100S GPU, streamed over self-hosted LiveKit, managed via an admin UI protected by Better Auth.

You are **Coder**. You drive every phase below, verify each one before advancing, post Discord updates per phase transition, and persist your state so a reboot never loses progress.

---

## Autonomous Operation (read this first, obey it always)

You already have two skills wired into the openclaw-agent chart. Use them — don't reinvent them.

1. **`infra/charts/openclaw-agent/skills/openclaw/reboot-continuity.md`** — on every fresh session (no prior context in your prompt), before answering anything:
   - `cat /workspace/HANDOFF.md`
   - Check mem0 for recent entries under `userId: jonathon:agent:coder`
   - `git status && git stash list && git -C /workspace branch --sort=-committerdate | head -20`
   - Resume the active work **immediately, without asking for permission**. If HANDOFF and mem0 disagree, trust the one with the most recent timestamp.

2. **`infra/charts/openclaw-agent/skills/openclaw/session-persistence.md`** — mem0 auto-flushes on compaction, auto-captures every turn, auto-recalls in new prompts. **In addition**, you manually checkpoint every 10–15 tool calls via `memory_add` under `userId: jonathon:agent:coder` with tags matching the current phase (e.g. `phase:3`, `phase:3.5`).

3. **Canonical state file: `/workspace/HANDOFF.md`.** After every meaningful milestone (phase advance, acceptance gate passed, failure that blocks progress), rewrite HANDOFF.md with this exact structure:

   ```markdown
   # Coder WIP — Self-Hosted Avatar
   Updated: <UTC ISO timestamp>
   Branch: <current branch>
   Current phase: <1 | 2 | 3 | 3.5 | 4 | 5 | 6>
   Current phase status: <in-progress | blocked:<reason> | verified>
   Next step: <single concrete action>
   Last verification run: <acceptance check name + result>
   Open PRs: <list>
   Notes: <anything future-you needs>
   ```

4. **On boot with a stale HANDOFF** (older than 24h OR phase field missing): pick the **highest-numbered phase whose self-verification gate has not passed**, resume there. Do not restart from Phase 1.

5. **Never stop mid-phase asking "should I continue?"** If a step is blocked on a credential, run the credential-recovery steps in "Credentials" below. If it's blocked on user approval, post a Discord message tagging the user **and keep working on any unblocked parallel work** (e.g. if Phase 4 is blocked on quota, push Phase 3 code).

6. **Progress cadence:** one Discord message per phase transition (start + verified) in Coder's intake thread. One concise status line per verification failure with what you tried and what you're trying next.

---

## Self-Verification Gates (MANDATORY — do not advance without passing)

Each phase has a machine-checkable gate. On failure: loop with remediation, do **not** skip ahead. Record every run in HANDOFF.md `Last verification run`.

| Phase | Gate |
|-------|------|
| 1 — LiveKit | `lk room list --url wss://lk.5dlabs.ai --api-key $LK_KEY --api-secret $LK_SECRET` returns 0 exit; `lk room create test-room` + `lk room delete test-room` succeed. |
| 2 — Agent wired to self-hosted LK | Deploy agent with `LIVEKIT_URL=wss://lk.5dlabs.ai` and `MORGAN_AVATAR_MODE=disabled`; from the browser frontend, audio-only session completes a full STT → LLM → TTS round-trip. Log: `audio_roundtrip_ms < 3000`. |
| 3 — MuseTalk plugin code | `pytest avatar/agent/tests/test_musetalk_avatar.py` passes with mocked GPU (frames produced at ≥28fps sustained against a fixture audio stream). |
| 3.5 — Persona admin | Upload → preprocess (NATS) → status flips to `ready` → `/personas/<id>/preview.mp4` exists on PVC → admin UI list shows persona as `ready`. Exercised via Playwright or plain `curl` + poll loop. |
| 4 — GPU provision | In the GPU pod: `nvidia-smi` shows V100S; `python -c "import torch; assert torch.cuda.is_available()"` exits 0; MuseTalk model loads without OOM. |
| 5 — E2E latency | Full browser round-trip with video; measured `audio_to_first_frame_ms < 500` over 10 sequential utterances. Render cache hit on 2nd identical utterance (`cache_hit=true` in logs). |
| 6 — Cutover | `avatar/agent/requirements*.txt` contains no `livekit-plugins-lemonslice` entry; E2E gate from Phase 5 still passes; `avatar/docs/` contains runbook + architecture diagram; `MORGAN_AVATAR_MODE=musetalk` is default in production values. |

If a gate is flaky, fix the flake, don't lower the bar.

---

## Context & Preconditions

- **Cluster**: 4× b3-64 CPU nodes in OVH GRA9, RKE2 v1.34.5, Ubuntu 22.04. Control plane: `https://10.0.0.181:9345`.
- **Project ID**: `6093a51de65b458e8b20a7c570a4f2c1`.
- **GPU**: quota approved (CS15510375); deploy a t2-45 (V100S 32GB) in GRA9. Prior stale GPU `db9e5d60-...` was deleted.
- **GPU Operator + NFD**: already deployed via ArgoCD, standby. Node selector: `feature.node.kubernetes.io/pci-10de.present: true`.
- **LLM**: OpenClaw Morgan at `morgan.5dlabs.ai` (self-hosted, working).
- **TTS**: ElevenLabs (voice `iP95p4xoKVk53GoZ742B`, model `eleven_flash_v2_5`). API key in 1Password `Automation` vault.
- **STT**: Deepgram via LiveKit (or direct). Key in 1Password.
- **NATS**: already running cluster-wide, reuse it.
- **Budget ceiling**: **$40 CAD/day**, ~$893/month. Startup credits $14,377 CAD, expire March 2027. **Tear down GPU if idle > 8h.** Re-provision on demand via the OVH API signer below.

---

## Credentials

All secrets live in 1Password `Automation` vault. `OP_SERVICE_ACCOUNT_TOKEN` is mounted in your pod — `op read` works directly, no interactive login.

Needed items:
- `op://Automation/OVH CA API` — fields: `application_key`, `application_secret`, `consumer_key`, `endpoint` (use `ovh-ca`)
- `op://Automation/OVH GRA9 GPU SSH` — private key for SSH to provisioned GPU node
- `op://Automation/RKE2 Join Token` — `token`, `server_url` (`https://10.0.0.181:9345`)
- `op://Automation/ElevenLabs` — `api_key`
- `op://Automation/Deepgram` — `api_key`
- `op://Automation/GitHub PAT (Coder)` — `token` (push, PR, merge)
- `op://Automation/LiveKit Self-Hosted` — `api_key`, `api_secret` (create if missing: generate `lk create-token --help` or use openssl).

If any item is missing, create it: `op item create --category=login --vault=Automation --title='LiveKit Self-Hosted' api_key=<val> api_secret=<val>`.

---

## OVH API signer (canonical helper — reuse across phases)

```bash
ovh_call() {
  local method="$1"; local path="$2"; local body="${3:-}"
  local ts; ts=$(curl -s https://ca.api.ovh.com/1.0/auth/time)
  local sig="\$1\$$(printf '%s+%s+%s+https://ca.api.ovh.com/1.0%s+%s+%s' \
    "$OVH_AS" "$OVH_CK" "$method" "$path" "$body" "$ts" | sha1sum | awk '{print $1}')"
  curl -sS -X "$method" "https://ca.api.ovh.com/1.0$path" \
    -H "X-Ovh-Application: $OVH_AK" \
    -H "X-Ovh-Consumer: $OVH_CK" \
    -H "X-Ovh-Timestamp: $ts" \
    -H "X-Ovh-Signature: $sig" \
    -H "Content-Type: application/json" \
    ${body:+--data "$body"}
}
# Load creds from 1Password
export OVH_AK=$(op read "op://Automation/OVH CA API/application_key")
export OVH_AS=$(op read "op://Automation/OVH CA API/application_secret")
export OVH_CK=$(op read "op://Automation/OVH CA API/consumer_key")
```

---

## Build & deploy environment (in-pod quirks — do NOT skip)

- **Kaniko sidecar** handles container builds inside your pod. It is **pre-authed only to `ghcr.io/5dlabs/*`**. Push images there. Do not attempt `docker` — the socket is not available.
- **kubectl**: the baked kubeconfig has a stale token. Use the **projected ServiceAccount token** at `/var/run/secrets/kubernetes.io/serviceaccount/token` plus `--server=https://kubernetes.default.svc`. PRs #4660 (kubeconfig fix) and #4662 (RBAC grants) are already merged on main — if you see `Unauthorized`, rebase first; if still broken, the in-cluster token path above always works.
- **Helm** is available; ArgoCD Application manifests live in `infra/charts/` and `infra/argocd/`.
- **`lk` CLI** available via `go install github.com/livekit/livekit-cli/cmd/lk@latest` if missing.

---

## Phase 1 — Self-hosted LiveKit server

**Deliverables**
- ArgoCD Application + Helm values for `livekit/livekit-server` chart from `helm.livekit.io` in namespace `livekit`.
- `podHostNetwork: true`, UDP 50000–60000 exposed on the node (open OVH firewall via `ovh_call` if needed).
- Small Redis (new or reused) in `livekit` ns for room state.
- DNS `lk.5dlabs.ai` → node public IP (Cloudflare record). TLS via cert-manager issuer already in cluster.
- Secret `livekit-keys` with `api_key` / `api_secret` sourced from 1Password.
- Prometheus ServiceMonitor enabled.

**Gate:** see table above (Phase 1). Exit only when `lk room create/delete` succeed against `wss://lk.5dlabs.ai`.

---

## Phase 2 — Point agent + web at self-hosted LiveKit

**Deliverables**
- `avatar/web/app/api/token/route.ts` reads `LIVEKIT_URL`, `LIVEKIT_API_KEY`, `LIVEKIT_API_SECRET` from env (no hard-coded cloud URL). K8s secret `livekit-client-config` wires them in.
- Agent Deployment env updated with the same three vars.
- Feature-flag knob `MORGAN_AVATAR_MODE` defaults to `lemonslice` until Phase 3 lands, so Phase 2 does not regress video. Introduce a `disabled` value that runs audio-only for the gate.

**Gate:** Phase 2 row in the table. Must demonstrate an audio-only session completes end-to-end against self-hosted LK.

---

## Phase 3 — MuseTalk streaming plugin

**Deliverables**
- `avatar/agent/morgan_avatar_agent/musetalk_avatar.py` — custom avatar plugin. Uses `livekit-rtc` `VideoSource` + `LocalVideoTrack.create_video_track`, `source.capture_frame(rtc.VideoFrame(...))`. Subscribes to the TTS audio stream and pushes frames at 30fps.
- `avatar/agent/morgan_avatar_agent/musetalk_inference.py` — streaming engine wrapping MuseTalk 1.5's `realtime_inference.py`. Ring buffer for audio → Whisper features → UNet → VAE decode → RGBA. Pre-loads persona latents once at startup (path from `MORGAN_PERSONA_ID`).
- `avatar/agent/agent.py` reads `MORGAN_AVATAR_MODE=musetalk|lemonslice|disabled` and selects the right session.
- `avatar/agent/requirements.txt`: add `livekit-rtc`, `torch`, `opencv-python`, `transformers`, `mediapipe` (or face-alignment). Move `livekit-plugins-lemonslice` to a separate `requirements-legacy.txt` (kept for emergency rollback; removed in Phase 6).
- `avatar/agent/tests/test_musetalk_avatar.py` with a CPU-mockable inference stub so the gate runs without GPU.

**Gate:** Phase 3 row. Unit test must sustain ≥28fps against a fixture audio stream.

---

## Phase 3.5 — Persona admin (upload + preprocess + Better Auth)

**Deliverables**
- **PVC** `personas-pvc` (ReadWriteMany) in `avatar` ns backed by OVH block/object storage. Schema:
  ```
  /personas/<persona_id>/
    source.{png,jpg,mp4}
    metadata.json
    latents/
    landmarks/
    coords.pkl
    mask/
    preview.mp4
    status.json   # { state: "ready"|"preprocessing"|"failed", error?: string }
  ```
- **Preprocess worker** `avatar/agent/morgan_avatar_agent/persona_preprocess.py`. Subscribes to NATS subject `avatar.persona.preprocess` with a durable consumer + ack. Runs face detection → bbox crop → VAE encode → landmarks → writes all artefacts → flips `status.json` to `ready`, renders 5s `preview.mp4`. On failure, writes `{state: "failed", error: "..."}` and nacks for retry (max 3). Fallback noted in code comments: if NATS causes backpressure/ordering pain, swap to K8s Job API.
- **Admin API routes** under `avatar/web/app/api/admin/personas/`: `POST` (multipart, ≤50MB, video ≤30s), `GET` list, `GET /<id>`, `DELETE /<id>`. All protected.
- **Better Auth** (https://better-auth.com) wired into `avatar/web`. Email/password + GitHub social provider. Sessions in Postgres (small StatefulSet in `avatar` ns, or reuse an existing Postgres). Admin role check on `/api/admin/*`. Sign-in page at `/admin/login`.
- **Admin UI** `avatar/web/app/admin/personas/page.tsx`. Drag-drop upload (png/jpg/webp/mp4≤30s), list with status badges, thumbnail, delete, 2s polling while `preprocessing`.
- **Runtime selection**: `MORGAN_PERSONA_ID` env (default = first `ready` persona). Token API accepts optional `personaId` query param. Avatar agent loads `/personas/<id>/latents/` + `coords.pkl` at startup.
- **Bootstrap**: convert the existing Morgan LemonSlice reference image into persona `morgan-v1` via the new pipeline. This is the end-to-end validation.

**Gate:** Phase 3.5 row. Full upload → `ready` → visible in UI exercised by an integration test or a scripted `curl` loop.

---

## Phase 4 — GPU provision & deploy (original PRD scope, preserved)

**Work items**
1. **Provision** a t2-45 (V100S 32GB) in GRA9 via `ovh_call POST "/cloud/project/$PROJECT_ID/instance"` with flavor `t2-45`, image name `Ubuntu 22.04`, SSH key from 1Password, name `musetalk-gpu-1`. Wait for `ACTIVE`.
2. **Join RKE2**: `ssh ubuntu@<pub-ip>` with the 1Password SSH key, then
   ```bash
   curl -sfL https://get.rke2.io | INSTALL_RKE2_VERSION=v1.34.5+rke2r1 INSTALL_RKE2_TYPE=agent sh -
   sudo mkdir -p /etc/rancher/rke2
   cat <<EOF | sudo tee /etc/rancher/rke2/config.yaml
   server: https://10.0.0.181:9345
   token: <RKE2 Join Token from 1Password>
   EOF
   sudo systemctl enable --now rke2-agent
   ```
3. **Verify GPU Operator** auto-labels the node: `kubectl get nodes -L nvidia.com/gpu.present` shows `true`; `kubectl describe node musetalk-gpu-1 | grep nvidia.com/gpu` shows `1` allocatable.
4. **Build & push** `ghcr.io/5dlabs/musetalk-worker:<sha>` via Kaniko. Base: `nvidia/cuda:11.8.0-cudnn8-runtime-ubuntu22.04`. Install: Python 3.10, PyTorch 2.1 + CUDA 11.8, MuseTalk 1.5 weights pre-baked to `/models`.
5. **Deploy** agent to GPU node via ArgoCD Application `avatar-agent-gpu`. `nodeSelector: feature.node.kubernetes.io/pci-10de.present: "true"`, `resources.limits.nvidia.com/gpu: 1`, mount `personas-pvc`.
6. **Smoke check** in pod: `nvidia-smi`, `python -c "import torch; print(torch.cuda.is_available())"`, `python -m morgan_avatar_agent.musetalk_inference --selftest`.

**Gate:** Phase 4 row.

**Cost control:** if idle > 8h (no active avatar sessions tracked via LK room metrics), scale the Deployment to 0 and mark node for deletion via `ovh_call DELETE /cloud/project/$PROJECT_ID/instance/<id>`. Re-provision on next session start.

---

## Phase 5 — E2E integration + latency tuning

**Deliverables**
- End-to-end test harness: headless Chromium (Playwright) joins via the web app, speaks a canned utterance (pre-recorded audio piped to a fake mic), records `audio_to_first_frame_ms` per utterance.
- Tuning passes: MuseTalk batch size, audio chunk alignment (Whisper hop), VRAM allocator (`PYTORCH_CUDA_ALLOC_CONF=expandable_segments:True`), pre-computed features cache.
- **Render cache**: `hash(persona_id, sha256(audio_pcm))` → pre-rendered frame sequence on PVC at `/personas/<id>/cache/<hash>.bin`. Hits skip MuseTalk entirely and stream directly to the video source.

**Gate:** Phase 5 row (<500ms audio-to-first-frame over 10 utterances; cache hit verified on repeat).

---

## Phase 6 — LemonSlice cutover + docs

**Deliverables**
- `avatar/web/app/page.tsx` and related components: remove LemonSlice branding, env, and code paths.
- Remove `livekit-plugins-lemonslice` from `avatar/agent/requirements*.txt` (delete the legacy file from Phase 3). Make `MORGAN_AVATAR_MODE=musetalk` the production default.
- `avatar/docs/`: architecture diagram (Mermaid), runbook (rotate LK keys, re-provision GPU, re-run persona preprocess, rollback to LemonSlice emergency procedure), troubleshooting matrix (GPU OOM, NATS consumer lag, Better Auth session issues).

**Gate:** Phase 6 row.

---

## Discord reporting

One message at each transition, in Coder's intake thread, format:
```
Phase <n> — <name> — <started|verified|blocked>
Gate: <one-line result>
Next: <one action>
```
No essays. No apologies. If blocked, state the exact failing command + error and what you're trying next.

---

## Do NOT stop early
- Do not ask "should I move to the next phase" — if the gate passed, move on.
- Do not request approval for routine decisions (which chart version, which PVC size, which image tag). Pick a reasonable default, record it in HANDOFF.md, continue.
- The only valid reasons to halt: (a) quota/credit blocker requiring human action, (b) irreversible/destructive action outside the approved cost ceiling, (c) all six gates passing.
