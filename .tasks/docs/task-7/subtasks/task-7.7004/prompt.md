<identity>
You are cipher working on subtask 7004 of task 7.
</identity>

<context>
<scope>
Systematically review the Anchor program for numerical and state safety vulnerabilities: arithmetic overflow/underflow, multiply-before-divide ordering, PDA seed collision potential, rent exemption on all initialized accounts, and reentrancy via CPI ordering (state updates must precede cross-program invocations).
</scope>
</context>

<implementation_plan>
1. **Arithmetic safety**: (a) For every arithmetic operation on u64/u128 values, verify checked_add/checked_sub/checked_mul/checked_div is used (or Anchor's require! with overflow-safe comparison). (b) Check multiply-before-divide ordering to prevent precision loss. (c) Verify no truncation on casts (e.g., u64 as u32). (d) Cross-reference with automated grep findings from subtask 7002 if available. 2. **PDA collision analysis**: (a) For each PDA (task_receipt, customer_balance, etc.), document the full seed array. (b) Assess seed entropy: are seeds unique per entity? Could an attacker craft colliding seeds? (c) Verify bump seeds are stored and reused (not re-derived) to prevent bump manipulation. 3. **Rent exemption**: (a) Verify all `init` instructions use `space` calculations that meet minimum rent-exempt thresholds. (b) Confirm `payer` and `system_program` are properly constrained. (c) Check for any account that could fall below rent exemption after mutation. 4. **Reentrancy / CPI ordering**: (a) For every CPI call (token transfers, system program invocations), verify all program state updates occur BEFORE the CPI. (b) Check that no state reads after CPI depend on pre-CPI assumptions. (c) Document the CPI call graph. 5. Document each finding with file:line reference, vulnerability class, and preliminary severity.
</implementation_plan>

<validation>
Every arithmetic operation on token amounts/balances is catalogued with checked/unchecked status. PDA seed analysis table exists covering all PDAs with seed composition and collision risk assessment. All init instructions verified for rent-exempt space. CPI call graph documented with state-update ordering verified for each cross-program invocation.
</validation>