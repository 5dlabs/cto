# Coder brief — Avatar provider failover (OVH + DO only)

**You are Coder (OpenClaw agent).** This brief defines *your* slice of the avatar
failover work. The full architecture lives in
[`docs/plans/avatar-provider-failover.md`](../../../docs/plans/avatar-provider-failover.md)
on `main` — **read it top to bottom before starting anything**. Everything
below narrows that plan to *your* deliverables.

---

## Your scope

You own three deliverables from the plan, and **only those three**:

| Deliverable | What you build | Repo path |
|---|---|---|
| **D2** | `avatar-worker` container image. **Pick the highest-quality open-source model that runs at ≥15 fps on a single L40S (48 GB) and delivers lip sync + body/hand gestures + persistent idle animation.** See "Model selection" below. | `infra/images/avatar-worker/` |
| **D3** | OVH AI Deploy provisioning scripts + docs | `infra/avatar/ovh/` |
| **D4** | DigitalOcean GPU Droplet provisioning scripts + docs | `infra/avatar/digitalocean/` |

Each deliverable ships as **one PR against `main`**. Stop and wait for review
between PRs. No stacking.

## What Jonathon (human) and the intake coordinator own — DO NOT TOUCH

- **D1** — `infra/images/voice-bridge/app/avatar/base.py` (`AvatarProvider` /
  `AvatarSession` Protocol). This is the **locked contract** you code against.
  If you think it needs changing, stop and ask the user — don't edit it.
- **D5** — voice-bridge WS integration (new `0x03` video-frame opcode,
  failover engine, Helm `avatar.enabled` flag, k8s manifests).
- **D6** — operational runbook.
- **Lemon Slice provider** (`LemonSliceProvider` class, livekit-plugins-lemonslice
  integration). Jonathon is handling this personally.

Your OVH and DO providers must conform to the D1 `AvatarProvider` Protocol
verbatim. The Protocol signature is in the plan doc §1 — copy it exactly.

## Hard guardrails (repeating from plan — read these again)

- **NEVER** `kubectl delete` against `node`, `pv`, `pvc`, or `namespace`.
- **NEVER** force-push any protected branch (`main`, `feat/voice-bridge-hermes`,
  anything under `release/*`).
- **NEVER** modify Cloudflare DNS records without explicit user "yes" in the
  same turn. Dry-run + print the intended change first.
- **NEVER** invoke `ovhai app run`, `doctl compute droplet create`, or any
  other live GPU provisioning command **from CI or from your own shell**.
  These are destructive (they burn real money). Your scripts must:
  1. Default to **dry-run**.
  2. Print the **exact command** and **$/hr estimate** to stdout.
  3. Require an explicit env flag (`CONFIRM_PROVISION=yes`) to actually run.
  4. On teardown, require `CONFIRM_TEARDOWN=yes` + the specific resource ID
     (`CONFIRM_DROPLET_ID=<id>` for DO, `CONFIRM_OVH_APP_ID=<id>` for OVH).
- **NEVER** commit API keys, tokens, or GPU credentials. Use 1Password refs
  (`op://...`) only. If you find a credential already committed, stop and
  report it — do not attempt to rotate it yourself.
- **Cost ceiling**: any hourly rate over **$5/hr** in your dry-run output
  must be accompanied by a big red warning banner in the script output and
  flagged back to the user before enabling `CONFIRM_PROVISION=yes`.

## Stop-and-review boundaries

After each of D2, D3, D4:

1. Open the PR.
2. Post a short summary comment: what works, what's next, any flags / known
   issues / decisions you want the user to make.
3. **Wait for the user to reply "go" before starting the next deliverable.**
   Do not speculatively start D3 while D2 is in review.

## Model selection (D2) — do this BEFORE writing worker code

**Target**: visual quality as close to LemonSlice hosted as possible given our hardware budget.

**Hard requirements** (all three):
1. **Lip sync** — accurate phoneme-level mouth shape from driving audio.
2. **Body / hand gestures** — not just head nods. Idle arm motion, gesture on emphasis.
3. **Persistent animation / natural idle** — avatar stays "alive" between utterances (blinks, micro-motion), not a frozen image.

**Hardware budget** (what we actually have, both OVH and DO):
- **L40S 48 GB** — primary target. Must run at ≥15 fps on a single card.
- H100 80 GB available on DO as stretch if a clearly-better model only fits there.
- Do not target V100S / 24 GB consumer cards.

**Your deliverable**: a table in the D2 PR description comparing the current-generation candidates. Fill real numbers from your own smoke test, not vendor claims. Include at minimum:

| Model | License | Gestures? | fps on L40S 48GB @ 512² | Time-to-first-frame | Quality notes |
|---|---|---|---|---|---|
| HunyuanVideo-Avatar | Custom (Tencent) | yes (audio-driven) | measure | measure | |
| Hallo3 | Academic / Apache-ish | yes | measure | measure | |
| Sonic (Tencent) | Apache-2 | limited (head-dom) | measure | measure | |
| OmniHuman-1 (if weights released) | check | yes | measure | measure | |
| EchoMimic v2 | Apache-2 | hands yes | measure | measure | |
| MuseTalk v1.5 | — | **no** | — | — | **fails gesture requirement — floor only, do not pick** |
| LivePortrait | MIT | partial (upper body drive) | measure | measure | |

