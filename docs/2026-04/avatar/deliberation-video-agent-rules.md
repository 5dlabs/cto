# Deliberation Video — Agent Rules

Canonical reference for who appears on a 5D Labs deliberation video, what they
wear, and how their portraits are produced.

## Canonical roster

The deliberation committee is **{Morgan, Optimus, Pessimus, Praxis, Rook,
Veritas}**.

| Agent      | Role on the committee                       | Species (canonical)     |
|------------|---------------------------------------------|-------------------------|
| Morgan     | Host / moderator                            | Red fox                 |
| Optimus    | Optimist — best-case framing                | (per existing portrait) |
| Pessimus   | Pessimist — worst-case framing              | (per existing portrait) |
| **Praxis** | **Pragmatist — shippability, real-world**   | **European badger**     |
| **Rook**   | **Long-game strategist — multi-step plans** | **Gray wolf**           |
| **Veritas**| **Rigor / fact-checker — data, sources**    | **Meerkat**             |

Species rationale and the literal prompt strings live in
[`committee-character-prompts.md`](./committee-character-prompts.md).

## Branding Standard (NON-NEGOTIABLE)

Every committee character portrait **must** satisfy all of the following:

1. **Anthropomorphic animal-human hybrid** in 5dlabs house style — semi-realistic
   (Beatrix Potter meets Pixar), expressive eyes, human posture.
2. **5dlabs uniform** — dark techwear blazer or field jacket, structured collar.
3. **Prominent "5D" patch** on the shoulder or chest, clearly legible — not a
   subtle logo, not abstract — readable "5D" lettering.
4. **Black tactical gloves** on both hands, visible at chest level.
5. **Portrait orientation, head-and-shoulders crop**, subject centered.
6. **Cyberpunk Neo-Kyoto background** — same world as Morgan's anchor portrait:
   neon Japanese signage (cyan / magenta / hot pink), rain-slick streets,
   distant flying vehicles, holographic UI accents. The 5D patch glows; gloves
   show circuit-pattern detailing. The committee shares Morgan's environment,
   not a clean studio.
7. **Photorealistic illustration** with intricate fur detail.
8. **Generation model:** `model_google-gemini-3-1-flash` (the model used for
   Morgan's canonical portrait `asset_Pu5sikArqYfER2M4YR6NRUyk`). Using the
   same model is what keeps the committee visually consistent — do not swap
   models without re-rendering the entire roster.

If a render comes back missing any of (3) the 5D patch or (4) the black
gloves, regenerate **once** with stronger prompt emphasis on the missing
element. Do not infinite-loop; escalate to a human reviewer if the second
attempt still misses.

## Adding a new committee member

1. Pick a species that is **thematically fitting but not on the nose** — avoid
   the obvious metaphor (e.g. owl for a fact-checker, bird for a "rook"-named
   strategist).
2. Pick a species visually distinct from existing members so the line-up reads
   clearly side by side (different silhouettes, fur colors, postures).
3. Write the full prompt in `committee-character-prompts.md` following the
   existing template (Persona / Species rationale / Full Scenario prompt /
   Suggested aspect ratio / Render record).
4. Render via `scenario-run_model` with `model_id =
   model_google-gemini-3-1-flash`. Record the seed and the resulting
   `asset_id` back into the prompts file so the portrait can be reproduced.
5. Update the roster table in this file.

## Related

- [`committee-character-prompts.md`](./committee-character-prompts.md) — literal
  prompts and per-character render records.
- `docs/2026-04/intake-flow.md` — where the deliberation video step lives in
  the intake pipeline.
