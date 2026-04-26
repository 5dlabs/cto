# Morgan Avatar — Log & Browser Validation Gate

> **Audience:** any agent or operator about to run a remote readiness test, a
> deploy, or a provider switch for the Morgan avatar (`/echo-turn`,
> EchoMimic).
>
> **Purpose:** make Datadog + browser validation a single, repeatable gate.
> Run this before every readiness test or deploy. If the gate fails, fix the
> root cause before proceeding — do not "just try the render" first.
>
> **Companion docs:**
> - [`docs/avatar/provider-switch.md`](provider-switch.md) — provider switch surface.
> - [`docs/avatar-architecture.md`](../avatar-architecture.md) — full architecture.

---

## TL;DR

```bash
# one-shot Datadog check (exits non-zero if blockers detected)
./scripts/2026-04/avatar-log-validation.sh validate

# continuous tail (Ctrl-C to stop) — use during a live readiness test
./scripts/2026-04/avatar-log-validation.sh tail

# widen / narrow the window
AVATAR_DD_FROM=now-1h ./scripts/2026-04/avatar-log-validation.sh validate
```

Current production cutline: `/echo-turn` uses OpenClaw Morgan text,
ElevenLabs TTS with cache/backoff, and async EchoMimic MP4 on OVH AI Deploy.
It does **not** require an OpenAI key, user-facing NATS, a Kubernetes/desktop
GPU, LemonSlice/TalkingHead, or `model_q8.onnx`.

The wrapper script auto-discovers the session-local
`dd-avatar-tail.sh` helper under
`~/.copilot/session-state/*/files/`. It reads Datadog credentials from the
OVH cluster Secret via 1Password (`op`) — no secrets are ever printed.

---

## 1. When to run the gate

Run **`validate`** (one-shot) immediately before:

1. Any **remote readiness test** that hits `/echo-turn` (manual or scripted).
2. Any **deploy** of `morgan-avatar-agent`, `openclaw-morgan`, or the
   EchoMimic FastAPI app.
3. Any **provider switch** (EchoMimic ↔ LemonSlice/reference) or gateway/TTS
   fallback behavior change.
4. Reopening an avatar session after the prior browser/CDP session expired.

