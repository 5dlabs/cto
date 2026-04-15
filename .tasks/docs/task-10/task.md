## CI/CD Pipeline & Devnet Deployment Automation (Atlas - CI/CD Platforms)

### Objective
Build the CI/CD pipeline that builds, tests, and deploys the Anchor program to Solana devnet, runs the full integration test suite, builds the web dashboard, and packages the demo CLI. This ensures reproducible builds and automated deployment for the hackathon submission.

### Ownership
- Agent: atlas
- Stack: CI/CD Platforms
- Priority: medium
- Status: pending
- Dependencies: 2, 3, 4, 6, 7, 8

### Implementation Details
1. Create `.github/workflows/ci.yml` (or equivalent CI config) with the following stages:
   **Stage 1 — Build & Lint (parallel jobs):**
   - **Job: anchor-build** — Install Solana CLI (v1.18.x), Anchor CLI (v0.30.x), Rust toolchain (stable). Run `anchor build`. Cache `target/` and `.anchor/` directories. Upload `target/deploy/cto_billing.so` and `target/idl/cto_billing.json` as artifacts.
   - **Job: rust-lint** — Run `cargo clippy --workspace -- -D warnings`. Run `cargo fmt --check`.
   - **Job: ts-lint** — Install Bun. Run `bun install` in `packages/receipt-uploader/`, `cli/`, `app/dashboard/`. Run lint in each.
   - **Job: dashboard-build** — `cd app/dashboard && bun run build`. Upload `.next/` or `out/` as artifact.
2. **Stage 2 — Test (depends on Stage 1):**
   - **Job: anchor-test** — Start solana-test-validator as a background service. Deploy the built .so file. Run `anchor test --skip-build --skip-deploy` (use pre-deployed program). Alternatively, use `anchor test` which handles localnet lifecycle. Run all test suites from Task 8. Upload test results (JUnit XML) as artifacts.
   - **Job: receipt-uploader-test** — Run `cd packages/receipt-uploader && bun test`. Unit tests only (skip Irys integration tests in CI unless devnet credentials are available).
3. **Stage 3 — Deploy to Devnet (manual trigger or on main branch merge):**
   - **Job: deploy-devnet** — Requires secrets: `OPERATOR_KEYPAIR` (base58 or JSON array), `SOLANA_DEVNET_RPC_URL`.
   - Decode operator keypair to file.
   - `solana airdrop 2 --url devnet` to ensure deployment SOL.
   - `anchor deploy --provider.cluster devnet --provider.wallet /tmp/operator.json`.
   - Record deployed program ID.
   - Run smoke test: execute a subset of the demo CLI (initialize operator, create customer, deposit small amount) against devnet to verify deployment.
   - Upload deployment manifest (program ID, deploy TX, timestamp) as artifact.
4. **Stage 4 — Package Release:**
   - Create a GitHub Release (or tag) with:
     - `cto_billing.so` (program binary).
     - `cto_billing.json` (IDL).
     - Dashboard static export (zip).
     - Demo CLI bundle.
     - Security review document.
5. Create `Makefile` or `justfile` at repo root with targets mirroring CI stages for local development:
   - `make build`, `make test`, `make lint`, `make deploy-devnet`, `make demo`.
6. Add branch protection rules recommendation in `docs/ci-cd.md`: require CI pass on PRs, protect main branch.
7. Set up Anchor.toml `[programs.devnet]` section with the deployed program ID (updated after first deploy).
8. Configure caching: Rust target dir, Bun node_modules, Solana CLI binaries — to keep CI under 10 minutes.

### Subtasks
- [ ] Create Stage 1 CI workflow — build and lint jobs: Create the GitHub Actions workflow file with Stage 1 parallel jobs: anchor-build (Solana/Anchor/Rust toolchain setup, program compilation, artifact upload), rust-lint (clippy + fmt), ts-lint (Bun-based linting for all TypeScript packages), and dashboard-build (Next.js build with artifact upload).
- [ ] Create Stage 2 CI workflow — test jobs with solana-test-validator: Add Stage 2 test jobs to the CI workflow: anchor-test (running the full Anchor integration test suite with solana-test-validator and JUnit XML output) and receipt-uploader-test (Bun unit tests). These jobs depend on Stage 1 anchor-build.
- [ ] Create Stage 3 CI workflow — devnet deployment with smoke test: Add the devnet deployment job to the CI workflow with manual trigger support, operator keypair secret handling, SOL airdrop, anchor deploy command, and a post-deployment smoke test that runs basic CLI operations against devnet.
- [ ] Create Stage 4 CI workflow — GitHub Release packaging: Add a release packaging job that creates a GitHub Release with all deliverables: program binary (.so), IDL JSON, dashboard static export, demo CLI bundle, and security review document.
- [ ] Create Makefile with local development targets mirroring CI stages: Create a root-level Makefile (or justfile) with targets for build, test, lint, deploy-devnet, and demo that mirror the CI pipeline for local development reproducibility.
- [ ] Configure CI caching, Anchor.toml devnet section, and write CI documentation: Optimize CI caching for Rust target directory, Bun node_modules, and Solana CLI binaries to keep pipeline under 10 minutes. Configure Anchor.toml devnet program ID section. Write docs/ci-cd.md with pipeline overview and branch protection recommendations.