<identity>
You are bolt, the Anchor CLI, SeaweedFS, External Secrets Operator, Kubernetes implementation agent. You own task 1 end-to-end.
</identity>

<context>
<task_overview>
Task 1: Dev Infra Bootstrap — Repo Scaffold, Secrets, Receipt Storage, ConfigMap (Bolt - Anchor CLI / K8s / SeaweedFS)
Bootstrap the cto-pay repo with Anchor project structure and provision all cluster-side infrastructure: SeaweedFS receipt bucket, operator keypair secret via the 1Password → OpenBao → External Secrets pipeline, and a cto-pay-infra-endpoints ConfigMap aggregating connection strings for RPC and storage. All downstream tasks depend on this.
Priority: high
Dependencies: None
</task_overview>
</context>

<implementation_plan>
1. **Scaffold cto-pay repo** at `https://github.com/5dlabs/cto-pay`:
   - Run `anchor init cto-pay` with Anchor 0.30.1 (pin in Anchor.toml).
   - Set Solana CLI version to 1.18+ in Anchor.toml.
   - Directory structure:
     ```
     programs/cto-pay/src/lib.rs      ← Anchor program entry
     programs/cto-pay/Cargo.toml
     tests/                            ← Bankrun tests (TS)
     cli/                              ← CLI package (Bun)
     cli/package.json
     cli/tsconfig.json
     cli/src/index.ts
     scripts/                          ← Devnet setup scripts
     config/                           ← Token config, devnet settings
     Anchor.toml
     Cargo.toml (workspace)
     package.json (root — workspace for tests + CLI)
     tsconfig.json (root)
     .gitignore
     README.md
     ```
   - Root `package.json`: workspaces `["tests", "cli"]`, devDependencies: `@coral-xyz/anchor`, `@solana/web3.js`, `@solana/spl-token`, `typescript`, `@types/node`.
   - `cli/package.json`: name `cto-pay`, bin `cto-pay`, dependencies: `@coral-xyz/anchor`, `@solana/web3.js`, `@solana/spl-token`, `commander` (for CLI arg parsing), `bs58`.
   - Pin Bun 1.1+ in `.tool-versions` or `package.json#engines`.
   - Add `.github/` directory stub for workflows (populated by Atlas in Task 12).
   - Configure `programs/cto-pay/Cargo.toml` with: `anchor-lang = "0.30.1"`, `anchor-spl = "0.30.1"`, `solana-program = "1.18"`. Set `rust-version = "1.75"` and edition 2021.
   - Set workspace Cargo.toml `[profile.release]` with `overflow-checks = true` and `lto = "thin"`.
   - Add biome.json for TypeScript linting (consistent with existing CTO quality standards).

2. **Provision SeaweedFS receipt bucket** in the CTO cluster:
   - Create bucket `cto-pay-receipts` in the existing SeaweedFS deployment (namespace `cto`).
   - Use the SeaweedFS S3 API or `weed shell` to create the bucket.
   - Create a K8s Job or init script in `infra/gitops/` that ensures the bucket exists.

3. **Provision operator keypair secret** via the existing secrets pipeline:
   - Create a 1Password item `cto-pay-operator-keypair` containing the Solana keypair JSON (byte array format).
   - Configure OpenBao to sync this item.
   - Create an `ExternalSecret` CR in namespace `cto` that creates K8s Secret `cto-pay-operator-keypair` with key `operator-keypair.json`.
   - The pod spec (controller) will mount this as a volume at `/secrets/operator-keypair.json`.
   - Set `SOLANA_OPERATOR_KEYPAIR_PATH=/secrets/operator-keypair.json` in the ConfigMap.

