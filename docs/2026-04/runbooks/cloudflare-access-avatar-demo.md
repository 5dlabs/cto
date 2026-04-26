# Runbook: Cloudflare Access for `avatar.5dlabs.ai`

## Why

PR #4792 exposed `avatar.5dlabs.ai` via a `TunnelBinding` pointing at
`next dev` (port 3000 on the `openclaw-coder` Service). The tunnel bypasses
code-server's built-in auth, so without an additional layer the Next.js
**development** server is reachable by anyone on the internet.

`next dev` is **not** safe for unauthenticated public exposure:

- Serves source maps and unminified bundles (leaks internal paths and
  code structure).
- Enables HMR / websocket endpoints that are not hardened against
  untrusted input.
- Has a history of RCE-class CVEs in the dev toolchain
  (e.g. CVE-2025-29927, CVE-2024-46982) that do not apply to
  `next start` builds.

We keep `next dev` (HMR is the whole point of the demo URL) and put
Cloudflare Access in front of the hostname instead.

## What this adds

A **Cloudflare Access Self-Hosted Application** bound to
`avatar.5dlabs.ai` with:

- Identity-provider login (One-time PIN fallback + Google Workspace if
  configured) before any HTTP request reaches the tunnel.
- An email-based allow policy scoped to `@5dlabs.ai` (plus any explicit
  external reviewers added ad-hoc).
- 24h session duration (re-auth after a working day).

Cloudflare Access terminates at Cloudflare's edge, so the authenticated
request still reaches `cloudflared → openclaw-coder:3000` exactly as
before — no in-cluster changes are required.

## Why this is a runbook and not GitOps

The repo's existing Cloudflare footprint is managed by
[`adyanth/cloudflare-operator`](https://github.com/adyanth/cloudflare-operator),
which only reconciles **Tunnels** and **TunnelBindings**. It does **not**
manage Cloudflare Access applications or policies. There is currently no
Terraform / Pulumi / `cf-terraforming` wiring in this repo for the
`cloudflare_access_application` / `cloudflare_access_policy` resources.

Until that gap is closed, Access apps are configured in the Cloudflare
dashboard by hand using the steps below. Track the gap in a follow-up
issue ("Manage Cloudflare Access via IaC") so new hostnames don't
accumulate un-codified policy.

## Pre-merge action (one-time, required)

Anyone with `Cloudflare Zero Trust → Access → Edit` permissions on the
`5dlabs.ai` account can do this. Takes ~3 minutes.

1. Log in to <https://one.dash.cloudflare.com/> and select the 5DLabs
   account.
2. Go to **Access → Applications → Add an application → Self-hosted**.
3. Application configuration:
   - **Application name**: `avatar-demo`
   - **Session Duration**: `24 hours`
   - **Application domain**:
     - Subdomain: `avatar`
     - Domain: `5dlabs.ai`
     - Path: *(leave empty — apply to whole host)*
   - **Identity providers**: leave the account defaults enabled (at
     minimum: **One-time PIN**; add Google Workspace if it's already
     wired up for other apps).
   - **CORS / extra settings**: leave defaults. Do **not** enable
     "Bypass options preflight" — Next dev needs Access to authenticate
     the WebSocket upgrade as well.
4. Click **Next** to add policies. Create a single **Allow** policy:
   - **Policy name**: `allow-5dlabs-team`
   - **Action**: `Allow`
   - **Session duration**: *Same as application*
   - **Include** rule:
     - Selector: **Emails ending in**
     - Value: `@5dlabs.ai`
   - (Optional) Add a second `Include` rule of type **Emails** and add
     any explicit external reviewers (e.g. a client demo account). Keep
     this list short and audit quarterly.
5. Save the application. Cloudflare will start returning the Access
   login interstitial on `https://avatar.5dlabs.ai` within ~30 s.

## Post-merge verification

From any machine off the corporate VPN:

```bash
# Expect HTTP 302 to https://<team>.cloudflareaccess.com/... for the login page
curl -sSI https://avatar.5dlabs.ai | head -n 5

# After logging in via a browser once, confirm the Next dev server is reachable
# (cookie provides CF_Authorization; you'll see the Next.js app HTML).
```

Negative checks:

- An incognito window with no `@5dlabs.ai` identity should land on the
  CF Access login screen and **not** receive any response from the Next
  dev server.
- `curl -sSI https://avatar.5dlabs.ai/_next/static/...` without an
  Access JWT should also be intercepted (confirms path coverage).

If verification fails, re-check the Application Domain in step 3 — a
common mistake is leaving the path as `/` instead of empty, which
excludes the root.

## Rollback

Delete the `avatar-demo` application from Cloudflare Zero Trust →
Access → Applications. The tunnel itself keeps working; the hostname
just becomes publicly reachable again. Do this only if you are
simultaneously taking the hostname down (e.g. scaling `next dev` to
zero).

## Follow-ups

- Track IaC for CF Access in its own issue so future hostnames
  (`*.5dlabs.ai`) get policy-as-code instead of dashboard clicks.
- Consider switching to `next start` once the demo stops needing HMR;
  `next start` serves a minified production build and is safer to
  expose even behind Access.
