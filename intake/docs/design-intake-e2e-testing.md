# Design Intake E2E Testing

This document defines how to validate the design intake flow end-to-end and how to include it in routine testing.

## Scope

The test coverage in this runbook validates:

- Provider routing across the OSS provider catalog (default: `stitch`; open-string for any catalog entry)
- Artifact generation under `.intake/design/*`
- Bundle materialization under `.tasks/design/*`
- Compatibility with downstream `design-review` and `design-deliberation` steps

## Test Layers

Use all three layers for confidence:

1. **Unit tests (fast):**
   - `apps/intake-agent/src/operations/design-intake.test.ts`
2. **Provider dry-run integration (medium):**
   - `intake/scripts/design-intake-dry-run.sh`
3. **Pipeline E2E (full):**
   - `intake/workflows/pipeline.lobster.yaml`

## Prerequisites

- Build or install `intake-agent` binary:
  - `cd apps/intake-agent && bun run build`
- Required credentials by provider:
  - `stitch`: `STITCH_API_KEY`
  - OSS providers (shadcn registries, headless kits, TanStack, etc.): no credentials required (catalog-driven)
- Optional:
  - `INTAKE_PREFLIGHT_BRIDGES_SKIP=true` for local non-bridge runs

## 1) Unit Tests (Fast Gate)

Run:

```bash
cd apps/intake-agent
bun test
```

Expected:

- `design-intake.test.ts` passes mode-routing checks
- No regressions in existing `prd-research.test.ts`

## 2) Provider Dry-Run (Integration Gate)

Run:

```bash
bash intake/scripts/design-intake-dry-run.sh
```

This executes runs across the supported provider modes (`stitch`, `auto`).

Expected artifacts per mode under `.intake/design-dry-run/<mode>/`:

- `response.json` with `.success == true`
- `design-context.json`
- `component-library.json`
- `design-system.md`

## 3) Full Pipeline E2E (Workflow Gate)

Run a focused pipeline invocation:

```bash
lobster run --mode tool intake/workflows/pipeline.lobster.yaml --args-json '{
  "project_name": "design-intake-e2e",
  "prd_path": ".intake/run-prd.txt",
  "design_mode": "ingest_plus_stitch",
  "design_provider": "stitch",
  "design_prompt": "Modernize UI while preserving product tone",
  "design_urls": "https://example.com",
  "include_codebase": false,
  "deliberate": false
}'
```

Expected outputs:

- `.intake/design/design-context.json`
- `.intake/design/candidates.normalized.json`
- `.intake/design/component-library.json`
- `.intake/design/design-system.md`
- `.tasks/design/manifest.json` containing:
  - `files.stitch_*`
  - `files.normalized_candidates`
  - `files.component_library`
  - `files.design_system`

## Optional: Optimizer POC (baseline vs candidate)

Use this when you want a quick "keep/discard" signal for one design-intake knob without changing core workflows.

Run:

```bash
bash intake/scripts/design-intake-optimizer-poc.sh
```

Default comparison:

- baseline: `design_provider=stitch`
- candidate: `design_provider=both`
- decision threshold: candidate average score must beat baseline by `>= 5`

Useful overrides:

```bash
DESIGN_POC_RUNS=3 \
DESIGN_POC_BASELINE_PROVIDER=stitch \
DESIGN_POC_CANDIDATE_PROVIDER=both \
DESIGN_POC_KEEP_THRESHOLD=5 \
bash intake/scripts/design-intake-optimizer-poc.sh
```

Outputs (timestamped):

- `.intake/optimizer-poc/design-intake/<timestamp>/run-manifest.json`
- `.intake/optimizer-poc/design-intake/<timestamp>/summary.json`
- `.intake/optimizer-poc/design-intake/<timestamp>/report.md`

Notes:

- This is a lightweight scoring harness for experimentation, not a replacement for the full Lobster pipeline gate.
- If provider credentials are missing, expect low-signal runs and frequent `DISCARD` or `NO_DECISION`.

## Assertions Checklist

Use this checklist after dry-run and E2E:

- [ ] `design-context.json` has `providerMode`
- [ ] `design-context.json` has `providers.stitch`
- [ ] `normalized_candidates` exists and includes `provider` on entries
- [ ] `component-library.json` exists and has `tokens`, `primitives`, `patterns`
- [ ] `design-system.md` exists
- [ ] Pipeline does not fail when a provider returns no candidates (graceful degradation)

## How to Include in Routine Tests

### Local developer flow

Use this sequence before merging design-intake changes:

```bash
cd apps/intake-agent && bun test
cd /Users/jonathon/5dlabs/cto
bash intake/scripts/design-intake-dry-run.sh
```

Run full pipeline E2E only when provider wiring, workflow behavior, or bridge integration changed.

### CI recommendation

Add two stages:

1. **Required on every PR touching design intake paths**
   - `cd apps/intake-agent && bun test`
2. **Required on workflow/provider changes**
   - `bash intake/scripts/design-intake-dry-run.sh`

Suggested path filters for stage 2:

- `apps/intake-agent/src/operations/design-intake.ts`
- `intake/workflows/pipeline.lobster.yaml`
- `intake/workflows/design-deliberation.lobster.yaml`
- `intake/prompts/**`
- `intake/schemas/**`

## Troubleshooting

- Missing Stitch key:
  - check `STITCH_API_KEY`
  - inspect `.intake/design/auth-discovery.json`
- Variant generation skipped:
  - verify generated entries in `.intake/design/candidates.normalized.json`
- Deliberation skipped unexpectedly:
  - verify design/mockup counts in pipeline logs and `.intake/design-decision-points.json`

## Notes on Terminology

- **Design system**: full visual and interaction system (tokens + rules + components)
- **Component library**: implementation-focused reusable components and patterns
- Current intake emits both:
  - `design-system.md`
  - `component-library.json`
