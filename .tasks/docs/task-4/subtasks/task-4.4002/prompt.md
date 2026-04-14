<identity>
You are rex working on subtask 4002 of task 4.
</identity>

<context>
<scope>
Define the SettlementEvent struct with EventType enum in billing/event.rs and the ReceiptJson struct with CostLineItem in billing/receipt.rs, both with full Serialize/Deserialize derives and feature gating.
</scope>
</context>

<implementation_plan>
1. Create `crates/controller/src/billing/event.rs`:
   ```rust
   use serde::{Serialize, Deserialize};

   #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
   #[serde(rename_all = "snake_case")]
   pub enum EventType {
       Settle,
       Refund,
   }

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct SettlementEvent {
       pub task_id: String,
       pub customer_pubkey: String,
       pub amount_usdc_lamports: u64,
       pub receipt_hash: [u8; 32],
       pub receipt_url: String,
       pub agent_package_id: Option<String>,
       pub quality_met: bool,
       pub event_type: EventType,
       pub coderun_name: String,
       pub timestamp: i64,
   }
   ```

2. Create `crates/controller/src/billing/receipt.rs`:
   ```rust
   use serde::{Serialize, Deserialize};

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct CostLineItem {
       pub description: String,
       pub amount_usdc: f64,
   }

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct ReceiptJson {
       pub task_id: String,
       pub customer: String,
       pub coderun_name: String,
       pub duration_seconds: u64,
       pub infra_tier: String,
       pub provider: String,
       pub agent_used: String,
       pub itemized_costs: Vec<CostLineItem>,
       pub total_usdc: f64,
       pub timestamp: String,  // ISO 8601
   }
   ```
   Add a method `ReceiptJson::to_json_bytes(&self) -> serde_json::Result<Vec<u8>>` for serialization.
   Add a method `ReceiptJson::compute_sha256(&self) -> [u8; 32]` that serializes to canonical JSON then hashes with sha2.

3. Add a helper function to compute billing amount: `pub fn compute_billing_amount(duration_seconds: u64, tier: &str) -> u64` that returns USDC lamports based on tier rates. Use constants for rates (STANDARD_RATE_USDC_LAMPORTS, etc.).

4. Write unit tests in each module:
   - `event.rs`: Serde round-trip test for SettlementEvent (serialize then deserialize, assert equality).
   - `receipt.rs`: Build a known ReceiptJson, serialize, compute SHA-256, assert against precomputed hash. Test `compute_billing_amount` for standard tier (120s → expected lamport value).
</implementation_plan>

<validation>
Run `cargo test --features solana-billing` — tests pass for: (1) SettlementEvent serializes to valid JSON and deserializes back with all fields intact, (2) ReceiptJson SHA-256 computation for a known receipt produces a deterministic, precomputed 32-byte hash, (3) compute_billing_amount(120, "standard") returns expected USDC lamport value (e.g., 750_000 for $0.75). All tests are gated behind #[cfg(test)] and #[cfg(feature = "solana-billing")].
</validation>