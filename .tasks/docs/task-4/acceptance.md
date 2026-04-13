## Acceptance Criteria for Task 4

1. Run `anchor build` — compiles with zero warnings under pedantic clippy.
2. IDL contains all 9 instructions: initialize_operator, create_customer_account, deposit, withdraw, settle_task, refund_task, update_spending_caps, pause, unpause.
3. Verify `settle_task` IDL args include `task_id_hash` (32-byte array), `amount` (u64), `receipt_hash` (32-byte array).
4. Code review: `settle_task` performs 5 validation checks in order (paused, amount > 0, per-task cap, daily cap with reset, balance).
5. Code review: fee_amount computed as `amount * protocol_fee_bps / 10_000` using checked arithmetic (multiply first).
6. Code review: `refund_task` does NOT check paused state.
7. Code review: `withdraw` (from Task 3) does NOT check paused state.
8. Code review: daily cap reset uses slot comparison with `SLOTS_PER_DAY = 216_000`.
9. Program binary size: `ls -la target/deploy/cto_pay.so` — under 1.4MB.

_Generated from task metadata (LLM fallback)._