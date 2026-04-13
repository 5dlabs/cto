<identity>
You are tess working on subtask 5004 of task 5.
</identity>

<context>
<scope>
Write full-loop.test.ts that executes the complete initialize → create customer → deposit → settle → verify receipt → withdraw sequence using the exact USDC amounts (6 decimal precision) that will appear in the hackathon demo video.
</scope>
</context>

<implementation_plan>
1. Create `tests/happy-path/full-loop.test.ts`:
   - Single test (or tightly coupled describe block): 'complete demo flow — init, deposit, settle, verify, withdraw'
   - Step 1: Initialize operator with treasury keypair and protocol_fee_bps = 500 (5%).
   - Step 2: Create customer with max_per_task = 50_000_000 (50 USDC), max_per_day = 200_000_000 (200 USDC).
   - Step 3: Customer deposits 100_000_000 (100.000000 USDC). Assert customer balance === 100_000_000. Assert vault balance increased by 100_000_000.
   - Step 4: Settle task 'TASK-42' for 12_500_000 (12.500000 USDC). Record pre-settle treasury balance.
   - Step 5: Assert customer balance === 87_500_000 (100M - 12.5M). Assert treasury balance increased by exactly 12_500_000.
   - Step 6: Derive TaskReceipt PDA for 'TASK-42'. Fetch it. Assert: amount === 12_500_000, fee_amount === 625_000 (12.5M * 500 / 10000), task_id_hash matches computeTaskIdHash('TASK-42'), status === Settled.
   - Step 7: Customer withdraws 87_500_000 (remaining balance). Assert customer balance === 0.
   - Step 8: Verify customer's token account final balance === (initial minted tokens - 12_500_000). This confirms the only tokens consumed were the settled task amount.

2. Use descriptive assertion messages: e.g., `expect(balance).toBe(87_500_000, 'customer balance after settling 12.50 USDC task')`.
3. This test serves as a regression gate — if this passes, the demo flow is guaranteed to work.
4. Ensure all amounts use integer arithmetic (no floating point) to match on-chain u64 representation.
</implementation_plan>

<validation>
Run `bun test tests/happy-path/full-loop.test.ts` — the single end-to-end test passes. Verify all 8 assertion checkpoints succeed with exact integer values. Confirm fee_amount === 625_000 (not 625_001 or 624_999). Confirm final customer ATA balance equals initial_mint_amount minus 12_500_000.
</validation>