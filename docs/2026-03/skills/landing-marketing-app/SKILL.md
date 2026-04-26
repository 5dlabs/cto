---
name: landing-marketing-app
description: How to work with the 5dlabs website app. Use when editing public pages, CTO routes, agent tiles, investor CTAs, or deploy flows.
---

# Website App

Guidance for the unified public-facing Next.js app in this repo: **website** (`apps/website`).

## App Overview

| App | Path | Purpose | Deploy target |
|-----|------|---------|----------------|
| **Website** | `apps/website/` | 5dlabs.ai site with landing pages at `/` and CTO marketing under `/cto/*` | Cloudflare Pages `5dlabs-splash` |

The website uses **Next.js 16**, **React 19**, **Tailwind 4**, **Framer Motion**, and **Radix UI**.

```bash
cd apps/website && npm ci && npm run dev   # http://localhost:3000
```

## Key Paths

- **Layout / global**: `apps/website/src/app/layout.tsx`
- **Landing routes**: `apps/website/src/app/*`
- **CTO routes**: `apps/website/src/app/cto/*`
- **CTO components**: `apps/website/src/components/cto/*`
- **Cloudflare functions**: `apps/website/functions/api/*`
- **UI primitives**: `apps/website/src/components/ui/*`

## Agent Data and Inventory

- **CTO agent tiles** are defined in `apps/website/src/app/cto/page.tsx` in the `squads` array (`AgentSquad[]`).
- **Source of truth for tools/skills**: `docs/agent-inventory.md` (built from cto-config, controller templates, and skill-mappings). When updating agent tiles, align with the inventory so the site matches the platform.

## Deploy

- Push to `main` with changes under `apps/website/**` to trigger `.github/workflows/deploy-splash.yaml` (Cloudflare Pages).
- Workflow runs: `npm ci` → `npm run build` in the app dir → `wrangler pages deploy out --project-name=...`.

## Conventions

- Use **Tailwind** for layout and styling; avoid one-off CSS unless necessary.
- **Motion**: Framer Motion (`framer-motion`) is available; use `whileHover` / `whileTap` for consistent interaction (e.g. investor CTAs, agent cards).
- **Branching**: Create a feature branch from latest `main`; open a PR for review. Don’t push to `main` directly.
- After changing copy or structure, run the app locally and do a quick visual check before opening a PR.

## Quick Checklist for Handoff

- [ ] Confirm route scope (`/` landing vs `/cto/*` marketing route).
- [ ] Run `npm ci && npm run dev` in `apps/website`; verify behavior.
- [ ] For agent tile changes, cross-check `docs/agent-inventory.md` and `apps/website/src/app/cto/page.tsx`.
- [ ] For deploy: merge to `main` (or trigger workflow) so the right path filter runs.
