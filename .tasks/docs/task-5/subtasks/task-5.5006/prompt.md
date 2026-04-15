<identity>
You are rex working on subtask 5006 of task 5.
</identity>

<context>
<scope>
Extend the customer profile configuration to include a Solana public key mapping, so the controller can look up a customer's on-chain wallet when processing billing for their CodeRuns.
</scope>
</context>

<implementation_plan>
1. For hackathon simplicity, use namespace annotations as the wallet mapping:
   - Annotation key: `cto.dev/solana-pubkey` — value is the customer's base58-encoded Solana pubkey.
   - Example: `kubectl annotate namespace customer-alice cto.dev/solana-pubkey=ALiCe...`.
2. In `crates/controller/src/tasks/code/billing.rs`, implement:
   `async fn get_customer_wallet(client: &kube::Client, namespace: &str) -> Result<Option<Pubkey>>`
   - Fetch the namespace resource.
   - Read annotation `cto.dev/solana-pubkey`.
   - Parse as `Pubkey` and return `Some(pubkey)`, or `None` if annotation is missing.
   - Return error if annotation exists but is not a valid base58 pubkey.
3. Add a helper `fn has_billing_wallet(coderun: &CodeRun) -> bool` that checks if the CodeRun's namespace has the annotation (for quick filtering in the reconciliation loop).
4. Document the annotation-based approach and note that a future CRD-based CustomerProfile would replace this.
5. If time permits, define a minimal `CustomerProfile` CRD struct with `solana_pubkey: Option<String>` field for future use, but the reconciliation loop should use the annotation approach for now.
</implementation_plan>

<validation>
Unit test: parse a valid base58 Solana pubkey from a mock namespace annotation, verify it returns the correct Pubkey. Test that an invalid base58 string returns a descriptive error. Test that a missing annotation returns None. Integration test: create a test namespace with the annotation set, call get_customer_wallet, verify it returns the correct Pubkey.
</validation>