# Avatar Runtime Validator Contract

> **Status:** Draft
> **Audience:** Pipeline operators, avatar agents (Rex, Blaze, Bolt), CI/CD
> **Companion docs:**
> - [`avatar-asset-spec.md`](avatar-asset-spec.md) — authoritative asset format & rig specification
> - [`../avatar/validation.md`](../avatar/validation.md) — Datadog & browser validation gate runbook
> - [`../avatar/model-dag-plan.md`](../avatar/model-dag-plan.md) — generation pipeline DAG

---

## 1. Purpose

This document defines the **minimum artifact bundle** that every avatar
candidate run must produce, and the **tiered pass/fail gates** that determine
whether a candidate advances, gets repaired, or is rejected. It is a
contract — any pipeline stage, CI job, or agent that evaluates an avatar
candidate MUST apply these gates in order.

The contract is designed so that:

- **Cheap gates run first** (file-level checks before GPU renders).
- **Every gate has a machine-readable exit code** (0 = pass, non-zero = fail
  with category).
- **Promotion requires all gates green.** A candidate MAY be kept for repair
  if only specific gates fail (see § 3 gate-failure routing).

---

## 2. Artifact Bundle Manifest

Every candidate run produces a **bundle directory** named
`<candidate_id>/` containing the artifacts below. A candidate that is
missing any **required** artifact cannot enter gate evaluation — it is
rejected at intake.

| # | File | Format | Required | Size limit | Description |
|---|------|--------|----------|------------|-------------|
| 1 | `avatar.glb` | glTF 2.0 binary (Draco+KTX2 OK) | **yes** | ≤ 25 MB | Single-file avatar asset. No external buffer/texture references. |
| 2 | `avatar.json` | JSON (schema v1) | **yes** | ≤ 256 KB | Manifest: name, rig family, skeleton summary, texture map, animation clips, viseme mapping ref, asset hash, LOD map. Schema defined in [`avatar-asset-spec.md` § avatar.json](avatar-asset-spec.md). |
| 3 | `validation.json` | JSON | **yes** | ≤ 1 MB | Machine-readable gate results. One object per gate tier (G0–G5), each with `pass: bool`, `details: {}`, `timestamp`, `duration_ms`. Populated incrementally as gates run. |
| 4 | `morph-inventory.json` | JSON | **yes** | ≤ 512 KB | Extracted morph-target list per mesh: `{ meshes: [{ name, targets: [string] }] }`. Source of truth for G2. |
| 5 | `skeleton-inventory.json` | JSON | **yes** | ≤ 512 KB | Extracted bone hierarchy: `{ bone_count, hierarchy: { name, children: [] }, joint_names: [string] }`. Source of truth for G1 bone checks. |
| 6 | `animation-inventory.json` | JSON | **yes** | ≤ 512 KB | Animation clip list: `{ clips: [{ name, duration_s, loop }] }`. Source of truth for G3 clip completeness. |
| 7 | `viseme-mapping.json` | JSON | **yes** | ≤ 128 KB | Maps each of 15 Oculus viseme IDs to one or more morph target names, with optional weight overrides. Also lists any species-specific morph alias table. |
| 8 | `renders/front.png` | PNG | **yes** | ≤ 8 MB | Front orthographic still (Blender or Three.js). |
| 9 | `renders/side.png` | PNG | **yes** | ≤ 8 MB | Side orthographic still. |
| 10 | `renders/three_quarter.png` | PNG | **yes** | ≤ 8 MB | Three-quarter perspective still. |
| 11 | `renders/turntable.mp4` | MP4 (H.264) | optional | ≤ 50 MB | 360° turntable video. Required only for final promotion or explicit video review, not for cheap early Scenario candidate triage. |
| 12 | `renders/summary.txt` | Text | **yes** | ≤ 64 KB | Render metadata: input path, import mode, bounding box, mesh count, material count. |
| 13 | `browser/screenshot.png` | PNG | optional | ≤ 8 MB | Screenshot of the avatar loaded in the browser runtime (Three.js / WebGL). |
| 14 | `browser/perf.json` | JSON | optional | ≤ 256 KB | Browser performance metrics snapshot (connection latency, frame drops, memory, etc.). Required for G4. |
| 15 | `provenance.json` | JSON | **yes** | ≤ 256 KB | Lineage record: source image(s), generation model, model version, parameters, timestamps, parent candidate ID (if repair), operator/agent ID. |
| 16 | `logs/pipeline.log` | Text | **yes** | ≤ 10 MB | Full pipeline stdout/stderr for the candidate run. |
| 17 | `logs/gate-summary.log` | Text | **yes** | ≤ 1 MB | Human-readable gate result summary (one line per gate, pass/fail + reason). |
| 18 | `index.html` | HTML | optional | ≤ 1 MB | Self-contained browser preview page that loads `avatar.glb` via Three.js or model-viewer. Used for manual QA and browser screenshot capture. |

