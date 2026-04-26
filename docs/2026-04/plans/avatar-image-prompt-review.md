# Avatar Image Prompt Review

This file is the review surface for prompt exploration while Phase 1 avatar runtime work is in progress.

## Goal

Create a Morgan visual direction that works for:

- deterministic local fallback avatar in the Phase 1 web client
- future rigged 3D character reference
- consistent 5DLabs brand language
- humanoid/anthro extension later if we decide Morgan should evolve beyond the current portrait

## Current in-repo source assets

- `avatar/morgan.jpg`
- `avatar/web/public/morgan.jpg`
- `docs/agent-avatar-prompts.md`
- existing 512px marketing avatars under `crates/cto-lite/ui/public/agents/`

## Prompt directions to review

### Direction A — realistic executive / operator

**Use when we want Morgan to feel like a premium human-facing operator.**

```text
Morgan, a sharp technical program manager in a cinematic cyberpunk command center, medium close-up portrait, intelligent calm expression, dark structured techwear jacket with subtle cyan and magenta edge lighting, visible 5DLabs badge, holographic task graphs and planning boards softly floating in the background, realistic digital art, premium startup brand aesthetic, teal and violet glow, highly readable face, clean silhouette, not cartoonish, not exaggerated, no text overlays, no extra fingers, no duplicate features
```

### Direction B — human but slightly stylized for 3D transfer

**Use when we want an easier bridge into a rigged 3D character later.**

```text
Morgan, stylized semi-realistic technical program manager character concept for a realtime 3D avatar, front-facing portrait, clean facial planes, controlled lighting, dark techwear jacket with cyan and pink seams, subtle 5DLabs chest badge, holographic project dashboard behind him, readable expression shapes for speech animation, design suitable for rigging and blendshapes, polished concept art, not anime, not cartoon, not photobash, no text, no watermark
```

### Direction C — anthro business-agent family alignment

**Use if we decide Morgan should align with the broader anthropomorphic business-agent visual system.**

```text
Morgan as a sleek anthropomorphic owl technical program manager in cyberpunk style, dark techwear jacket with cyan and pink neon seams, visible 5DLabs badge, holding a holographic task and dependency dashboard with PRD notes and ticket flow, neon-lit command center, realistic digital art, premium and intelligent, consistent with the 5DLabs business-agent avatar family, no text, no watermark
```

## Review criteria

Score each sample on:

1. **Face readability** — clear expression, readable at thumbnail size
2. **Brand fit** — feels like 5DLabs, not generic sci-fi slop
3. **Rig readiness** — easy to translate into a rigged 3D character later
4. **Speech readiness** — mouth/jaw/eye area looks usable as facial reference
5. **Production fit** — can serve as fallback 2D avatar now while 3D assets mature

## Current recommendation

**Start with Direction B.**

Why:

- best bridge between current Phase 1 fallback UI and future Option A rigged 3D implementation
- preserves human readability
- avoids overcommitting to either photoreal or anthro branding too early
- gives us a cleaner target for future GLB/VRM production

## Next artifact step

When image generation is available in a clean loop, create:

- `docs/artifacts/avatar-prompts/morgan-direction-a.png`
- `docs/artifacts/avatar-prompts/morgan-direction-b.png`
- `docs/artifacts/avatar-prompts/morgan-direction-c.png`

and add a short comparison note here.