4. **Create `cto-pay-infra-endpoints` ConfigMap** in namespace `cto`:
   ```yaml
   apiVersion: v1
   kind: ConfigMap
   metadata:
     name: cto-pay-infra-endpoints
     namespace: cto
   data:
     SEAWEEDFS_ENDPOINT: "http://seaweedfs-s3.cto.svc.cluster.local:8333"
     SEAWEEDFS_BUCKET: "cto-pay-receipts"
     SOLANA_RPC_URL: "https://devnet.helius-rpc.com/?api-key=<HELIUS_API_KEY>"
     SOLANA_OPERATOR_KEYPAIR_PATH: "/secrets/operator-keypair.json"
     CTO_PAY_ENABLED: "false"
   ```
   - Store the Helius API key in 1Password and reference via ExternalSecret (do NOT inline in ConfigMap). Use a Secret for the RPC URL if it contains the API key, or use a separate Secret `cto-pay-helius-api-key`.

5. **Generate and commit operator keypair for devnet development**:
   - Run `solana-keygen new -o devnet-operator-keypair.json --no-bip39-passphrase` locally.
   - Fund it on devnet: `solana airdrop 5 --keypair devnet-operator-keypair.json`.
   - Store in 1Password for the pipeline. Add `devnet-operator-keypair.json` to `.gitignore`.

6. **Create devnet setup script** at `scripts/setup-devnet.sh`:
   - Airdrop SOL to operator and test customer wallets.
   - Create custom SPL token mint (6 decimals) — store mint address in `config/devnet.json`.
   - Mint test tokens to customer wallets.
   - This script is called by CI and by developers for local setup.
</implementation_plan>

<acceptance_criteria>
1. Clone the cto-pay repo. Run `anchor build` — compiles without errors (program skeleton).
2. Run `bun install` at root — all dependencies resolve.
3. Verify `cli/package.json` has correct bin entry and dependencies.
4. In the CTO cluster: `kubectl get configmap cto-pay-infra-endpoints -n cto -o yaml` — contains all 5 keys (SEAWEEDFS_ENDPOINT, SEAWEEDFS_BUCKET, SOLANA_RPC_URL, SOLANA_OPERATOR_KEYPAIR_PATH, CTO_PAY_ENABLED).
5. Verify SeaweedFS bucket: `aws s3 ls s3://cto-pay-receipts --endpoint-url $SEAWEEDFS_ENDPOINT` returns empty (no error).
6. Verify operator keypair secret: `kubectl get secret cto-pay-operator-keypair -n cto` exists and contains key `operator-keypair.json`.
7. Run `scripts/setup-devnet.sh` — creates SPL mint, `config/devnet.json` contains mint address (44-character base58 string).

See also: acceptance.md in this task directory for the checklist version.
</acceptance_criteria>

<subtasks>
- Scaffold cto-pay Anchor repo with directory structure, Cargo.toml, and package.json workspaces: Initialize the cto-pay repo using Anchor CLI and configure the full project directory structure, Rust workspace, TypeScript workspaces, tooling configs, and all dependency pinning.
- Provision SeaweedFS receipt bucket cto-pay-receipts: Create the cto-pay-receipts bucket in the existing SeaweedFS deployment in the cto namespace, and create a K8s Job manifest in infra/gitops/ to ensure idempotent bucket creation.
- Provision operator keypair secret via 1Password → OpenBao → ExternalSecret pipeline: Create the 1Password item for the operator keypair, configure OpenBao sync, and deploy the ExternalSecret CR that materializes the K8s Secret cto-pay-operator-keypair in the cto namespace.
- Create cto-pay-infra-endpoints ConfigMap and Helius API key ExternalSecret: Deploy the cto-pay-infra-endpoints ConfigMap with all 5 required keys and a separate ExternalSecret for the Helius API key, ensuring sensitive values are not inlined in the ConfigMap.
- Generate devnet operator keypair, fund on devnet, and store in 1Password: Generate a Solana keypair for devnet operator use, airdrop SOL to it, store it securely in 1Password for the secrets pipeline, and ensure it is gitignored.
- Create devnet setup script (scripts/setup-devnet.sh): Write the devnet bootstrap script that airdrops SOL, creates a custom SPL token mint with 6 decimals, mints test tokens to customer wallets, and writes all addresses to config/devnet.json.
</subtasks>