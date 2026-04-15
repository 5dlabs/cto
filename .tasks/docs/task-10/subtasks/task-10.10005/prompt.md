<identity>
You are atlas working on subtask 10005 of task 10.
</identity>

<context>
<scope>
Create a root-level Makefile (or justfile) with targets for build, test, lint, deploy-devnet, and demo that mirror the CI pipeline for local development reproducibility.
</scope>
</context>

<implementation_plan>
1. Create `Makefile` at the repository root.
2. Define targets:
   - `make build`: Run `anchor build`. Print program keypair path and .so size.
   - `make test`: Run `anchor test`. Optionally accept `TEST_FILTER` env var for running specific test files.
   - `make lint`: Run `cargo clippy --workspace -- -D warnings && cargo fmt --all --check`. Then run `cd packages/receipt-uploader && bun run lint`, `cd cli && bun run lint`, `cd app/dashboard && bun run lint`.
   - `make fmt`: Run `cargo fmt --all` (auto-format, unlike lint which only checks).
   - `make deploy-devnet`: Check for `OPERATOR_KEYPAIR_PATH` env var. Run `anchor deploy --provider.cluster devnet --provider.wallet $(OPERATOR_KEYPAIR_PATH)`. Print deployed program ID.
   - `make demo`: Run the demo CLI flow (from Task 7) end-to-end against localnet or devnet.
   - `make clean`: Run `anchor clean`, remove node_modules, clear build artifacts.
   - `make install`: Install all dependencies — `bun install` in all packages, verify Solana CLI and Anchor CLI versions.
3. Add `.PHONY` declarations for all targets.
4. Add a default target that prints available targets: `make help`.
5. Test each target locally to verify they work with the project structure.
6. Add a note in the Makefile header about required tool versions (Solana CLI, Anchor CLI, Bun, Rust).
</implementation_plan>

<validation>
Makefile exists at repo root. `make help` prints all available targets. `make build` runs anchor build successfully. `make test` runs the full test suite. `make lint` runs all linters (Rust + TypeScript). `make fmt` auto-formats code. `make clean` removes build artifacts. Each target exits with code 0 on a clean repo state. `make test` locally reproduces the same test results as CI.
</validation>