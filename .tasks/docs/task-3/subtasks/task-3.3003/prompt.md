<identity>
You are tess working on subtask 3003 of task 3.
</identity>

<context>
<scope>
Implement Mocha describe blocks for Groups 4 and 5 covering deposit, withdraw, and settle_task (without agent package) instructions, including balance verification, cumulative totals, insufficient balance errors, and TaskReceipt field validation.
</scope>
</context>

<implementation_plan>
1. **Group 4: Deposits & Withdrawals** — `describe('Deposits & Withdrawals', ...)`:
   - Setup: Ensure customer account exists (from Group 2 state or create fresh one in `before` hook). Create customer ATA, mint 200 USDC to it.
   - `it('deposits 100 USDC')`: Call deposit with amount=100_000_000. Fetch customer PDA, verify balance=100_000_000, total_deposited=100_000_000. Verify vault ATA received tokens.
   - `it('deposits another 50 USDC')`: Call deposit with amount=50_000_000. Verify balance=150_000_000, total_deposited=150_000_000.
   - `it('withdraws 30 USDC')`: Call withdraw with amount=30_000_000. Verify balance=120_000_000. Verify customer ATA received 30 USDC back.
   - `it('fails withdraw exceeding balance')`: Call withdraw with amount=200_000_000. Assert InsufficientBalance error.
   - `it('withdraws remaining balance to zero')`: Call withdraw with amount=120_000_000. Verify balance=0.

2. **Group 5: Settlement — Case 1 (No Agent Package)** — `describe('Settlement - No Agent Package', ...)`:
   - Setup: Deposit 100 USDC into customer account. Note treasury ATA balance before settlement.
   - `it('settles TASK-001 for 15 USDC without agent package')`: Call settle_task with task_id='TASK-001', amount=15_000_000, agent_package=None, quality_met=true. Fetch customer PDA: balance=85_000_000. Fetch treasury ATA: received 15_000_000. Fetch TaskReceipt PDA by deriveReceiptPda('TASK-001'): verify task_id, amount=15_000_000, customer, author_earned=0, quality_met=true, status=Settled.
   - `it('verifies customer counters after settlement')`: Fetch customer PDA, verify task_count=1, total_spent=15_000_000.

3. All token balance checks should use `getTokenAccountBalance` and compare the `amount` string or parsed integer. Account for potential rent-exempt minimum SOL but USDC balances should be exact.
</implementation_plan>

<validation>
Run `anchor test` and verify Groups 4-5 pass: 5 tests in Group 4 (deposit 100, deposit 50, withdraw 30, withdraw fail, withdraw to zero), 2 tests in Group 5 (settle no-agent, verify counters). Total: 7 tests. Verify all USDC balances are exact to the lamport. Verify TaskReceipt PDA exists and all fields are correct.
</validation>