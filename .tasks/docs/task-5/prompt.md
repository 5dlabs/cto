<identity>
You are tess, the TypeScript, Bankrun, @coral-xyz/anchor, @solana/spl-token, Bun implementation agent. You own task 5 end-to-end.
</identity>

<context>
<task_overview>
Task 5: Anchor Program Tests — Happy-Path Integration Suite (Tess - TypeScript/Bankrun)
Write comprehensive happy-path integration tests covering the full lifecycle of the CTO Pay program using Bankrun for fast local execution. Tests verify the complete deposit → settle → verify → withdraw flow that will be demonstrated in the hackathon video.
Priority: high
Dependencies: 4
</task_overview>
</context>

<implementation_plan>
1. **Test setup** (`tests/setup.ts`):
   - Use `bankrun` (`solana-bankrun`) for fast local program tests — no validator process needed.
   - Create helpers:
     - `createMint()`: Create the custom SPL token with 6 decimals.
     - `createTokenAccount()`: Create associated token accounts.
     - `mintTo()`: Mint test tokens.
     - `initializeProgram()`: Deploy program, call `initialize_operator`, return context.
     - `createCustomer()`: Generate keypair, create token account, mint 1000 tokens, call `create_customer_account`.
   - Helper to compute `task_id_hash`: `SHA256(task_id_string)` using Node.js `crypto` module.
   - Standard test wallets: operator, treasury, customer1, customer2.

2. **Test file structure** (`tests/`):
   ```
   tests/
     setup.ts
     helpers.ts
     happy-path/
       initialize.test.ts
       customer-lifecycle.test.ts
       settlement.test.ts
       refund.test.ts
       full-loop.test.ts
   ```

3. **initialize.test.ts** — OperatorConfig lifecycle:
   - Test: `initialize_operator` with valid params creates OperatorConfig PDA with correct authority, treasury, mint, protocol_fee_bps, paused=false.
   - Test: Vault token account is created and owned by program PDA.
   - Test: OperatorConfig PDA is deterministic (re-derive and compare).

4. **customer-lifecycle.test.ts** — Customer account operations:
   - Test: `create_customer_account` creates CustomerBalance PDA with correct initial values (balance=0, caps set, created_at populated).
   - Test: `deposit` increases customer balance by exact amount; vault balance increases.
   - Test: Multiple deposits accumulate correctly (deposit 50, deposit 30 → balance = 80, total_deposited = 80).
   - Test: `withdraw` decreases customer balance; customer token account increases.
   - Test: Partial withdraw (deposit 100, withdraw 40 → balance = 60).
   - Test: Full withdraw (deposit 100, withdraw 100 → balance = 0).
   - Test: `update_spending_caps` changes caps correctly.

5. **settlement.test.ts** — Core settlement flow:
   - Test: `settle_task` with valid params: customer balance decreases, treasury balance increases, TaskReceipt PDA created with correct fields.
   - Test: Verify `fee_amount` on TaskReceipt equals `amount * protocol_fee_bps / 10_000`.
   - Test: Multiple settlements for same customer (settle 3 tasks, verify task_count = 3, total_spent correct).
   - Test: `daily_spent` increments correctly across multiple settlements within same day.
   - Test: Settlement after daily cap reset — warp slots forward by `SLOTS_PER_DAY`, verify daily_spent resets to 0 and new settlement succeeds.
   - Test: TaskReceipt `task_id_hash` matches SHA-256 of the input task_id string.
   - Test: TaskReceipt `receipt_hash` stores the provided hash correctly.
   - Test: TaskReceipt `status` is `Settled`.

6. **refund.test.ts** — Refund flow:
   - Test: `refund_task` on a settled task: customer balance increases by refund amount, TaskReceipt status changes to Refunded, total_spent decreases.
   - Test: Refunded funds are withdrawable (refund → withdraw succeeds).

7. **full-loop.test.ts** — End-to-end demo flow:
   - Test: Complete sequence matching the demo video:
     1. Initialize operator with treasury and 500bps fee.
     2. Create customer with caps (max_per_task=50_000_000, max_per_day=200_000_000).
     3. Customer deposits 100_000_000 (100 USDC at 6 decimals).
     4. Settle task "TASK-42" for 12_500_000 (12.50 USDC).
     5. Verify: customer balance = 87_500_000, treasury increased by 12_500_000.
     6. Verify: TaskReceipt exists with correct hash and amount.
     7. Customer withdraws 87_500_000.
     8. Verify: customer balance = 0, customer token account = original - 12_500_000.

8. **Use Bankrun's `warp_to_slot()`** for daily cap reset tests — warp forward by 216_001 slots to trigger reset.

9. All test assertions use strict equality. Test names describe the business scenario, not the implementation.
</implementation_plan>

<acceptance_criteria>
Run `anchor test` (or `bun test tests/happy-path/`) — all tests pass. Minimum 15 happy-path test cases covering: 1 initialize_operator, 6 customer lifecycle (create, deposit x2, withdraw partial, withdraw full, update caps), 5 settlement (single settle, multi settle, fee verification, daily cap tracking, daily reset with slot warp), 2 refund (refund + withdraw refunded funds), 1 full end-to-end loop. Test suite completes in under 30 seconds using Bankrun. Zero tests skipped.

See also: acceptance.md in this task directory for the checklist version.
</acceptance_criteria>

<subtasks>
- Test infrastructure — setup.ts and helpers.ts with Bankrun configuration and standard wallets: Create the foundational test infrastructure including Bankrun context initialization, SPL token helpers, program deployment helpers, customer factory, SHA-256 task_id_hash utility, and standard test wallet keypairs that all happy-path test files will import.
- Initialize and customer lifecycle tests — OperatorConfig and CustomerBalance happy paths: Write initialize.test.ts covering OperatorConfig PDA creation/verification and customer-lifecycle.test.ts covering account creation, single/multiple deposits, partial/full withdrawals, and spending cap updates.
- Settlement and refund tests — settle_task flow, fee verification, daily cap reset, and refund lifecycle: Write settlement.test.ts covering single/multi-settlement, fee computation, daily_spent tracking, daily cap reset via slot warp, and TaskReceipt field verification. Write refund.test.ts covering refund flow and withdrawability of refunded funds.
- Full end-to-end loop test — demo video sequence with exact 6-decimal precision amounts: Write full-loop.test.ts that executes the complete initialize → create customer → deposit → settle → verify receipt → withdraw sequence using the exact USDC amounts (6 decimal precision) that will appear in the hackathon demo video.
</subtasks>