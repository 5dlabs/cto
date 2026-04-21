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
| **D2** | `avatar-worker` container image (MuseTalk v1.5 MVP; HunyuanVideo-Avatar stretch, gated behind `AVATAR_MODEL=hunyuan`) | `infra/images/avatar-worker/` |
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

## Exit criteria — summary (full details in the plan)

- **D2** passes: image builds in CI; MuseTalk dry-run script outputs frames
  for a stubbed audio sample locally; `AVATAR_MODEL=hunyuan` path is stubbed
  with a clear `NotImplementedError("stretch — needs H100")`; image <= 12 GB
  compressed; README documents env vars.
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
2. **HunyuanVideo weight hosting** (~50 GB). For D2 stretch path, do **not**
   bake the weights into the image. Document the plan to pull from an object
   store at runtime, but don't actually implement the pull path until the
   user approves a hosting target.
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
