<identity>
You are cipher working on subtask 9004 of task 9.
</identity>

<context>
<scope>
Audit every arithmetic operation in the program for overflow/underflow safety. Focus on the BPS split calculation (split_bps * amount / 10000), balance deposit/withdraw/settle/refund arithmetic, daily spending cap checks, and counter increments. Verify checked arithmetic or u128 intermediate values are used consistently.
</scope>
</context>

<implementation_plan>
1. Search the program source for all arithmetic operations: `+`, `-`, `*`, `/`, `%`, `+=`, `-=`.
2. **BPS split calculation**: Locate the `split_bps * amount / 10000` logic in settle_task. Check if it uses:
   - `checked_mul` + `checked_div` (returns Option, must handle None), OR
   - u128 intermediate: `(amount as u128).checked_mul(split_bps as u128).unwrap().checked_div(10000).unwrap() as u64`, OR
   - Anchor's `checked_math!` macro.
   - If it uses raw `*` and `/` on u64, this is a HIGH finding: `u64::MAX * 10000` overflows.
3. **Deposit arithmetic**: `balance.amount += deposit_amount`. Must use `checked_add` or verify Anchor handles this. Overflow at u64::MAX is unlikely but must be safe.
4. **Withdraw arithmetic**: `balance.amount -= withdraw_amount`. Must verify `withdraw_amount <= balance.amount` BEFORE subtraction. Check for underflow.
5. **Settle arithmetic**: `balance.amount -= task_cost`. Same underflow check needed. Also: `treasury_amount = task_cost - author_split` — verify ordering.
6. **Refund arithmetic**: `balance.amount += refund_amount`. Same overflow check as deposit.
7. **Daily spending cap**: `balance.daily_spent + amount`. Must use checked_add. Verify comparison: `daily_spent.checked_add(amount)? <= daily_cap`.
8. **Counter increments**: `task_count += 1`, `success_count += 1`, `total_earned += amount`. All should use checked ops. While overflow is astronomically unlikely, document as INFO if unchecked.
9. For each finding, note: file:line, operation, current implementation (checked/unchecked), severity, recommended fix.
10. If ANY balance-affecting arithmetic is unchecked, classify as HIGH (potential fund loss).
</implementation_plan>

<validation>
Every arithmetic operation in the program is catalogued with file:line reference. BPS split calculation is confirmed to use overflow-safe math (u128 intermediate or checked ops) — if not, a HIGH finding is documented with exact fix. All balance mutations (deposit, withdraw, settle, refund) are confirmed to use checked arithmetic or have explicit bounds checks. Daily spending cap addition is confirmed overflow-safe. A minimum of 6 arithmetic operations are reviewed and documented.
</validation>