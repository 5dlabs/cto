<identity>
You are bolt, the Kubernetes/Helm implementation agent. You own task 1 end-to-end.
</identity>

<context>
<task_overview>
Task 1: Bootstrap Solana Dev Infrastructure & Anchor Workspace (Bolt - Kubernetes/Helm)
Set up the Solana development toolchain, Anchor workspace scaffolding, devnet configuration, Irys/Bundlr devnet credentials, and supporting infrastructure (local validator option, USDC devnet mint). This is the foundational layer that all subsequent Solana-related tasks depend on.
Priority: high
Dependencies: None
</task_overview>
</context>

<implementation_plan>
1. Create a new top-level directory `programs/cto-billing/` with the Anchor workspace structure:
   - `Anchor.toml` configured for Solana devnet (https://api.devnet.solana.com) with program ID placeholder.
   - `programs/cto-billing/Cargo.toml` with anchor-lang 0.30.x, anchor-spl 0.30.x, spl-token 4.x dependencies.
   - `programs/cto-billing/src/lib.rs` with skeleton `declare_id!` and empty program module.
   - `tests/` directory for TypeScript integration tests (tsconfig, mocha/anchor test runner).
   - `app/` directory placeholder for the web dashboard.
   - `cli/` directory placeholder for the demo CLI.
2. Create a `Makefile` or `justfile` with targets: `build`, `test`, `deploy-devnet`, `localnet-up`, `localnet-down`.
3. Create a Helm chart or Kubernetes Job manifest (`infra/solana-dev/`) that provisions:
   - A Solana local validator pod (solanalabs/solana:v1.18.x) for CI testing, exposing RPC on port 8899.
   - A ConfigMap `cto-billing-infra-endpoints` with keys: `SOLANA_RPC_URL`, `SOLANA_WS_URL`, `IRYS_RPC_URL`, `IRYS_TOKEN`.
4. Generate devnet operator keypair and treasury keypair (store as Kubernetes Secrets via the existing OpenBao pipeline described in docs/secrets-management.md). Add `OPERATOR_KEYPAIR_PATH` and `TREASURY_KEYPAIR_PATH` to the ConfigMap.
5. Document how to obtain devnet USDC: create a script `scripts/setup-devnet-usdc.sh` that:
   - Airdrops SOL to operator and test wallets.
   - Creates a devnet SPL token mint (acting as USDC for testing) or uses the official devnet USDC mint (EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v if available, otherwise create a mock mint).
   - Mints test USDC to the operator and demo customer wallets.
6. Create `.env.example` and `scripts/env-setup.sh` documenting all required environment variables.
7. Ensure `anchor build` succeeds with the skeleton program and `anchor test --skip-deploy` runs without error.
</implementation_plan>

<acceptance_criteria>
Run `anchor build` in the workspace — compilation succeeds with zero errors. Run `anchor test --skip-deploy` — test runner initializes without error. Apply the Helm chart to a dev namespace — Solana validator pod reaches Ready state within 90s. Run `solana cluster-version --url http://solana-validator:8899` — returns a valid version string. Verify ConfigMap `cto-billing-infra-endpoints` exists and contains `SOLANA_RPC_URL`, `IRYS_RPC_URL` keys. Run `scripts/setup-devnet-usdc.sh` — operator wallet has >1 SOL and >1000 test USDC balance confirmed via `spl-token balance`.

See also: acceptance.md in this task directory for the checklist version.
</acceptance_criteria>

<subtasks>
- Scaffold Anchor workspace directory structure and configuration files: Create the full Anchor workspace with directory structure, Anchor.toml, Cargo.toml, skeleton lib.rs, and placeholder directories for tests, app, and CLI.
- Create Kubernetes namespace and Solana local validator deployment: Create Helm chart or Kubernetes manifests in infra/solana-dev/ that provision a Solana local validator pod for CI testing, single-replica, exposing RPC on port 8899.
- Generate operator and treasury keypairs and store via OpenBao secrets pipeline: Generate devnet operator and treasury keypairs, store them as Kubernetes Secrets using the existing OpenBao pipeline, and make secret paths available for reference.
- Create cto-billing-infra-endpoints ConfigMap with all environment references: Create the ConfigMap cto-billing-infra-endpoints containing SOLANA_RPC_URL, SOLANA_WS_URL, IRYS_RPC_URL, IRYS_TOKEN, OPERATOR_KEYPAIR_PATH, and TREASURY_KEYPAIR_PATH keys.
- Create devnet USDC setup script with SOL airdrop and SPL token minting: Create scripts/setup-devnet-usdc.sh that airdrops SOL to operator and test wallets, creates or references a devnet USDC mint, and mints test USDC tokens to relevant wallets.
- Create build tooling, environment documentation, and end-to-end verification: Create Makefile/justfile with all build targets, .env.example, env-setup.sh, and verify the entire bootstrap works end-to-end (anchor build, anchor test --skip-deploy, validator pod health).
</subtasks>