# Deliberation Committee — Character Prompts

Companion to [`deliberation-video-agent-rules.md`](./deliberation-video-agent-rules.md).
These prompts produce the canonical portraits for the three new committee members
that join Morgan, Optimus, and Pessimus on the deliberation stage.

## Branding standard (must be present in every prompt)

- 5dlabs aesthetic: **anthropomorphic animal-human hybrid** (Beatrix Potter meets
  Pixar — semi-realistic fur, expressive eyes, human posture).
- **5dlabs uniform** — dark techwear blazer / field jacket, structured collar.
- **Prominent "5D" patch** on the shoulder or chest (clearly legible).
- **Black tactical gloves** on both hands (visible at chest level).
- Portrait orientation, **head-and-shoulders crop**, subject centered.
- **Clean studio background** — soft neutral gradient, gentle rim light.
- Photorealistic illustration, soft cinematic studio lighting, intricate fur
  detail.
- Reference model: `model_google-gemini-3-1-flash` (same as Morgan portrait
  `asset_Pu5sikArqYfER2M4YR6NRUyk`) for visual consistency.

---

## Praxis — the pragmatist

**Persona.** Cares about shippability, real-world constraints, getting it built.
Speaks in deadlines, dependencies, and "what's the minimum viable cut?". The
voice that pulls deliberations back from theoretical purity into something the
team can actually deploy on Tuesday.

**Species rationale.** **European badger.** Badgers are the archetype of
"grounded builders" — sturdy, low to the ground, relentless diggers, famously
practical and unfussy. Visually distinctive from Morgan's red fox (different
silhouette, monochrome face stripes vs. fox's warm orange) so the two read
clearly side-by-side on a deliberation stage.

**Full Scenario prompt.**
```
Anthropomorphic animal-human hybrid character portrait, semi-realistic
Beatrix-Potter-meets-Pixar style, exact same studio treatment as the 5D Labs
Morgan red-fox character: head and shoulders of a European badger with the
classic black-and-white facial stripes, broad muzzle, and small alert dark
eyes, expression calm and decisive — the pragmatist. Wearing a fitted dark
charcoal 5dlabs techwear field jacket with structured collar and a clearly
legible embroidered "5D" patch on the left shoulder. Both hands visible at
chest level wearing matte black tactical gloves, one hand resting on a small
spiral-bound builder's notebook. Clean studio background, soft neutral
gray gradient, gentle rim light from upper left, soft cinematic key light,
photorealistic illustration with intricate fur detail. Centered composition,
portrait orientation, head-and-shoulders crop. Character is "Praxis", the
shipping-focused pragmatist on the 5D Labs deliberation committee.
```

**Suggested aspect ratio.** `1:1` (1024x1024) — matches Morgan canonical.
**Suggested seed.** Random on first render; recorded below.

**Render record.**
- Model: `model_google-gemini-3-1-flash`
- Seed used: 1842739215
- Asset ID: asset_Hct17255cUBRFxJmQjyTkawH

---

## Rook — the long-game strategist

**Persona.** Cares about positioning, multi-step plans, what wins in 6 months.
The one who asks "what does this look like after three more moves?" while
everyone else is debating the next commit. Patient, unhurried, sees the board.

**Species rationale.** **Gray wolf.** The user explicitly flagged "Rook the
bird is too obvious — surprise us". A wolf gives us the long-game energy
through a different metaphor: pack strategist, knows the territory, hunts on a
horizon measured in days not seconds. Wolves are also visually striking in a
head-and-shoulders crop (strong silhouette, intense yellow-amber eyes) and
read as immediately distinct from Morgan's fox despite being the same family.

**Full Scenario prompt.**
```
Anthropomorphic animal-human hybrid character portrait, semi-realistic
Beatrix-Potter-meets-Pixar style, exact same studio treatment as the 5D Labs
Morgan red-fox character: head and shoulders of a gray wolf with thick
salt-and-pepper fur, sharp pale yellow-amber eyes, alert ears forward,
expression measured and contemplative — the long-game strategist. Wearing a
sharp dark slate 5dlabs techwear blazer over a high-collar undershirt with a
clearly legible embroidered "5D" patch on the right chest. Both hands visible
at chest level wearing matte black tactical gloves, fingers interlaced in a
thoughtful steepled pose. Clean studio background, soft neutral gray gradient,
gentle rim light from upper right, soft cinematic key light, photorealistic
illustration with intricate fur detail. Centered composition, portrait
orientation, head-and-shoulders crop. Character is "Rook", the long-horizon
strategist on the 5D Labs deliberation committee.
```

**Suggested aspect ratio.** `1:1` (1024x1024).
**Suggested seed.** Random on first render; recorded below.

**Render record.**
- Model: `model_google-gemini-3-1-flash`
- Seed used: 297410683
- Asset ID: asset_sKyZqwQUdLrC3Arg6MxLaTCb

---

## Veritas — the rigor / fact-checker

**Persona.** Cares about data, sources, ground-truth claims. The one who asks
"what's the citation?" and "did anyone actually measure this?". Sceptical of
narrative without evidence; the deliberation's epistemic immune system.

**Species rationale.** **Meerkat.** The user explicitly flagged "Veritas as an
owl would be on the nose". Meerkats are the perfect lateral pick: famously
**vigilant sentinels** — they stand upright on lookout, scanning the
environment, sounding alarms when something is off. That's exactly Veritas's
role on the committee. Visually they're distinctive (slim build, dark eye
mask, alert posture) and very different from a fox/badger/wolf, so the
portrait line-up has clear silhouette variety.

**Full Scenario prompt.**
```
Anthropomorphic animal-human hybrid character portrait, semi-realistic
Beatrix-Potter-meets-Pixar style, exact same studio treatment as the 5D Labs
Morgan red-fox character: head and shoulders of a meerkat with sandy-tan
fur, distinctive dark eye mask, large alert dark eyes, ears small and high,
expression sharp, scrutinising, evidence-driven — the rigor and fact-checker.
Wearing a fitted dark navy 5dlabs techwear field jacket with structured
collar and a clearly legible embroidered "5D" patch on the left shoulder.
Both hands visible at chest level wearing matte black tactical gloves, one
hand holding a small folded data printout / source document. Clean studio
background, soft neutral gray gradient, gentle rim light from upper left,
soft cinematic key light, photorealistic illustration with intricate fur
detail. Centered composition, portrait orientation, head-and-shoulders crop.
Character is "Veritas", the rigor and fact-check voice on the 5D Labs
deliberation committee.
```

**Suggested aspect ratio.** `1:1` (1024x1024).
**Suggested seed.** Random on first render; recorded below.

**Render record.**
- Model: `model_google-gemini-3-1-flash`
- Seed used: 518439207
- Asset ID: asset_MwHuyNs2GbedXPMRfJZjzftg

---

## Re-render checklist

If a portrait comes back missing the **black gloves** or the **5D patch**,
regenerate that single character with stronger prompt emphasis on the missing
element (e.g. "matte black tactical gloves clearly visible on both hands" or
"large clearly embroidered '5D' patch on shoulder, sharp readable lettering").
Do not infinite-loop — one corrective regeneration per character, then
escalate to the human reviewer.
