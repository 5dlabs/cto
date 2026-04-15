<identity>
You are atlas working on subtask 10001 of task 10.
</identity>

<context>
<scope>
Create the GitHub Actions workflow file with Stage 1 parallel jobs: anchor-build (Solana/Anchor/Rust toolchain setup, program compilation, artifact upload), rust-lint (clippy + fmt), ts-lint (Bun-based linting for all TypeScript packages), and dashboard-build (Next.js build with artifact upload).
</scope>
</context>

<implementation_plan>
1. Create `.github/workflows/ci.yml` with `on: [push, pull_request]` trigger.
2. Define the `anchor-build` job:
   - Use `ubuntu-latest` runner.
   - Install Rust stable toolchain via `dtolnay/rust-toolchain@stable`.
   - Install Solana CLI v1.18.x: `sh -c "$(curl -sSfL https://release.solana.com/v1.18.26/install)"`.
   - Install Anchor CLI v0.30.x: `cargo install --git https://github.com/coral-xyz/anchor avm && avm install 0.30.1 && avm use 0.30.1`.
   - Run `anchor build`.
   - Upload `target/deploy/cto_billing.so` and `target/idl/cto_billing.json` as artifacts using `actions/upload-artifact@v4`.
3. Define the `rust-lint` job (parallel with anchor-build):
   - Install Rust stable + clippy + rustfmt components.
   - Run `cargo clippy --workspace -- -D warnings`.
   - Run `cargo fmt --all --check`.
4. Define the `ts-lint` job (parallel):
   - Install Bun via `oven-sh/setup-bun@v1`.
   - Run `bun install` in `packages/receipt-uploader/`, `cli/`, `app/dashboard/`.
   - Run lint command in each package (e.g., `bun run lint` or `bunx eslint .`).
5. Define the `dashboard-build` job (parallel):
   - Install Bun, run `cd app/dashboard && bun install && bun run build`.
   - Upload build output (`.next/` or `out/`) as artifact.
6. Configure caching for Rust target directory (`actions/cache@v4` with key based on Cargo.lock hash) and Bun node_modules.
7. Verify YAML syntax is valid and all jobs are at the same `needs` level (no dependencies — Stage 1 is all parallel).
</implementation_plan>

<validation>
The workflow file `.github/workflows/ci.yml` exists with 4 parallel Stage 1 jobs. Push a commit and verify: anchor-build job installs Solana CLI + Anchor CLI and produces cto_billing.so artifact. rust-lint job runs clippy and fmt. ts-lint job runs lint in all 3 TypeScript packages. dashboard-build produces a build artifact. All 4 jobs run in parallel (not sequentially). Cache is configured for Rust target dir.
</validation>