> **Restart / redeploy caveat (3-day production bridge).** The async
> EchoMimic render path keeps job state in an in-memory `Map` inside the
> Next.js process. A redeploy or pod restart will drop all in-flight and
> recently-completed `jobId`s — pollers will get `unknown job` and the
> UI must degrade to "audio available, video unavailable" for that turn.
> Run deploys in a maintenance window with no active sessions, and keep
> the Next.js avatar deployment at exactly one replica (or sticky
> routing) until the durable job store follow-up lands. See
> [`provider-switch.md` §1.4](provider-switch.md#14-operational-constraint--single-replica-no-rolling-restart)
> for the full constraint and the Redis / PV / EchoMimic-side follow-up
> options.

Run **`tail`** (continuous) while a live readiness test or render is in
flight. Always run it in a separate terminal so the readiness operator can
see blocker hits in real time.

The gate does **not** require a live EchoMimic render — that is the whole
point. Skip the live render unless the gate is green and a render is
explicitly required. When a live production render is required, budget for
the latest observed latency: ~205s upstream EchoMimic time and ~215s total
public `/echo-turn` E2E before the final `video/mp4`.

---

## 2. Blocker checklist

The wrapper grep-scans the Datadog window for these classes of issue.
Production blockers must be **absent** for the gate to pass; if any are
present, fix upstream before any render. Legacy NATS hits are informational
for `/echo-turn` unless you are explicitly validating the LiveKit agent path.

| Label | What it means | Typical fix surface |
|---|---|---|
| **cloudflare-524** | Origin (EchoMimic / Next.js) exceeded Cloudflare's 100s timeout. | EchoMimic OVH AI Deploy saturation, queue depth, or a stuck `/animate` request. |
| **openai-auth-fallback** *(legacy script label)* | Historical scanner label. For current `/echo-turn`, treat it as an OpenClaw Morgan routing/auth issue only if `/api/echo-turn/chat` returned 401/403 or fell back to the canned reply. | Verify `MORGAN_GATEWAY_URL`, `OPENCLAW_TOKEN`, and `MORGAN_LLM_AGENT_ID=morgan`; do **not** add or rotate `OPENAI_API_KEY` for this path. |
| **tts-fallback-header** | `/api/echo-turn/tts` served the canned `voice_clone_sample.mp3` (header `X-Morgan-TTS-Fallback`) or ElevenLabs returned 401/403/429. Cache hits use `X-Morgan-TTS-Cache: hit` and are healthy. | Rotate `ELEVENLABS_API_KEY`, check ElevenLabs egress/rate limits, or wait for the route's `Retry-After` backoff to expire. |
| **echomimic-5xx** | `/animate` returned 5xx, timed out, or logged an internal error. | Check the EchoMimic OVH AI Deploy worker and queue. Do not block on Kubernetes GPU scheduling or `model_q8.onnx`; neither is required by the production cutline. |
| **nats-stale-narration** *(legacy/non-blocking for `/echo-turn`)* | NATS narration/channel errors appear in legacy LiveKit/agent logs. | Not a `/echo-turn` production blocker unless you are explicitly validating the LiveKit agent path. The public avatar/desktop path does not use user-facing NATS. |
| **browser-stuck-working** | Browser/agent logs show the avatar frozen in a working / stalled state. | Reload tab, re-run gate; if persistent, treat as `echomimic-5xx`. |

The patterns live at the bottom of `scripts/2026-04/avatar-log-validation.sh`. Add
new blockers there; keep the label kebab-cased so they are easy to grep.

---

## 3. One-shot validate — exit codes

| Exit | Meaning | Action |
|---|---|---|
| `0` | No blockers in window. | Proceed. |
| `1` | One or more blockers matched. | Fix the listed blocker(s). |
| `2` | Bad CLI usage / helper missing. | Set `AVATAR_DD_TAIL` or restore the session helper. |
| `3` | Datadog API call failed. | Re-auth `op signin`; verify `DD_*` keys via the helper. |
| `4` | Zero log lines in window. | Avatar pods may not be emitting — widen `AVATAR_DD_FROM` and confirm pods are up. |

`exit 4` is a **soft warn**: it is not a render-blocker by itself, but you
must explain why the window is empty before continuing.

---

## 4. Browser validation — `/echo-turn`

Run this **after** the Datadog gate is green. The browser side is a separate
class of failure (HMR, CSP, fetch errors) and the Datadog gate cannot see
it.

### 4.1 Setup

If the prior CDP / Playwright session expired, re-open Chrome with remote
debugging:

```bash
# fresh user data dir avoids picking up a half-dead session
"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
  --remote-debugging-port=9222 \
  --user-data-dir="$HOME/.chrome-cto-debug" \
  https://<avatar-host>/echo-turn
```

DevTools auto-open is optional; the readiness operator should at least pin
the **Console** and **Network** tabs.

### 4.2 Non-rendering checks (run these every time)

Do these **without** triggering an animate render — they are cheap and
catch most regressions.

1. **Page GET sanity.**
   ```bash
   curl -sS -o /dev/null -w 'http=%{http_code} ttfb=%{time_starttransfer}s\n' \
     https://<avatar-host>/echo-turn
   ```
   Expect `http=200` and TTFB < 2s. A 5xx here is an immediate stop.

2. **Console — expected HMR noise (ignore):**
   - `[Fast Refresh]` rebuilding / done lines.
   - `[HMR] connected` / `[webpack] hot update`.
   - `Download the React DevTools…` banner.

   **Treat as failure:**
   - `Failed to load resource: the server responded with a status of 5xx`.
   - Any `Refused to connect to … because it violates the … Content
     Security Policy` line touching `*.elevenlabs.io`, the EchoMimic origin,
     or our gateway.
   - Uncaught `TypeError` / `SyntaxError` from `app/echo-turn/*`.

3. **Network — actionable failures:**
   - `POST /api/echo-turn/chat` should be `200` with an SSE stream. `401`
     here = OpenClaw gateway auth/routing (cross-check the legacy
     `openai-auth-fallback` scanner label, but do not add an OpenAI key).
   - `POST /api/echo-turn/tts` should be `200` and the response **must
     not** carry `X-Morgan-TTS-Fallback` for a real readiness test.
     `X-Morgan-TTS-Cache: hit` is fine. Cross-check
     `tts-fallback-header` blocker.
   - `POST /api/echo-turn/avatar` is the only request expected to be
     long-running. If you are doing a **non-rendering** validation, you can
     stop before clicking the run-turn button — the prior two requests are
     enough to prove the upstreams are healthy.

4. **No-stuck-state check.** The "working…" indicator must clear within a
    second after a chat-only interaction (no avatar render). If it sticks,
    reload, re-run the Datadog gate, and look for `browser-stuck-working` /
    `echomimic-5xx`. Do not treat NATS as a user-facing `/echo-turn`
    dependency.

### 4.3 Rendering check (only when required)

Click run-turn once, then watch:
- The continuous `tail` window (separate terminal).
- DevTools Network for `/api/echo-turn/avatar` — expect the submit request to
  return `202` with a `jobId`, polling to continue while the page says
  Morgan's voice is available, and the final poll to return `200` with
  `Content-Type: video/mp4`. A `524` here = `cloudflare-524` blocker.
- Headers on the final MP4 should include `X-EchoMimic-Elapsed-S` and
  `X-EchoMimic-Job-Id`; latest good public E2E was ~215s total / ~205s
  upstream.
- DevTools Console for `<video>` decode errors.

If any blocker fires mid-render, **abort the readiness test** and re-run
the gate after the fix.

---

## 5. Add a new blocker

1. Confirm the pattern by running `./scripts/2026-04/avatar-log-validation.sh tail`
   while reproducing the failure.
2. Add a new entry to `PATTERNS=(…)` at the bottom of
   `scripts/2026-04/avatar-log-validation.sh`. Format:
   `'<kebab-label>|<egrep pattern>'`. Patterns are matched
   case-insensitively against the redacted log stream.
3. Add a row to **§ 2 Blocker checklist** above with the fix surface.
4. Re-run `./scripts/2026-04/avatar-log-validation.sh validate` and confirm the new
   label fires on the canned failure log line.

---

## 6. Safety / hygiene

- The wrapper uses narrow secret-shaped redaction (`sk-*`, JWTs,
  Bearer/Token/Api-Key values, long pure hex/base64) while preserving URL
  paths, hosts, query strings, and HTTP status codes so blocker scans still
  work. Do not paste raw Datadog output into chat / PRs / Discord — always
  run it through the wrapper.
- The wrapper never writes Datadog or 1Password credentials to disk; the
  underlying helper uses an ephemeral `mktemp` kubeconfig with `chmod 600`.
- `AVATAR_DD_QUERY` overrides exist for triage but **must not** be checked
  in. Default to the helper's built-in query.
