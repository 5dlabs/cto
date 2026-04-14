<identity>
You are rex working on subtask 1001 of task 1.
</identity>

<context>
<scope>
Scaffold the Anchor workspace at programs/cto-billing/ with Anchor v0.30+, configure Cargo.toml dependencies, Anchor.toml settings, and lib.rs entry point with program ID placeholder and module declarations.
</scope>
</context>

<implementation_plan>
1. Run `anchor init cto-billing` or manually create the workspace structure under `programs/cto-billing/`.
2. Set program ID placeholder in both `lib.rs` (`declare_id!(...)`) and `Anchor.toml`.
3. Configure Cargo.toml with dependencies: anchor-lang 0.30+, anchor-spl (for token operations), solana-program, and sha2 crate (for hash_seed).
4. Set up module structure in lib.rs: declare `mod state;`, `mod instructions;`, `mod utils;`, `mod errors;`.
5. Create empty module files: `state/mod.rs`, `instructions/mod.rs`, `utils/mod.rs`, `errors.rs`.
6. Verify `anchor build` runs (even if empty) without configuration errors.
</implementation_plan>

<validation>
Run `anchor build` — the workspace compiles without configuration errors. Verify Anchor.toml has correct program name and cluster settings. Verify Cargo.toml includes anchor-lang, anchor-spl, and sha2 dependencies.
</validation>