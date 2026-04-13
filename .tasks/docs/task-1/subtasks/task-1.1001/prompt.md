<identity>
You are bolt working on subtask 1001 of task 1.
</identity>

<context>
<scope>
Initialize the cto-pay repo using Anchor CLI and configure the full project directory structure, Rust workspace, TypeScript workspaces, tooling configs, and all dependency pinning.
</scope>
</context>

<implementation_plan>
1. Create repo at `https://github.com/5dlabs/cto-pay`. Run `anchor init cto-pay` with Anchor 0.30.1.
2. Pin Anchor 0.30.1 and Solana CLI 1.18+ in Anchor.toml.
3. Create full directory structure: `programs/cto-pay/src/lib.rs`, `programs/cto-pay/Cargo.toml`, `tests/`, `cli/`, `cli/package.json`, `cli/tsconfig.json`, `cli/src/index.ts`, `scripts/`, `config/`, `.github/` stub.
4. Configure workspace `Cargo.toml` with `[profile.release]` overflow-checks=true, lto="thin".
5. Configure `programs/cto-pay/Cargo.toml` with `anchor-lang = "0.30.1"`, `anchor-spl = "0.30.1"`, `solana-program = "1.18"`, rust-version = "1.75", edition 2021.
6. Root `package.json`: workspaces `["tests", "cli"]`, devDependencies: `@coral-xyz/anchor`, `@solana/web3.js`, `@solana/spl-token`, `typescript`, `@types/node`.
7. `cli/package.json`: name `cto-pay`, bin `cto-pay`, dependencies: `@coral-xyz/anchor`, `@solana/web3.js`, `@solana/spl-token`, `commander`, `bs58`.
8. Pin Bun 1.1+ in `.tool-versions` or `package.json#engines`.
9. Add `biome.json` for TypeScript linting, `.gitignore` (include `devnet-operator-keypair.json`, `target/`, `node_modules/`), `README.md`.
10. Ensure `lib.rs` has a minimal `declare_id!` and `#[program]` module so `anchor build` succeeds.
</implementation_plan>

<validation>
Run `anchor build` — compiles with zero errors on the program skeleton. Run `bun install` at root — all dependencies resolve with no missing packages. Verify `cli/package.json` has `bin` field pointing to correct entry. Verify `.tool-versions` or engines field pins Bun 1.1+. Verify biome.json exists. Verify workspace Cargo.toml has overflow-checks=true in release profile.
</validation>