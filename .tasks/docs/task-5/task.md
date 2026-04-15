## Integrate Solana Settlement Hook into CTO Controller (Rex - Rust/Axum)

### Objective
Add the settlement hook to the existing CTO Kubernetes controller so that when a CodeRun reaches a terminal state, the controller computes the billable amount, uploads the receipt to Arweave (calling the receipt-uploader module via subprocess or HTTP), and submits settle_task or refund_task to the Solana program. This bridges the off-chain runtime to the on-chain billing rail per PRD §9.

### Ownership
- Agent: rex
- Stack: Rust/Axum
- Priority: high
- Status: pending
- Dependencies: 2, 3, 4

### Implementation Details
1. Add Solana dependencies to `crates/controller/Cargo.toml`:
   - `solana-sdk 1.18.x`, `solana-client 1.18.x`, `anchor-client 0.30.x` (or use raw RPC with `solana-client`).
   - `sha2` for SHA-256 hashing.
   - `serde_json` (likely already present) for receipt serialization.
2. Extend controller config in `crates/controller/src/tasks/config.rs`:
   - `solana_rpc_url: Option<String>` — Solana RPC endpoint (from ConfigMap).
   - `solana_operator_keypair_path: Option<String>` — path to operator keypair file (from OpenBao secret mount).
   - `solana_program_id: Option<String>` — deployed program ID.
   - `solana_billing_enabled: bool` — feature flag, default false. Allows gradual rollout.
   - `irys_upload_endpoint: Option<String>` — endpoint for receipt upload (either call the Bun module via CLI subprocess or expose it as a small HTTP service).
3. Create new module `crates/controller/src/tasks/code/billing.rs`:
   - `struct BillingConfig` parsed from controller config.
   - `struct BillableTask` { task_id, customer_pubkey, amount_usdc_lamports, receipt_json, agent_package_id, quality_met }.
   - `fn compute_billable_amount(coderun: &CodeRun) -> u64`: Calculate USDC lamports from pod duration × tier rate. Use rates from PRD §5 (Free=$3/run, Team=$1.50, Growth=$0.75, Enterprise=$0.50). For hackathon, use a flat metered rate.
   - `fn build_receipt_json(coderun: &CodeRun, amount: u64) -> serde_json::Value`: Build the itemized receipt matching the schema from Task 4.
   - `async fn upload_receipt(config: &BillingConfig, receipt: &serde_json::Value) -> Result<(String, [u8; 32])>`: Shell out to `bun packages/receipt-uploader/scripts/upload.ts --receipt '{json}'` or call an HTTP endpoint. Returns (arweave_tx_id, sha256_hash).
   - `async fn submit_settlement(config: &BillingConfig, task: &BillableTask, receipt_hash: [u8; 32]) -> Result<Signature>`: Build and send the `settle_task` Solana transaction. Construct instruction data matching the Anchor IDL. Sign with operator keypair. Send via `RpcClient::send_and_confirm_transaction`.
   - `async fn submit_refund(config: &BillingConfig, task_id: &str) -> Result<Signature>`: Build and send `refund_task`.
4. Integrate into the reconciliation loop in `crates/controller/src/tasks/code/controller.rs`:
   - After a CodeRun transitions to terminal state (detect via status change in the reconcile function):
     - If `solana_billing_enabled` and customer has a linked Solana wallet:
       - On success (merged): compute amount, build receipt, upload, settle.
       - On failure (failed/cancelled): submit refund or settle with quality_met=false.
     - Record the Solana transaction signature in the CodeRun status annotations: `cto.dev/solana-tx: <signature>`.
   - Handle errors gracefully: if Solana submission fails, log error and retry (add to requeue). Do not block the main reconciliation loop.
5. Add customer wallet mapping: extend the customer profile CRD or config to include `solana_pubkey: Option<String>`. For hackathon, this can be a simple annotation on the namespace or a ConfigMap entry.
6. Create a standalone test script `crates/controller/src/tasks/code/billing_test.rs` with unit tests for compute_billable_amount and build_receipt_json.

### Subtasks
- [ ] Add Solana, Anchor, and crypto crate dependencies to controller Cargo.toml: Add all required Solana ecosystem crates to the controller's Cargo.toml so that subsequent billing module subtasks can compile. This includes solana-sdk, solana-client, anchor-client (or equivalent), and sha2 for receipt hashing.
- [ ] Extend controller config with Solana billing fields: Add billing-related configuration fields to the controller's config struct so that the billing module can read Solana RPC URL, operator keypair path, program ID, feature flag, and receipt upload endpoint from environment/ConfigMap.
- [ ] Implement billable amount computation and receipt JSON builder: Create pure functions compute_billable_amount and build_receipt_json in the billing module that transform CodeRun metadata into a USDC lamport amount and a structured receipt JSON matching the Task 4 schema.
- [ ] Implement Arweave receipt upload integration: Implement the async upload_receipt function that sends the receipt JSON to Arweave via the Bun receipt-uploader module (subprocess or HTTP call) and returns the Arweave transaction ID and SHA-256 hash.
- [ ] Implement Solana settle_task and refund_task transaction submission: Implement async functions submit_settlement and submit_refund that construct, sign, and send Solana transactions for the settle_task and refund_task instructions to the on-chain cto_billing program.
- [ ] Add customer wallet mapping to CRD or namespace config: Extend the customer profile configuration to include a Solana public key mapping, so the controller can look up a customer's on-chain wallet when processing billing for their CodeRuns.
- [ ] Integrate billing into the controller reconciliation loop: Hook the billing module into the existing CodeRun controller reconcile function so that terminal-state transitions trigger billing computation, receipt upload, and Solana settlement or refund, with proper feature flagging, error handling, and status annotation.
- [ ] Write unit tests for billing module: Create comprehensive unit tests for compute_billable_amount, build_receipt_json, PDA derivation, and instruction serialization in a dedicated test file to ensure correctness before integration testing.