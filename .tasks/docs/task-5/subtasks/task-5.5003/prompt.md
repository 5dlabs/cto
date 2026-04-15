<identity>
You are rex working on subtask 5003 of task 5.
</identity>

<context>
<scope>
Create pure functions compute_billable_amount and build_receipt_json in the billing module that transform CodeRun metadata into a USDC lamport amount and a structured receipt JSON matching the Task 4 schema.
</scope>
</context>

<implementation_plan>
1. In `crates/controller/src/tasks/code/billing.rs`, define:
   - `struct BillableTask` with fields: task_id (String), customer_pubkey (Pubkey), amount_usdc_lamports (u64), receipt_json (serde_json::Value), agent_package_id (String), quality_met (bool).
   - Pricing constants: `FREE_RATE: u64 = 3_000_000` (3 USDC), `TEAM_RATE: u64 = 1_500_000` (1.50), `GROWTH_RATE: u64 = 750_000` (0.75), `ENTERPRISE_RATE: u64 = 500_000` (0.50). For hackathon, a flat `HACKATHON_RATE: u64 = 750_000`.
2. Implement `fn compute_billable_amount(coderun: &CodeRun) -> u64`:
   - Extract pod duration from CodeRun status (start_time to end_time).
   - Look up customer tier from CodeRun annotations or default to HACKATHON_RATE.
   - Calculate: `(duration_minutes.ceil() as u64) * rate_per_run`. For hackathon, simplify to flat per-run rate.
   - Return amount in USDC lamports (1 USDC = 1_000_000 lamports).
3. Implement `fn build_receipt_json(coderun: &CodeRun, amount: u64) -> serde_json::Value`:
   - Build JSON matching Task 4 receipt schema with fields: task_id, customer_pubkey, timestamp (ISO 8601), billing_items (array of {description, quantity, unit_price_usdc, subtotal_usdc}), total_amount_usdc, agent_package_id, quality_met.
   - Billing items should include: coderun duration charge, infra tier charge.
   - Amounts in the JSON should be human-readable decimal strings (e.g., "0.75") while the return struct uses integer lamports.
4. Both functions must be pure (no I/O, no async) for easy unit testing.
</implementation_plan>

<validation>
Run `cargo test -p controller -- billing::compute` and `cargo test -p controller -- billing::receipt` — all pass. Specific assertions: compute_billable_amount returns 750_000 for a Growth-tier single CodeRun; returns 1_500_000 for a Team-tier single CodeRun; returns 0 for a zero-duration run. build_receipt_json output parses as valid JSON, contains task_id field matching input, billing_items array is non-empty, total_amount_usdc matches computed amount as decimal string.
</validation>