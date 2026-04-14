<identity>
You are rex, the Rust/Axum implementation agent. You own task 4 end-to-end.
</identity>

<context>
<task_overview>
Task 4: Controller Settlement Event Publisher & Receipt Storage (Rex - Rust/Axum)
Extend the CTO Kubernetes controller to publish settlement events to a Redis stream when CodeRuns reach terminal states. This is the controller-side half of the decoupled sidecar architecture (D5). All Solana-related code is feature-gated behind `solana-billing`.
Priority: medium
Dependencies: 1, 2
</task_overview>
</context>

<implementation_plan>
1. Add `solana-billing` feature flag to `crates/controller/Cargo.toml`, defaulting to `false`:
   ```toml
   [features]
   solana-billing = ["dep:redis", "dep:serde_json", "dep:sha2"]
   ```
   Add redis crate (deadpool-redis or redis-rs with async) and sha2 as optional dependencies.

2. Define the settlement event schema in `crates/controller/src/billing/event.rs` (new module, gated):
   ```rust
   #[derive(Serialize, Deserialize)]
   pub struct SettlementEvent {
       pub task_id: String,
       pub customer_pubkey: String,   // Solana pubkey from customer profile
       pub amount_usdc_lamports: u64, // Amount in USDC smallest unit
       pub receipt_hash: [u8; 32],    // SHA-256 of receipt JSON
       pub receipt_url: String,       // S3 URL of receipt JSON
       pub agent_package_id: Option<String>, // If a public agent was used
       pub quality_met: bool,
       pub event_type: EventType,     // Settle | Refund
       pub coderun_name: String,
       pub timestamp: i64,
   }
   ```

3. Define receipt JSON schema in `crates/controller/src/billing/receipt.rs`:
   ```rust
   pub struct ReceiptJson {
       pub task_id: String,
       pub customer: String,
       pub coderun_name: String,
       pub duration_seconds: u64,
       pub infra_tier: String,      // standard | high-mem | GPU
       pub provider: String,        // anthropic | openai | etc
       pub agent_used: String,      // rex | blaze | tess | public-package-id
       pub itemized_costs: Vec<CostLineItem>,
       pub total_usdc: f64,
       pub timestamp: String,       // ISO 8601
   }
   ```

4. Implement receipt storage: generate receipt JSON, upload to S3 (using existing S3 client or a simple HTTP PUT to S3-compatible storage), compute SHA-256 hash.

5. Implement the Redis stream publisher in `crates/controller/src/billing/publisher.rs`:
   - Connect to Redis at the configured URL (reuse `gitlab/gitlab-redis-master` — D5).
   - Publish to stream `cto:settlements` using XADD.
   - Include the full SettlementEvent as JSON in the stream entry.
   - Log publication success/failure but **never block or panic** — settlement is fire-and-forget from the controller's perspective.

6. Integrate into the controller reconciliation loop in `crates/controller/src/tasks/code/controller.rs`:
   - After a CodeRun transitions to terminal state (merged/failed/cancelled), behind `#[cfg(feature = "solana-billing")]`:
     a. Compute billable amount from pod duration × tier rate.
     b. Build ReceiptJson, upload to S3, hash it.
     c. Determine if a public agent package was used (from CodeRun spec/annotations).
     d. Determine quality_met from Tess/Cipher/Stitch attestations on the CodeRun.
     e. Publish SettlementEvent to Redis stream.
     f. Annotate the CodeRun status with a `solana-settlement-event-id` field for traceability.
   - If Redis is unreachable, log a warning and continue — controller must never be blocked.

7. Add Solana billing configuration to controller config (`crates/controller/src/tasks/config.rs`):
   ```rust
   #[cfg(feature = "solana-billing")]
   pub struct SolanaBillingConfig {
       pub redis_url: String,
       pub receipt_s3_bucket: String,
       pub receipt_s3_prefix: String,
       pub enabled: bool,
   }
   ```

8. Add `customer_solana_pubkey` field to the customer profile type (or CodeRun annotation) — a simple String field for the hackathon (D9.3 from PRD).
</implementation_plan>

<acceptance_criteria>
Run `cargo build --features solana-billing` in the controller crate — compiles with zero errors. Run `cargo test --features solana-billing` — unit tests pass for: (1) SettlementEvent serialization/deserialization round-trip, (2) ReceiptJson generation with correct SHA-256 hash computation (hash a known receipt, verify against precomputed hash), (3) Redis publisher correctly formats XADD command (mock Redis or use testcontainers with Redis), (4) Billing amount calculation from pod duration (120s) and tier (standard at $0.75/run) produces expected USDC lamport value. Run `cargo build` without the feature flag — compiles cleanly with no Solana-related code included (verify via binary size or grep).

See also: acceptance.md in this task directory for the checklist version.
</acceptance_criteria>

<subtasks>
- Feature flag setup, billing config struct, and customer pubkey field: Add the solana-billing feature flag to the controller Cargo.toml with optional dependencies, create the SolanaBillingConfig struct, and add the customer_solana_pubkey field to the customer profile or CodeRun annotation type.
- Settlement event and receipt JSON schema definitions: Define the SettlementEvent struct with EventType enum in billing/event.rs and the ReceiptJson struct with CostLineItem in billing/receipt.rs, both with full Serialize/Deserialize derives and feature gating.
- Receipt storage: S3 upload and SHA-256 hash computation: Implement receipt JSON generation, SHA-256 hashing, and upload to S3-compatible object storage in billing/receipt.rs, with non-blocking error handling suitable for the controller's reconciliation loop.
- Redis stream publisher: async XADD to cto:settlements: Implement the Redis stream publisher in billing/publisher.rs that publishes SettlementEvent JSON to the cto:settlements stream using XADD, with non-blocking fire-and-forget error handling.
- Controller reconciliation loop integration: billing hook on terminal CodeRun states: Integrate the billing pipeline into the controller's reconciliation loop, triggering receipt generation, S3 upload, and Redis event publication when CodeRuns reach terminal states (merged/failed/cancelled), all behind #[cfg(feature = "solana-billing")].
</subtasks>