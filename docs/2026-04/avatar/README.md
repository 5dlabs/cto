# Morgan avatar current plan

This directory contains the **active Morgan avatar execution plan** and the
small set of operational docs needed to run or validate that plan.

Broad model/provider catalogs live in `.avatar-docs/` so this directory does not
mix the current plan with every available option.

## Active planning docs

| File | Purpose |
| --- | --- |
| `model-dag-plan.md` | Current gated DAG strategy: get a high-quality GLB/VRM-style Morgan asset working first, then optimize cost. |
| `asset-feasibility.md` | Acceptance gates for source conditioning, GLB validation, renders, head viability, rigging, and face controls. |
| `validation.md` | Runtime/browser validation gate for `/echo-turn` and provider-switch tests. |

## Historical or runtime reference docs

| File | Status |
| --- | --- |
| `provider-switch.md` | Runtime/provider-switch reference for the existing `/echo-turn` and LemonSlice/LiveKit paths. Useful context, not the active asset-provisioning plan. |
| `phase4-disposition.md` | Historical WS-F/Phase 4 disposition. Keep for archaeology; do not use it as the current model strategy. |

## Research/catalog docs

Use `.avatar-docs/` for:

- available model catalogs,
- provider/credit maps,
- Hugging Face and Scenario indexes,
- talking-portrait / 3D-generation repo summaries,
- local clone caches for research.
