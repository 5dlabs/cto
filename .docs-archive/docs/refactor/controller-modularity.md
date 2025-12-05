---
description: controller refactor plan for modularity and pedantic compliance
---

# Controller Modularity & Clippy Pedantic Plan

## Current Hotspots

| File | Lines | Concerns |
| --- | ---: | --- |
| `controller/src/tasks/code/templates.rs` | ~2,960 | Multi-CLI renderer, 20+ responsibilities, repeated Handlebars plumbing, pedantic `too_many_lines` + `format_push_string` |
| `controller/src/tasks/code/resources.rs` | ~1,760 | PVC/ConfigMap/Job creation, cleanup, CLI config munging all in one `impl`, heavy Kubernetes logic |
| `controller/src/tasks/code/controller.rs` | ~1,400 | CodeRun reconciliation, GitHub orchestration, retries, status updates |
| `controller/src/tasks/docs/resources.rs` | ~1,100 | Mirrors code resources with docs-specific quirks |
| `controller/src/cli/bridge.rs` | ~858 | Bridging logic for Cursor/Factory adapters, repeated formatting |

Pedantic lints surface primarily in these files because long functions (>100 lines) and multi-purpose helpers make it impossible to reason about individual responsibilities. We also duplicate adapter plumbing, Kubernetes resource builders, and CLI-specific serialization.

## Refactor Principles

1. **Single Responsibility Modules**  
   Each resource phase (PVC, ConfigMap, Job, Cleanup) and each CLI renderer (Claude, Cursor, Factory, OpenCode, Codex) gets its own module with a narrow API.  
2. **Stateless Builders + Context Structs**  
   Replace ad-hoc `json!` chains with typed builder structs (e.g., `JobSpecBuilder`, `TemplateContext`). This keeps pedantic satisfied and aides testing.
3. **Trait-based CLI Overrides**  
   Introduce `CliRender` trait so new adapters only implement `fn build_memory`, `fn build_container`, etc., without editing the giant template file.
4. **Incremental Migration**  
   Split files progressively, verifying pedantic + tests after each slice to avoid destabilizing the branch.

## Planned Module Layout

```
controller/src/tasks/code/
├── resources/
│   ├── mod.rs                // thin facade (new/cleanup orchestrations)
│   ├── pvc.rs                // ensure/build PVC
│   ├── configmap.rs          // configmap name/data/labels/owner
│   ├── job.rs                // job & pod spec builders, requirements parsing
│   ├── cleanup.rs            // job+configmap GC
│   ├── cli.rs                // CLI config merge, provider resolution, images
│   └── tests.rs
└── templates/
    ├── mod.rs                // dispatcher
    ├── shared.rs             // render settings, hook scripts, guidelines
    ├── claude.rs
    ├── cursor.rs
    ├── opencode.rs
    ├── factory.rs
    └── codex.rs
```

Follow-up work mirrors this pattern for `docs/resources.rs` (shared PVC + ConfigMap builder, docs-specific job builder) and eventually `tasks/code/controller.rs` (split reconciliation into `state`, `actions`, `status`).

## Implementation Steps

1. **Module Scaffolding (resources)**
   - Convert `resources.rs` into directory module (`mod.rs + submodules`).
   - Move `ensure_pvc_exists`/`build_pvc_spec` → `pvc.rs`.
   - Move ConfigMap helpers + labels → `configmap.rs`.
   - Move cleanup routines → `cleanup.rs`.
   - Extract CLI config merge utilities + adapter defaults → `cli.rs`.
   - Add `tests.rs` for existing merge tests.
   - Keep Job builder in `mod.rs` initially to keep diff small; migrate after everything compiles.

2. **Job Builder Extraction**
   - Introduce `JobContext` struct capturing `code_run`, `cli_type`, `workspace`, `env`.
   - Move `create_or_get_job`, `create_job`, `build_job_spec`, `process_task_requirements` into `job.rs`.
   - Replace raw `json!` usage with typed helper methods where possible (while preserving behavior).

3. **Template Refactor**
   - Rename `templates.rs` → directory module.
   - Create `shared.rs` for render settings, guidelines, hook scripts.
   - One file per CLI (Claude/Cursor/Codex/Factory/OpenCode) implementing `TemplateBundle` trait returning `BTreeMap`.
   - Introduce typed context structs instead of nested `Value`.

4. **Docs Resource Parity**
   - After code resources stabilized, apply same module split to `docs/resources.rs`, reusing shared PVC/ConfigMap builders with generics.

5. **Controller + MCP follow-up**
   - Reconcile logic: move GitHub interactions into `github.rs`, workspace orchestration into `workspace.rs`.
   - MCP server: split monolithic request handling into `router`, `tool_registry`, `session_state`.

## Testing & Verification

For each migration slice:
1. `cargo fmt`
2. `cargo clippy -p controller --all-targets --all-features -- -D warnings -W clippy::pedantic`
3. `cargo test -p controller`

Additionally, integration scenarios:
- CodeRun PVC reuse (shared vs isolated)  
- ConfigMap owner updates with concurrent jobs  
- Task requirements env injection + secret visibility  
- CLI merge logic for GitHub app overrides  

## Next Actions

1. Implement Step 1 (resources scaffolding) with zero behavior changes; re-run pedantic/tests.  
2. Prioritize Job builder extraction (Step 2) to unlock pedantic fixes around `too_many_lines`.  
3. Draft similar plan for `templates.rs` before editing code (identify per-CLI responsibilities).  
4. Track progress in this doc as modules move (checkbox style) and surface new lint targets.

This plan keeps the refactor incremental, continuously pedantic-compliant, and ready for follow-up template/docs splits without destabilizing the workflow controller.

