<identity>
You are rex working on subtask 5002 of task 5.
</identity>

<context>
<scope>
Add billing-related configuration fields to the controller's config struct so that the billing module can read Solana RPC URL, operator keypair path, program ID, feature flag, and receipt upload endpoint from environment/ConfigMap.
</scope>
</context>

<implementation_plan>
1. Open `crates/controller/src/tasks/config.rs`.
2. Add the following fields to the existing controller config struct (gated behind `#[cfg(feature = "billing")]` if using feature flag):
   - `solana_rpc_url: Option<String>` — e.g., `https://api.devnet.solana.com`. Populated from `SOLANA_RPC_URL` env var.
   - `solana_operator_keypair_path: Option<String>` — filesystem path to the operator's Solana keypair JSON. Populated from `SOLANA_OPERATOR_KEYPAIR_PATH` env var (mounted from OpenBao secret).
   - `solana_program_id: Option<String>` — the deployed cto_billing program ID. From `SOLANA_PROGRAM_ID` env var.
   - `solana_billing_enabled: bool` — feature toggle, default `false`. From `SOLANA_BILLING_ENABLED` env var.
   - `irys_upload_endpoint: Option<String>` — URL or command for receipt upload. From `IRYS_UPLOAD_ENDPOINT` env var.
3. Update the config parsing/loading function to read these from environment variables.
4. Add validation: if `solana_billing_enabled` is true, assert that `solana_rpc_url`, `solana_operator_keypair_path`, and `solana_program_id` are all `Some`, otherwise log a warning and disable billing.
5. Create `struct BillingConfig` in a new file `crates/controller/src/tasks/code/billing.rs` that extracts and holds these fields in a typed, validated form, including a parsed `Pubkey` for program_id and a loaded `Keypair` for the operator.
</implementation_plan>

<validation>
Run `cargo test -p controller -- config` — config parsing tests pass. Test that setting `SOLANA_BILLING_ENABLED=true` without `SOLANA_RPC_URL` logs a warning and disables billing. Test that all fields parse correctly from mock environment variables. BillingConfig::new returns Ok with valid inputs and descriptive Err with missing fields.
</validation>