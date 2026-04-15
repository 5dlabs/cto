<identity>
You are bolt working on subtask 1001 of task 1.
</identity>

<context>
<scope>
Create the full Anchor workspace with directory structure, Anchor.toml, Cargo.toml, skeleton lib.rs, and placeholder directories for tests, app, and CLI.
</scope>
</context>

<implementation_plan>
1. Create top-level `programs/cto-billing/` directory. 2. Create `Anchor.toml` configured for Solana devnet (https://api.devnet.solana.com) with a placeholder program ID generated via `solana-keygen grind`. 3. Create `programs/cto-billing/Cargo.toml` with dependencies: anchor-lang 0.30.x, anchor-spl 0.30.x, spl-token 4.x. 4. Create `programs/cto-billing/src/lib.rs` with `declare_id!` macro using the placeholder ID and an empty `#[program]` module. 5. Create `tests/` directory with `tsconfig.json` configured for Anchor/mocha test runner and a placeholder test file that imports `@coral-xyz/anchor`. 6. Create `app/` and `cli/` placeholder directories with README.md stubs. 7. Verify `anchor build` compiles the skeleton program without errors.
</implementation_plan>

<validation>
Run `anchor build` — compilation succeeds with zero errors. Verify directory structure exists: programs/cto-billing/src/lib.rs, tests/, app/, cli/. Verify Anchor.toml contains devnet cluster URL. Verify Cargo.toml contains anchor-lang, anchor-spl, spl-token dependencies.
</validation>