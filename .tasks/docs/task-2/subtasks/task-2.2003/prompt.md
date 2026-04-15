<identity>
You are rex working on subtask 2003 of task 2.
</identity>

<context>
<scope>
Implement the slot-based daily spending cap reset logic as a standalone helper function that can be called from deposit, withdraw, and future settle instructions.
</scope>
</context>

<implementation_plan>
1. Create `programs/cto-billing/src/helpers/mod.rs` (or `helpers/spending_caps.rs`). 2. Define constant `SLOTS_PER_DAY: u64 = 216_000` (approximately 1 day at ~400ms per slot). 3. Implement `pub fn maybe_reset_daily_spending(customer: &mut CustomerBalance, current_slot: u64)` that: checks if `current_slot >= customer.daily_reset_slot + SLOTS_PER_DAY`, if so resets `customer.daily_spent = 0` and sets `customer.daily_reset_slot = current_slot`. 4. Implement `pub fn check_spending_caps(customer: &CustomerBalance, amount: u64) -> Result<()>` that: verifies `amount <= customer.max_per_task` (else ExceedsPerTaskCap), verifies `customer.daily_spent + amount <= customer.max_per_day` (else ExceedsDailyCap). 5. Handle overflow checks using checked_add. 6. Export helpers module from lib.rs.
</implementation_plan>

<validation>
Unit tests cover: (1) No reset when within same day (daily_spent preserved). (2) Reset triggers when current_slot exceeds daily_reset_slot + SLOTS_PER_DAY. (3) check_spending_caps passes when amount is within both caps. (4) check_spending_caps fails with ExceedsPerTaskCap. (5) check_spending_caps fails with ExceedsDailyCap. (6) Overflow is handled gracefully.
</validation>