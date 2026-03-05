# Agent Avatar Image Prompts

Canonical prompts for generating agent avatars for the marketing site. **Badges are important:** every avatar must show clear **5DLabs** branding (e.g. logo patch on jacket, chest badge, or shoulder patch) so the motif is consistent and on-brand.

---

## Motif (all avatars)

- Sleek anthropomorphic animal in cyberpunk style
- Dark, high-tech environment with neon accents (cyan, pink, magenta)
- **5DLabs logo** visible on clothing or badge (required)
- Holographic UI or dev-environment elements where relevant
- Same lighting and palette across the set

---

## Glitch (Game Developer)

**Role:** Builds games and interactive experiences — indie titles, serious games, browser-based play. Unity, Godot, Unreal, WebGL.

**Compact prompt** (for character-limited fields, ~120 chars):

```
Sleek anthropomorphic fox in cyberpunk style, 5DLabs badge on jacket, holographic game editor and neon GAME LAB background.
```

**Long prompt** (when the tool allows more characters): fox in dim game dev studio; dark jacket with cyan/magenta neon seams and 5DLabs logo patch on shoulder or chest; paw on holographic level editor with PLAY/BUILD labels; neon signs GAME LAB, WEBGL; server racks; same dramatic lighting and style as Vex and Block avatars.

**Export for marketing:** Save as `glitch-avatar-512.png` and add to `apps/marketing/public/agents/`. Set `avatar: "/agents/glitch-avatar-512.png"` on Glitch in `apps/marketing/src/app/page.tsx` (Specialists squad).

---

## Business agents

Same motif: sleek anthropomorphic animal, cyberpunk style, dark techwear with **5DLabs badge**, holographic UI, neon environment. Use the long-format style below for image generation.

### Morgan (Technical Program Manager / Intake)

**Role:** Orchestrates project lifecycles — syncing GitHub with Linear, decomposing PRDs into tasks. Research for docs, web, and codebase context.

**Long prompt:**

Morgan → Sleek anthropomorphic owl in cyberpunk style, wearing a dark techwear jacket with cyan and pink neon seams and a 5DLabs badge, holding a holographic project dashboard with PRD documents, Linear ticket boards, and task dependency graphs, surrounded by floating research panels, doc snippets, and GitHub/Linear sync status indicators, neon-lit command center with warm cyan and pink glow in the background, realistic digital art with cyan and magenta metallic accents.

**Export:** `morgan-avatar-512.png` → `apps/marketing/public/agents/`. Avatar path on Morgan in Project Management squad.

---

### Atlas (Integration Master / Merge Gate)

**Role:** Manages PR merges, rebases stale branches, and ensures clean integration.

**Long prompt:**

Atlas → Sleek anthropomorphic bear in cyberpunk style, wearing a heavy dark techwear jacket with slate and zinc metallic accents and a 5DLabs badge, holding a holographic merge queue and branch graph with PR status indicators and green checkmarks, surrounded by floating branch diagrams, merge conflict resolvers, and CI pipeline status panels, neon-lit integration hub with cool slate and blue reactor glow in the background, realistic digital art with silver and zinc metallic accents.

**Export:** `atlas-avatar-512.png` → `apps/marketing/public/agents/`. Avatar path on Atlas in Operations squad.

---

### Keeper (Operations)

**Role:** Cluster maintenance, monitoring. (Roster agent; add to marketing page if/when featured.)

**Long prompt:**

Keeper → Sleek anthropomorphic raven in cyberpunk style, wearing a dark techwear jacket with teal and blue neon seams and a 5DLabs badge, holding a holographic cluster topology map with node health indicators and alert dashboards, surrounded by floating metrics panels, log streams, and status lights, neon-lit NOC with cool teal and blue glow in the background, realistic digital art with teal and silver metallic accents.

**Export:** `keeper-avatar-512.png` → `apps/marketing/public/agents/`. Add Keeper to page when ready.

---

## New business agents (home page — In Development)

Same long-format style. These four appear on the home page as coming soon: Lex (Legal), Hype (Marketing), Tally (Accounting), Chase (Sales).

### Lex (Legal Counsel)

**Role:** Contract review, compliance checks, and legal risk assessment. Trained on your jurisdiction, your agreements, your standards.

**Long prompt:**

Lex → Sleek anthropomorphic wolf in cyberpunk style, wearing a dark techwear jacket with indigo and blue neon seams and a 5DLabs badge, holding a holographic contract review interface with clause highlights, compliance checklists, and risk indicators, surrounded by floating document panes, jurisdiction tags, and approval status badges, neon-lit legal command center with cool indigo and blue glow in the background, realistic digital art with indigo and silver metallic accents.

**Export:** `lex-avatar-512.png` → `apps/marketing/public/agents/`. Set avatar on Lex when added to page.

---

### Hype (Marketing Strategist)

**Role:** Campaign strategy, copy, and analytics. From brand voice to conversion — autonomous marketing that moves as fast as your product.

**Long prompt:**

Hype → Sleek anthropomorphic peacock in cyberpunk style, wearing a dark techwear jacket with magenta and gold neon seams and a 5DLabs badge, holding a holographic campaign dashboard with brand voice metrics, conversion funnels, and copy variants, surrounded by floating analytics charts, audience segments, and engagement indicators, neon-lit strategy room with vibrant magenta and gold glow in the background, realistic digital art with magenta and gold metallic accents.

**Export:** `hype-avatar-512.png` → `apps/marketing/public/agents/`. Set avatar on Hype when added to page.

---

### Tally (Accounting Specialist)

**Role:** Bookkeeping, reconciliation, and financial reporting. Always accurate, always current, zero overhead.

**Long prompt:**

Tally → Sleek anthropomorphic beaver in cyberpunk style, wearing a dark techwear jacket with emerald and gold neon seams and a 5DLabs badge, holding a holographic ledger and reconciliation view with balance sheets, transaction streams, and report summaries, surrounded by floating spreadsheets, audit trails, and compliance badges, neon-lit finance hub with warm emerald and gold glow in the background, realistic digital art with emerald and gold metallic accents.

**Export:** `tally-avatar-512.png` → `apps/marketing/public/agents/`. Set avatar on Tally when added to page.

---

### Chase (Sales Agent)

**Role:** Outreach, pipeline management, and closing. Handles discovery, follow-ups, and deal tracking so your team stays focused on building.

**Long prompt:**

Chase → Sleek anthropomorphic cheetah in cyberpunk style, wearing a dark techwear jacket with amber and orange neon seams and a 5DLabs badge, holding a holographic pipeline board with deal stages, follow-up reminders, and close probability indicators, surrounded by floating outreach queues, meeting schedules, and win/loss metrics, neon-lit sales war room with warm amber and orange glow in the background, realistic digital art with amber and orange metallic accents.

**Export:** `chase-avatar-512.png` → `apps/marketing/public/agents/`. Set avatar on Chase when added to page.
