<identity>
You are tess working on subtask 5003 of task 5.
</identity>

<context>
<scope>
Write settlement.test.ts covering single/multi-settlement, fee computation, daily_spent tracking, daily cap reset via slot warp, and TaskReceipt field verification. Write refund.test.ts covering refund flow and withdrawability of refunded funds.
</scope>
</context>

<implementation_plan>
1. Create `tests/happy-path/settlement.test.ts`:
   - Test 'settle_task decreases customer balance and increases treasury balance': Initialize program, create customer, deposit 100_000_000. Settle task 'TASK-1' for 20_000_000. Assert customer balance === 80_000_000, treasury token balance increased by 20_000_000. Assert TaskReceipt PDA exists.
   - Test 'TaskReceipt fee_amount equals amount * protocol_fee_bps / 10_000': With 500 bps fee, settle 20_000_000. Fetch TaskReceipt, assert fee_amount === 20_000_000 * 500 / 10_000 === 1_000_000.
   - Test 'multiple settlements update task_count and total_spent': Settle 3 tasks (10M, 15M, 20M). Fetch CustomerBalance, assert task_count===3, total_spent===45_000_000.
   - Test 'daily_spent increments correctly across settlements': Settle 2 tasks in same context (no slot warp). Assert daily_spent === sum of both amounts.
   - Test 'daily cap resets after SLOTS_PER_DAY — new settlement succeeds': Settle a task, then call `context.warpToSlot(currentSlot + 216_001)`. Settle another task. Fetch CustomerBalance, assert daily_spent equals only the second settlement amount (reset occurred).
   - Test 'TaskReceipt task_id_hash matches SHA-256 of input': Settle 'TASK-42'. Fetch TaskReceipt, compare task_id_hash field to `computeTaskIdHash('TASK-42')`.
   - Test 'TaskReceipt receipt_hash stores provided hash': Provide a known 32-byte hash during settlement. Fetch TaskReceipt, assert receipt_hash matches.
   - Test 'TaskReceipt status is Settled': Fetch TaskReceipt after settlement, assert status enum === Settled (variant index 0 or matching Anchor enum).

2. Create `tests/happy-path/refund.test.ts`:
   - Test 'refund_task restores customer balance and updates TaskReceipt status': Settle a task for 20_000_000, then refund it. Assert customer balance increased by 20_000_000, TaskReceipt status === Refunded, total_spent decreased by 20_000_000.
   - Test 'refunded funds are withdrawable': Settle 20_000_000, refund, then withdraw the refunded amount. Assert customer balance === 0, customer ATA increased by refund amount.

3. Use Bankrun's `warpToSlot()` for the daily reset test — compute current slot from context and add 216_001.
</implementation_plan>

<validation>
Run `bun test tests/happy-path/settlement.test.ts tests/happy-path/refund.test.ts` — all 10 tests pass (8 settlement + 2 refund). Verify fee computation test checks exact integer arithmetic. Verify slot warp test confirms daily_spent reset to 0 before the new settlement amount is applied.
</validation>