<identity>
You are rex working on subtask 5008 of task 5.
</identity>

<context>
<scope>
Create comprehensive unit tests for compute_billable_amount, build_receipt_json, PDA derivation, and instruction serialization in a dedicated test file to ensure correctness before integration testing.
</scope>
</context>

<implementation_plan>
1. Create `crates/controller/src/tasks/code/billing_test.rs` (or use `#[cfg(test)] mod tests` inside billing.rs).
2. Tests for compute_billable_amount:
   - Growth tier single run → 750_000 lamports.
   - Team tier single run → 1_500_000 lamports.
   - Free tier single run → 3_000_000 lamports.
   - Enterprise tier single run → 500_000 lamports.
   - Hackathon flat rate → 750_000 lamports.
   - Zero-duration CodeRun → 0 lamports (or minimum charge, depending on design).
3. Tests for build_receipt_json:
   - Output is valid JSON.
   - Contains required fields: task_id, customer_pubkey, timestamp, billing_items, total_amount_usdc, agent_package_id.
   - billing_items is a non-empty array.
   - total_amount_usdc matches the amount parameter as a decimal string.
   - Timestamp is valid ISO 8601.
4. Tests for SHA-256 receipt hashing:
   - Known receipt JSON produces expected hash (precompute expected hash offline).
   - Same receipt always produces same hash (determinism).
5. Tests for PDA derivation:
   - Derive operator_config PDA with known operator pubkey, compare against TypeScript-derived address.
   - Derive customer_balance PDA with known customer pubkey.
   - Derive task_receipt PDA with known task_id string.
6. Run all tests with: `cargo test -p controller --features billing -- billing`.
</implementation_plan>

<validation>
All tests in the billing test module pass with `cargo test -p controller --features billing -- billing`. Zero test failures. Coverage includes: all pricing tiers for compute_billable_amount, valid JSON structure for build_receipt_json, deterministic SHA-256 hashing, and correct PDA derivation for at least 3 PDA types. Tests run in under 5 seconds (no network I/O).
</validation>