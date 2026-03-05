---
name: landing-marketing-app
description: How to work with the 5dlabs landing page (splash) and CTO marketing app. Use when editing the public sites, agent tiles, investor CTAs, deploy flows, or when handed off to work on landing/marketing.
---

# Landing Page & Marketing App

Guidance for working with the two public-facing Next.js apps in this repo: **splash** (5dlabs.ai landing) and **marketing** (CTO product site).

## The Two Apps

| App | Path | Purpose | Deploy target |
|-----|------|---------|----------------|
| **Splash** | `apps/splash/` | 5dlabs.ai landing: investor CTAs, magnetic filings background, header/nav | Cloudflare Pages `5dlabs-splash` |
| **Marketing** | `apps/marketing/` | CTO marketing: agent cards, squads, waitlist, pricing, tech stack | Cloudflare Pages `cto-marketing` |

Both use **Next.js 16**, **React 19**, **Tailwind 4**, **Framer Motion**, and **Radix UI**. Run from repo root or app dir:

```bash
cd apps/splash && npm ci && npm run dev   # http://localhost:3000
cd apps/marketing && npm ci && npm run dev
```

## Key Paths

### Splash (landing)

- **Layout / global**: `apps/splash/src/app/layout.tsx` (e.g. magnetic filings wrapper)
- **Pages**: `apps/splash/src/app/investors/page.tsx`, other route folders under `app/`
- **Components**: `header.tsx`, `investor-cta-buttons.tsx`, `magnetic-filings-background.tsx`
- **Styling**: `app/globals.css`, Tailwind in components

### Marketing

- **Home page (agents, waitlist, hero)**: `apps/marketing/src/app/page.tsx`
- **Other routes**: `app/pricing/page.tsx`, `app/ralph/page.tsx`
- **Components**: `agent-card.tsx` (flip card, squads), `header.tsx`, `waitlist-form.tsx`, `tech-stack.tsx`, `grid-pulse.tsx`, `shift-dimensions-wrapper.tsx`
- **UI primitives**: `components/ui/` (button, card, avatar, hover-card, input)
- **Config**: `@/config/feature-flags` if used

## Agent Data and Inventory

- **Marketing agent tiles** (name, role, tools, skills) are defined in `apps/marketing/src/app/page.tsx` in the `squads` array (`AgentSquad[]`). Each agent has `name`, `role`, `avatar`, `color`, `description`, `tools[]`, `skills[]`.
- **Source of truth for tools/skills**: `docs/agent-inventory.md` (built from cto-config, controller templates, and skill-mappings). When updating agent tiles, align with the inventory so the site matches the platform.

## Deploy

- **Splash**: Push to `main` with changes under `apps/splash/**` triggers `.github/workflows/deploy-splash.yaml` (Cloudflare Pages). Or run the "Deploy Splash Site (5dlabs.ai)" workflow manually.
- **Marketing**: Push to `main` with changes under `apps/marketing/**` triggers `.github/workflows/deploy-marketing.yaml`. Or run "Deploy Marketing Site" manually.
- Workflow runs: `npm ci` → `npm run build` in the app dir → `wrangler pages deploy out --project-name=...`.

## Conventions

- Use **Tailwind** for layout and styling; avoid one-off CSS unless necessary.
- **Motion**: Framer Motion (`framer-motion`) is available; use `whileHover` / `whileTap` for consistent interaction (e.g. investor CTAs, agent cards).
- **Branching**: Create a feature branch from latest `main`; open a PR for review. Don’t push to `main` directly.
- After changing copy or structure, run the app locally and do a quick visual check before opening a PR.

## Quick Checklist for Handoff

- [ ] Confirm which app (splash vs marketing) and which route/component.
- [ ] Run `npm ci && npm run dev` in that app; verify current behavior.
- [ ] For agent tile changes, cross-check `docs/agent-inventory.md` and `apps/marketing/src/app/page.tsx`.
- [ ] For deploy: merge to `main` (or trigger workflow) so the right path filter runs.