**Rules for the matrix:**
- Add any candidate you find during research — list is not exhaustive.
- Exclude anything that fails the three hard requirements.
- Prefer Apache-2 / MIT / permissive. Flag custom licenses (Tencent Hunyuan etc.)
  in the PR and wait for user approval before picking one that needs a license ack.
- Prefer models that can run audio-driven end-to-end (we only have audio from
  voice-bridge; no pose/video driver).
- If two candidates tie on quality, pick the one with smaller weights /
  faster time-to-first-frame.
- Anthropomorphic-animal compatibility (see § below) is a tiebreaker, not
  a hard requirement — our committee avatars are animals, but we can swap
  to human-styled portraits if no audio-driven model handles non-humans.

**Stop and ask the user before picking** if:
- No candidate hits ≥15 fps on L40S (we may need to drop resolution or target H100).
- The best candidate is a custom / non-commercial license.
- Every audio-driven gesture model fails on the anthropomorphic committee
  portraits — we'll decide human-swap vs. hosted-only at that point.

---

## Exit criteria — summary (full details in the plan)

- **D2** passes: written model-selection matrix in the PR description (candidates + fps + VRAM + quality notes + chosen winner); image builds in CI; dry-run script renders ≥3s of 512² video with lipsync + at least one visible gesture from a stubbed audio+pose input; image <= 15 GB compressed (weights pulled at runtime, not baked); README documents env vars and the evaluation result.
- **D3** passes: `make -C infra/avatar/ovh plan` prints a dry-run of the
  `ovhai app run` command with `l40s-1-gpu` default and `$/hr`; `make destroy`
  refuses without `CONFIRM_OVH_APP_ID=<id>`; `README.md` documents the full
  one-shot provisioning flow; no live provisioning attempted.
- **D4** passes: `make -C infra/avatar/digitalocean plan` prints the `doctl`
  command with `gpu-l40sx1-48gb` in `nyc2` (with a fallback note if L40S is
  TOR1-only — run `doctl compute size list` in a comment on the PR so we can
  confirm region availability); teardown script requires `CONFIRM_DROPLET_ID`;
  no live provisioning attempted.

## Branching

- Base all three PRs off `main`. The voice-bridge Hermes work (PR #4769) and
  the plan doc (PR #4770) are already merged.
- Branch naming: `feat/avatar-worker-image` (D2),
  `feat/avatar-ovh-provision` (D3), `feat/avatar-do-provision` (D4).

## Known unknowns — flag these to the user when you hit them

1. **L40S availability in NYC2.** Shubham at DO said NYC2 *and* TOR1;
   earlier internal docs said TOR1-only. Your D4 PR must run
   `doctl compute size list --format Slug,Regions | grep l40s` and
   comment the output. If TOR1-only, flag and wait for user decision before
   hardcoding region.
2. **Model weight hosting.** Whichever model you pick, weights likely ≥10 GB
   and may be ≥50 GB (Hunyuan-class). Do **not** bake weights into the
   container image. Document the plan to pull from an object store at
   runtime (OVH Object Storage bucket or DO Spaces — ask the user which),
   but don't actually implement the pull path until the user approves a
   hosting target.
3. **Anthropomorphic-face compatibility.** Our committee portraits
   (Morgan = dog, Vanguard/Sentinel = stylized animals) have broken every
   human-face-landmark-based lipsync model we've tried (MuseTalk, SadTalker
   both confirmed dead via `face_alignment` detector failure). During your
   D2 smoke test, run the chosen model against `avatar/morgan-512.png`
   specifically. If the detector bombs, flag immediately — we will either
   swap to human-styled portraits or fall back to hosted (LemonSlice) for
   animal avatars. Do not spend cycles trying to fix landmark detection.
3. **CORS/auth for the worker endpoint.** Worker must not accept public
   internet traffic. Document your assumption (VPC-only, or fronted by
   voice-bridge with a shared secret header) in the D2 README and flag for
   user confirmation.

## Context links

Read these before starting D2:

- **Plan (single source of truth):**
  `docs/plans/avatar-provider-failover.md` on `main`
- **Voice-bridge (already on main):** `infra/images/voice-bridge/app/`
- **Session findings (helpful but optional):**
  - `session:files/echomimicv3-dd-findings.md` — why EchoMimicV3 is *excluded*
  - `session:files/ovh-ai-deploy-findings.md` — `ovhai app run` notes
  - `session:files/do-avatar-demo-plan.md` — DO L40S pricing / region notes

## First action

Read the plan, then reply with:

1. A one-paragraph recap of your scope in your own words.
2. The branch name + commit plan for D2.
3. Any clarifying questions for the user before you `git checkout -b`.

Do **not** start writing code until step 3 is answered.
