# Task 1.1 – Extend Controller for Cursor CLI Type

## Dependencies
- Completion of Task 0.1 (requirements baseline).

## Parallelization Guidance
- Unblocks Tasks 1.2 and 1.3 once enum/API changes land.

## Task Prompt
Introduce `Cursor` as a first-class `CLIType` inside the controller so downstream code can branch on it just like `Codex` and `Claude`.

Detailed steps:
1. Update `controller/src/cli/types.rs`:
   - Add `Cursor` variant to the `CLIType` enum with serde (string) aliases (`"cursor"`, `"Cursor"`).
   - Extend `Display`, `FromStr`, and helper methods to recognise the new variant without breaking existing pattern matches.
2. Ensure CRDs understand the new variant:
   - Modify `controller/src/crds/coderun.rs` so serde (de)serialisation of `CLIConfig.cliType` handles `Cursor`.
   - Regenerate or update JSON schemas (`infra/charts/controller/crds/coderun-crd.yaml`) if needed.
3. Adapter plumbing:
   - Create a skeleton adapter module under `controller/src/cli/adapters/cursor.rs` (mirroring `codex.rs`) with TODOs for command construction.
   - Wire the adapter into `controller/src/cli/adapters/mod.rs`, `factory.rs`, and `bridge.rs` so selecting `CLIType::Cursor` instantiates the new adapter.
4. Fallback safety:
   - Update any exhaustive `match` statements (e.g., `controller/src/tasks/code/templates.rs`, `resources.rs`) to include `Cursor`, returning meaningful errors/panics if invoked before implementation is ready.
5. Documentation comments explaining that Cursor shares much of the Codex flow but uses the `cursor-agent` binary.

## Acceptance Criteria
- `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test` succeed.
- `test-templates` binary still compiles (even if Cursor templates are not yet hooked up, the generator must recognise the enum value without panic).
- Running `rg "todo" controller/src/cli` shows a placeholder in the new adapter documenting pending work.
- No dead code warnings about the new variant (ensure it’s exercised in unit tests where appropriate, e.g., extend `cli::types::tests::deserializes_case_insensitive_variants`).

## Implementation Notes / References
- Follow patterns in `controller/src/cli/adapters/codex.rs` for struct layout and trait impls.
- Remember to update `controller/src/cli/discovery.rs` if CLI discovery logic enumerates supported types.
- Helm/ConfigMap tasks (Group 2) rely on the enum string value `cursor`; align serde alias accordingly to avoid YAML casing mismatches.