### 2.1 Bundle integrity

The bundle root MUST also contain a `checksums.sha256` file listing the
SHA-256 hash of every file in the bundle (one `<hash>  <path>` per line,
GNU coreutils format). Gate G0 verifies this file against the actual
contents.

---

## 3. Gate Tiers

Gates are evaluated **in order** G0 → G5. A gate failure at tier N means
tiers N+1 … 5 are **not run** (fail-fast). Each gate writes its result
into `validation.json` before exiting.

### Exit code convention

Every gate script MUST use these exit codes:

| Code | Meaning |
|------|---------|
| 0 | Pass |
| 1 | Fail — fixable (candidate may be routed to repair) |
| 2 | Fail — configuration / usage error |
| 3 | Fail — infrastructure error (network, missing tool) |

### Gate-failure routing

| Failed gate | Routing |
|-------------|---------|
| G0 | **Reject.** Candidate is corrupt or incomplete — no repair path. |
| G1 | Route to **retopo / rig-repair** stage if mesh is salvageable; otherwise reject. |
| G2 | Route to **blendshape-transfer** stage (FLAME/ARKit/Oculus mapping). |
| G3 | Route to **animation-authoring** or **lip-sync-repair** stage. |
| G4 | Route to **optimization** stage (LOD, draw-call reduction, texture compression). |
| G5 | Hold for **operator review** — candidate is structurally sound but operationally unproven. |

---

### G0 — Structural Integrity

**Purpose:** Verify the candidate bundle is complete, parseable, and
internally consistent before any content inspection.

