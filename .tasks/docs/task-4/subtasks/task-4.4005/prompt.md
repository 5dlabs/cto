<identity>
You are rex working on subtask 4005 of task 4.
</identity>

<context>
<scope>
Integrate the billing pipeline into the controller's reconciliation loop, triggering receipt generation, S3 upload, and Redis event publication when CodeRuns reach terminal states (merged/failed/cancelled), all behind #[cfg(feature = "solana-billing")].
</scope>
</context>

<implementation_plan>
1. Edit `crates/controller/src/tasks/code/controller.rs` (or wherever the CodeRun reconciliation loop handles terminal state transitions).

2. Add a feature-gated billing hook function:
   ```rust
   #[cfg(feature = "solana-billing")]
   async fn handle_billing(
       coderun: &CodeRun,
       config: &SolanaBillingConfig,
       publisher: &Arc<SettlementPublisher>,
   ) -> Option<String> {
       if !config.enabled { return None; }
       // a. Extract pod duration from CodeRun status (start_time to completion_time)
       let duration_seconds = compute_duration(coderun);
       // b. Determine infra tier from CodeRun spec annotations
       let tier = coderun.spec.annotations.get("cto.ai/infra-tier").unwrap_or("standard");
       // c. Compute billable amount
       let amount = compute_billing_amount(duration_seconds, tier);
       // d. Determine agent info
       let agent_used = coderun.spec.annotations.get("cto.ai/agent").cloned();
       let agent_package_id = coderun.spec.annotations.get("cto.ai/agent-package-id").cloned();
       // e. Determine quality_met from attestation annotations
       let quality_met = determine_quality(coderun);
       // f. Get customer pubkey
       let customer_pubkey = coderun.spec.annotations.get("cto.ai/customer-solana-pubkey")?.clone();
       // g. Build receipt
       let receipt = build_receipt(coderun, duration_seconds, tier, &agent_used, amount);
       // h. Upload to S3
       let (receipt_url, receipt_hash) = match upload_receipt(&receipt, config).await {
           Ok(r) => r,
           Err(e) => { tracing::warn!("Receipt upload failed: {e}"); return None; }
       };
       // i. Build and publish event
       let event = SettlementEvent { task_id: coderun.name(), customer_pubkey, amount_usdc_lamports: amount, receipt_hash, receipt_url, agent_package_id, quality_met, event_type: EventType::Settle, coderun_name: coderun.name(), timestamp: chrono::Utc::now().timestamp() };
       match publisher.publish(&event).await {
           Ok(stream_id) => Some(stream_id),
           Err(e) => { tracing::warn!("Redis publish failed: {e}"); None }
       }
   }
   ```

3. Call `handle_billing` after the CodeRun transitions to a terminal state. If it returns `Some(stream_id)`, annotate the CodeRun status with `solana-settlement-event-id: {stream_id}` via a status patch.

4. Implement helper functions:
   - `compute_duration(coderun: &CodeRun) -> u64` — extracts start/completion timestamps from status, returns seconds.
   - `determine_quality(coderun: &CodeRun) -> bool` — checks for attestation annotations (e.g., `cto.ai/tess-passed=true`, `cto.ai/cipher-approved=true`). Returns true if merged with passing attestations, false if failed/cancelled.
   - `build_receipt(...)` — constructs the ReceiptJson with itemized costs.

5. Initialize `SettlementPublisher` in the controller startup, passing it into the reconciliation context. Gate initialization behind the feature flag.

6. Ensure the entire billing path is wrapped in a catch-all: if any billing step fails, log and continue. The controller must never crash or stall due to billing.

7. Write unit tests:
   - Test `compute_duration` with known timestamps → expected seconds.
   - Test `determine_quality` with various annotation combinations.
   - Test `handle_billing` end-to-end with mocked S3 and Redis → verify event fields are correct.
   - Test that with `config.enabled = false`, handle_billing returns None immediately.
</implementation_plan>

<validation>
Run `cargo test --features solana-billing` — tests pass for: (1) compute_duration returns correct seconds for a CodeRun with known start/completion timestamps, (2) determine_quality returns true for merged CodeRun with passing attestation annotations and false for failed CodeRun, (3) handle_billing with enabled=false returns None without calling S3 or Redis, (4) handle_billing end-to-end (mocked S3/Redis) produces a SettlementEvent with correct amount, customer_pubkey, quality_met, and receipt_hash. Run `cargo build` without the feature flag — compiles cleanly, no billing code in binary.
</validation>