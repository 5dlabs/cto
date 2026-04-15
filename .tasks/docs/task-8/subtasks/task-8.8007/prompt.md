<identity>
You are tess working on subtask 8007 of task 8.
</identity>

<context>
<scope>
Write the daily spending cap reset test suite that simulates time advancement past the daily boundary to verify the daily_spent counter resets, allowing new settlements to proceed.
</scope>
</context>

<implementation_plan>
1. Create `tests/06-daily-reset.test.ts` with describe block 'Daily Spending Cap Reset'.
2. Before-all: initialize operator, create customer with max_per_day=100_000_000 (100 USDC), deposit 500 USDC.
3. Test: 'resets daily_spent after day boundary' — 
   a. Settle tasks totaling near the daily cap (e.g., 90 USDC out of 100 USDC daily cap). Assert daily_spent is ~90 USDC.
   b. Attempt to settle 20 USDC — should fail with ExceedsDailyCap.
   c. Advance the validator clock past SLOTS_PER_DAY. Use `connection.requestAirdrop` or `BanksClient.warpToSlot` (if using bankrun) or manipulate the slot via localnet clock advancement.
   d. Attempt to settle 20 USDC again — should succeed because daily_spent resets.
   e. Assert new daily_spent = 20 USDC and customer balance debited correctly.
4. Test: 'daily_spent accumulates within same day' — settle two tasks in quick succession, assert daily_spent = sum of both amounts. This confirms accumulation logic within a day period.
5. Note: slot advancement on localnet may require `solana-test-validator --slots-per-epoch` configuration or using the warp_to_slot approach. Document the chosen approach in test comments.
</implementation_plan>

<validation>
Run `anchor test --skip-build -- --grep 'Daily Spending Cap Reset'` — both tests pass. The first test demonstrates the full cycle: accumulate near cap → fail at cap → advance time → succeed after reset. Slot advancement mechanism works reliably on localnet. Daily_spent values match expected amounts at each step.
</validation>