| Check | Criterion | Source |
|-------|-----------|--------|
| G0.1 — Bundle completeness | All **required** files from § 2 are present and non-empty. | `ls` / manifest check |
| G0.2 — Checksum integrity | Every file in `checksums.sha256` matches its on-disk SHA-256. | `sha256sum -c checksums.sha256` |
| G0.3 — GLB validity | `avatar.glb` passes `gltf-validator` with zero errors (warnings OK). | [`KhronosGroup/glTF-Validator`](https://github.com/KhronosGroup/glTF-Validator) |
| G0.4 — GLB file size | `avatar.glb` ≤ 25 MB. | `stat` / `wc -c` |
| G0.5 — Manifest parse | `avatar.json` is valid JSON and matches the manifest schema (version, name, family fields present). | JSON schema validation |
| G0.6 — Manifest hash match | `avatar.json → asset_hash` matches SHA-256 of `avatar.glb`. | `sha256sum` + `jq` |
| G0.7 — Inventory parse | `morph-inventory.json`, `skeleton-inventory.json`, `animation-inventory.json`, `viseme-mapping.json` all parse as valid JSON. | `jq .` |
| G0.8 — Provenance parse | `provenance.json` is valid JSON with required fields: `source_images`, `model`, `model_version`, `timestamp`. | JSON schema validation |

**Pass:** All checks green.
**Fail:** Any check fails → exit 1, candidate rejected.

---

### G1 — Mesh & Rig Quality

**Purpose:** Verify skeleton, mesh topology, and material limits meet
runtime constraints defined in [`avatar-asset-spec.md`](avatar-asset-spec.md).

| Check | Criterion | Threshold | Source |
|-------|-----------|-----------|--------|
| G1.1 — Bone count | Total bones in skeleton ≤ 80 (≤ 60 for mobile target). | `skeleton-inventory.json → bone_count` |
| G1.2 — Skeleton hierarchy | Root bone is `Hips`; required chain `Hips → Spine → Spine1 → Spine2 → Neck → Head` exists. Eye bones (`LeftEye`, `RightEye`) and jaw bone (`Jaw`) present. | `skeleton-inventory.json → hierarchy` |
| G1.3 — Species bones | If `avatar.json → family` is `biped-anthro`, `quadruped-mammal`, or `quadruped-avian`: species-specific `aux_*` bones present per asset spec. | `skeleton-inventory.json` cross-ref `avatar.json → family` |
| G1.4 — Material count | ≤ 12 materials total. | GLB inspection or `renders/summary.txt → material count` |
| G1.5 — Texture limits | Albedo ≤ 2048×2048, normal ≤ 1024×1024, ORM ≤ 1024×1024. ≤ 5 textures per material region. | GLB texture metadata inspection |
| G1.6 — Mesh count | Total draw-call contributing meshes reported in `renders/summary.txt`. No hard reject threshold, but flag if > 20. | `renders/summary.txt` |

**Pass:** G1.1–G1.5 all green; G1.6 is advisory.
**Fail:** Any of G1.1–G1.5 fails → exit 1.

---

### G2 — Morph & Viseme Presence

**Purpose:** Verify that the GLB carries the required morph targets for
facial expression and lip-sync, as defined by
[`validate-avatar-glb.py`](../../../scripts/2026-04/validate-avatar-glb.py).

| Check | Criterion | Threshold | Source |
|-------|-----------|-----------|--------|
| G2.1 — ARKit blendshapes | All 52 Apple ARKit blendshape names present on at least one mesh. | `morph-inventory.json` cross-ref canonical list |
| G2.2 — Oculus visemes | All 15 Oculus viseme names present: `viseme_sil`, `viseme_aa`, `viseme_E`, `viseme_I`, `viseme_O`, `viseme_U`, `viseme_PP`, `viseme_FF`, `viseme_TH`, `viseme_DD`, `viseme_kk`, `viseme_CH`, `viseme_SS`, `viseme_nn`, `viseme_RR`. | `morph-inventory.json` |
| G2.3 — Species add-ons | If `avatar.json → family ∈ {biped-anthro, quadruped-mammal}`: 10 canine-species morph targets present (`snout_open`, `snout_wrinkle`, `jowl_flap_left`, `jowl_flap_right`, `tongue_loll`, `ear_left_rotate`, `ear_right_rotate`, `ear_left_droop`, `ear_right_droop`, `nose_twitch`). | `morph-inventory.json` cross-ref `avatar.json → family` |
| G2.4 — Weight range | All morph target weights clamp to `[0.0, 1.0]`. | GLB accessor inspection |
| G2.5 — Active target limit | ≤ 60 morph targets active per draw call (WebGL 2 constraint). | `morph-inventory.json` per-mesh target count |
| G2.6 — Viseme mapping completeness | `viseme-mapping.json` contains an entry for every one of the 15 Oculus viseme IDs, each mapping to ≥ 1 morph target name that exists in `morph-inventory.json`. | Cross-reference |
| G2.7 — Total morph count | Summary count matches expected total: 52 ARKit + 15 Oculus + species add-ons (if applicable). Minimum 67 for `biped-human`, 77 for `biped-anthro` / `quadruped-mammal`. | `morph-inventory.json` |

**Pass:** All checks green.
**Fail:** Any check fails → exit 1, route to blendshape-transfer repair.

---

### G3 — Expression & Behavior

**Purpose:** Verify animation clips and lip-sync behavior meet runtime
expectations.

| Check | Criterion | Threshold | Source |
|-------|-----------|-----------|--------|
| G3.1 — Required clips | Animation clips `idle`, `listen`, `speak`, `think`, `acknowledge` all present. | `animation-inventory.json → clips[].name` |
| G3.2 — Clip duration | Each required clip ≤ 3 seconds duration. | `animation-inventory.json → clips[].duration_s` |
| G3.3 — Clip loopability | Each required clip is marked as loopable. | `animation-inventory.json → clips[].loop == true` |
| G3.4 — Total animation size | Sum of all animation data ≤ 500 KB. | GLB animation accessor size or bundle heuristic |
| G3.5 — Gesture cue mapping | `avatar.json` includes a `gesture_cues` map covering all five protocol gestures: `idle`, `listen`, `speak`, `think`, `acknowledge`. Each maps to a clip name present in the animation inventory. | `avatar.json` cross-ref `animation-inventory.json` |
| G3.6 — Lip-sync quality (manual) | When paired with reference audio via EchoMimic or runtime adapter, mouth motion visually tracks phonemes. | Gate 2 script (`avatar/scripts/gate2-validate.sh`) — **operator judgment required**. |

**Pass:** G3.1–G3.5 automated checks green. G3.6 is a manual gate — record
operator verdict in `validation.json` as `g3_6_verdict: "pass" | "fail" | "skip"`.
**Fail:** Any of G3.1–G3.5 fails → exit 1.

---

### G4 — Browser Performance

**Purpose:** Verify that the avatar meets real-time rendering performance
thresholds when loaded in the browser runtime, as defined in
[`avatar-metrics.ts`](../../../avatar/web/lib/avatar-metrics.ts).

This gate requires `browser/perf.json` (generated by loading `index.html`
or the runtime test harness in a headless or instrumented browser).

| Check | Metric | Threshold | Source |
|-------|--------|-----------|--------|
| G4.1 — Connection latency | `connection_latency_ms` | < 2000 ms | Time from `START_SESSION` → `SESSION_READY`. |
| G4.2 — Audio latency | `audio_latency_ms` | < 1500 ms | Time from `SESSION_READY` → first audio frame playing. |
| G4.3 — Viseme sync | `viseme_sync_ms` | < 50 ms | Absolute delta between audio cursor and viseme cursor. |
| G4.4 — Frame drop rate | `frame_drop_rate` | < 0.01 (1%) | Dropped frames / target frames, rolling 5-second window at 60 fps. |
| G4.5 — Error recovery | `error_recovery_ms` | < 5000 ms | Time from `ERROR` state → recovered state. |
| G4.6 — Memory usage | `memory_usage_mb` | < 500 MB | `performance.memory.usedJSHeapSize` (Chromium). |
| G4.7 — WebGL context | WebGL 2 context acquired successfully | boolean | `canvas.getContext('webgl2')` not null. |
| G4.8 — GLB load time | Time to parse + render first frame of `avatar.glb` | < 5000 ms | Measured in test harness. |

**Pass:** All metric thresholds met.
**Fail:** Any threshold exceeded → exit 1, route to optimization.

**Note:** If `browser/perf.json` is absent, G4 is recorded as `"skip"` in
`validation.json` with reason `"no_browser_perf_data"`. The candidate
cannot be promoted but is not rejected — it awaits a browser test pass.

---

### G5 — Promotion Readiness

**Purpose:** Final operational checks before a candidate is promoted to
the active avatar slot.

| Check | Criterion | Source |
|-------|-----------|--------|
| G5.1 — Datadog log gate | Zero blocker patterns in the observation window. Six categories scanned: `cloudflare-524`, `openai-auth-fallback`, `tts-fallback-header`, `echomimic-5xx`, `nats-stale-narration`, `browser-stuck-working`. | `scripts/2026-04/avatar-log-validation.sh validate` (exit 0). |
| G5.2 — Render artifacts complete | `renders/` directory contains all required still/report files (`front.png`, `side.png`, `three_quarter.png`, `summary.txt`) and each is non-empty. `turntable.mp4` is required only when `avatar.json → promotion_requires_video` is true or the run is a final promotion candidate. | File check. |
| G5.3 — Provenance complete | `provenance.json` contains all required fields and `source_images` array is non-empty. | JSON validation. |
| G5.4 — All prior gates green | `validation.json` shows `pass: true` for G0–G4 (G3.6 may be `"skip"` if no live lip-sync test was run; G4 may be `"skip"` if no browser perf data). | `validation.json` inspection. |
| G5.5 — No open repair flags | No gate wrote a `repair_needed` flag into `validation.json`. | `validation.json` inspection. |
| G5.6 — index.html loads (if present) | If `index.html` exists, it loads without console errors in headless Chrome and renders `avatar.glb`. | Puppeteer/Playwright smoke test (optional). |

**Pass:** All checks green → candidate is **promotable**.
**Fail:** Any check fails → hold for operator review; do not promote.

---

## 4. Reusable Scripts Inventory

Existing scripts that can be wired into gate automation with little or no
modification:

| Script | Path | Gate(s) | Reuse status |
|--------|------|---------|-------------|
| GLB morph-target validator | `scripts/2026-04/validate-avatar-glb.py` | G2.1, G2.2, G2.3, G2.7 | **Ready.** Reads GLB binary directly, checks 77 targets, exit 0/1. |
| Blender turntable renderer | `avatar/scripts/render-glb-turntable.py` | Artifact generation (renders/) | **Ready.** Produces front/side/three_quarter PNGs + turntable MP4 + summary.txt. |
| Gate 2 lip-sync validation | `avatar/scripts/gate2-validate.sh` | G3.6 | **Ready** (manual judgment). EchoMimic V3 audio→MP4, validates HTTP + output size. |
| Datadog log gate | `scripts/2026-04/avatar-log-validation.sh` | G5.1 | **Ready.** Scans 6 blocker pattern categories, exit 0/1/2/3/4. |
| Browser metric definitions | `avatar/web/lib/avatar-metrics.ts` | G4 thresholds | **Reference only.** Defines thresholds and tracker classes; needs a test harness to produce `browser/perf.json`. |
| Protocol type definitions | `avatar/web/lib/avatar-state.ts` | G2 (viseme list), G3 (gesture cues) | **Reference only.** Canonical viseme enum and gesture cue types for cross-reference. |
| Runtime adapter | `avatar/web/lib/avatar-runtime.ts` | G4 (runtime correctness) | **Reference only.** Adapter pattern + metric bag; useful for building the browser test harness. |
| gltf-validator (external) | `npm i -g gltf-validator` / CI | G0.3 | **Ready** (external tool). Already referenced in `avatar-asset-spec.md` CI gate. |

---

## 5. Gap Analysis

Scripts and tooling that do **not yet exist** and must be built to fully
automate each gate:

| Gap | Needed for | Description | Priority |
|-----|-----------|-------------|----------|
| **Skeleton extractor** | G1.1, G1.2, G1.3 | Script to parse GLB binary and emit `skeleton-inventory.json` (bone count, hierarchy tree, joint names). Similar approach to `validate-avatar-glb.py` but targeting the skin/joint data. | High |
| **Animation extractor** | G3.1, G3.2, G3.3, G3.4 | Script to parse GLB animations and emit `animation-inventory.json` (clip names, durations, loop flags, size). | High |
| **Morph inventory extractor** | G2 (all) | Script to parse GLB and emit `morph-inventory.json`. `validate-avatar-glb.py` already reads morph targets but outputs pass/fail, not a structured inventory. Extend or wrap. | High |
| **Viseme mapping generator** | G2.6 | Script to produce `viseme-mapping.json` from a morph inventory + canonical mapping table. Possibly templated from `avatar.json → viseme_map_ref`. | Medium |
| **Bone-count CI gate** | G1.1 | Noted as TODO in `avatar-asset-spec.md` CI section (`avatar-lint.yml`). Wire skeleton extractor into CI. | Medium |
| **Browser perf harness** | G4 (all) | Headless Chromium test harness (Puppeteer/Playwright) that loads `index.html` or the runtime, runs a scripted interaction sequence, and emits `browser/perf.json` matching the metric schema from `avatar-metrics.ts`. | Medium |
| **Browser screenshot capture** | Artifact (browser/screenshot.png) | Integrate screenshot into the browser perf harness run. | Low |
| **Provenance generator** | G0.8, G5.3 | Script to assemble `provenance.json` from pipeline context (env vars, generation logs, model metadata). | Medium |
| **Checksum generator** | G0.2 | Script to generate `checksums.sha256` for the full bundle. Trivial (`sha256sum * > checksums.sha256`). | Low |
| **Bundle validator (G0 orchestrator)** | G0 (all) | Script that runs all G0 checks in sequence and writes G0 results to `validation.json`. | High |
| **Gate orchestrator** | All gates | Top-level script that runs G0–G5 in order with fail-fast, writes `validation.json` incrementally, and emits `logs/gate-summary.log`. | High |
| **index.html generator** | Artifact | Template that produces a self-contained preview page for the GLB. Could be a static template with a `{{GLB_PATH}}` placeholder. | Low |
| **VRM validation** | Future | No VRM-specific checks exist. If/when VRM export is added, extend G0 and G1 with VRM metadata and spring-bone validation. | Future |
| **Texture dimension extractor** | G1.5 | Parse GLB image/sampler metadata to verify texture resolution limits. May be part of the skeleton extractor or a separate utility. | Medium |
| **Expression behavior test** | G3.5 (automated) | Automated test that loads the avatar in browser, sends gesture cue events, and verifies the correct animation clip plays. | Low (manual gate acceptable initially) |

---

## 6. validation.json Schema

The `validation.json` file is the machine-readable record of all gate
evaluations for a candidate. It is written incrementally — each gate
appends its result.

```jsonc
{
  "candidate_id": "morgan-v3-20260415-a1b2c3",
  "bundle_version": "1.0",
  "evaluated_at": "2026-04-15T14:30:00Z",
  "gates": {
    "g0": {
      "pass": true,
      "timestamp": "2026-04-15T14:30:01Z",
      "duration_ms": 450,
      "checks": {
        "g0_1_bundle_complete": { "pass": true },
        "g0_2_checksum_integrity": { "pass": true },
        "g0_3_glb_validity": { "pass": true, "warnings": 2 },
        "g0_4_glb_size_bytes": { "pass": true, "value": 18200000 },
        "g0_5_manifest_parse": { "pass": true },
        "g0_6_manifest_hash": { "pass": true },
        "g0_7_inventory_parse": { "pass": true },
        "g0_8_provenance_parse": { "pass": true }
      }
    },
    "g1": {
      "pass": true,
      "timestamp": "2026-04-15T14:30:02Z",
      "duration_ms": 120,
      "checks": {
        "g1_1_bone_count": { "pass": true, "value": 65 },
        "g1_2_skeleton_hierarchy": { "pass": true },
        "g1_3_species_bones": { "pass": true, "family": "biped-anthro" },
        "g1_4_material_count": { "pass": true, "value": 8 },
        "g1_5_texture_limits": { "pass": true },
        "g1_6_mesh_count": { "pass": true, "value": 12, "advisory": false }
      }
    },
    "g2": {
      "pass": true,
      "timestamp": "2026-04-15T14:30:03Z",
      "duration_ms": 80,
      "checks": {
        "g2_1_arkit_blendshapes": { "pass": true, "count": 52 },
        "g2_2_oculus_visemes": { "pass": true, "count": 15 },
        "g2_3_species_addons": { "pass": true, "count": 10 },
        "g2_4_weight_range": { "pass": true },
        "g2_5_active_target_limit": { "pass": true, "max_per_mesh": 55 },
        "g2_6_viseme_mapping": { "pass": true },
        "g2_7_total_morph_count": { "pass": true, "value": 77 }
      }
    },
    "g3": {
      "pass": true,
      "timestamp": "2026-04-15T14:30:04Z",
      "duration_ms": 200,
      "checks": {
        "g3_1_required_clips": { "pass": true },
        "g3_2_clip_duration": { "pass": true },
        "g3_3_clip_loopability": { "pass": true },
        "g3_4_animation_size_kb": { "pass": true, "value": 380 },
        "g3_5_gesture_cue_mapping": { "pass": true },
        "g3_6_lipsync_verdict": { "pass": null, "verdict": "skip", "reason": "no_live_test" }
      }
    },
    "g4": {
      "pass": null,
      "verdict": "skip",
      "reason": "no_browser_perf_data",
      "timestamp": "2026-04-15T14:30:04Z"
    },
    "g5": {
      "pass": false,
      "timestamp": "2026-04-15T14:30:05Z",
      "duration_ms": 3200,
      "checks": {
        "g5_1_datadog_log_gate": { "pass": true },
        "g5_2_render_artifacts": { "pass": true },
        "g5_3_provenance_complete": { "pass": true },
        "g5_4_prior_gates_green": { "pass": false, "reason": "g4 skipped" },
        "g5_5_no_repair_flags": { "pass": true },
        "g5_6_index_html": { "pass": null, "verdict": "skip", "reason": "file_absent" }
      }
    }
  },
  "promotable": false,
  "promotion_blockers": ["g4 not evaluated"],
  "repair_routing": []
}
```

---

## 7. Cross-Reference: Asset Spec Alignment

This contract **extends** (does not replace) [`avatar-asset-spec.md`](avatar-asset-spec.md).
The relationship:

| Concern | Asset Spec | This Contract |
|---------|-----------|---------------|
| File format & limits | Defines GLB format, compression, size limits | G0 enforces these limits |
| Rig families & skeleton | Defines 4 families, bone hierarchy, bone limits | G1 validates skeleton against spec |
| Blendshapes | Defines 52 ARKit + 15 Oculus + species add-ons | G2 validates morph target presence |
| Materials & textures | Defines material/texture count & resolution limits | G1 validates material/texture limits |
| Animation clips | Defines 5 required clips, duration, loop, size | G3 validates clip inventory |
| avatar.json manifest | Defines schema with all required fields | G0 validates manifest parse & hash |
| CI gate (`avatar-lint.yml`) | References gltf-validator, manifest check, size check, bone count (TODO) | This contract provides the full gate spec that CI should implement |
| Browser runtime | Not covered | G4 defines runtime performance thresholds |
| Operational readiness | Not covered | G5 defines Datadog + render + provenance checks |
| Artifact bundle | Not covered | § 2 defines the full bundle manifest |
| Gate routing & repair | Not covered | § 3 defines fail-fast ordering and repair routing |

---

## 8. Implementation Sequence

Recommended order for building the gate automation (contract definition
only — implementation is a separate effort):

1. **Phase 1 — Extractors:** skeleton-inventory, morph-inventory, animation-inventory extractors (all parse GLB binary, emit JSON).
2. **Phase 2 — G0 orchestrator:** bundle completeness + checksum + glTF-validator + manifest checks.
3. **Phase 3 — G1 + G2 gates:** wire extractors into gate scripts; extend `validate-avatar-glb.py` or build parallel validators.
4. **Phase 4 — G3 gate:** animation inventory checks + gesture cue mapping validation.
5. **Phase 5 — Browser harness:** Playwright/Puppeteer test harness → `browser/perf.json` → G4 gate.
6. **Phase 6 — G5 gate + orchestrator:** wire Datadog gate, provenance check, and top-level gate runner.
7. **Phase 7 — CI integration:** wire gate orchestrator into `avatar-lint.yml` and candidate promotion workflow.
