<identity>
You are tess working on subtask 3004 of task 3.
</identity>

<context>
<scope>
Implement Mocha describe blocks for Groups 6, 7, and 8 covering settle_task with agent package (quality met and not met), author split verification, AgentPackage counter updates, and spending cap enforcement (per-task and per-day limits).
</scope>
</context>

<implementation_plan>
1. **Group 6: Settlement — Case 2 (Quality Met, Split)** — `describe('Settlement - Quality Met Split', ...)`:
   - Setup: Ensure customer has sufficient balance (deposit 100 USDC). Ensure agent package 'test-agent-v1' is active with split_bps=3000 (or re-register fresh). Note author ATA and treasury ATA balances.
   - `it('settles TASK-002 with agent split, quality_met=true')`: Call settle_task with task_id='TASK-002', amount=20_000_000, agent_package=packagePda, quality_met=true. Verify:
     - Customer balance decreased by 20_000_000.
     - Author ATA received 6_000_000 (30% of 20 USDC).
     - Treasury ATA received 14_000_000 (70% of 20 USDC).
   - `it('verifies AgentPackage counters')`: Fetch AgentPackage PDA: total_earned=6_000_000, task_count=1, success_count=1.
   - `it('verifies TaskReceipt fields')`: Fetch TaskReceipt PDA for 'TASK-002': amount=20_000_000, author_earned=6_000_000, quality_met=true.

2. **Group 7: Settlement — Case 3 (Quality Not Met)** — `describe('Settlement - Quality Not Met', ...)`:
   - Setup: Note customer balance, author ATA balance, treasury ATA balance before.
   - `it('settles TASK-003 with quality_met=false')`: Call settle_task with task_id='TASK-003', agent_package=packagePda, quality_met=false. Verify:
     - Customer balance unchanged (no deduction).
     - Author ATA unchanged.
     - Treasury ATA unchanged.
     - TaskReceipt: amount=0, quality_met=false.
   - `it('verifies AgentPackage counters after quality-not-met')`: Fetch AgentPackage PDA: task_count incremented (now 2), success_count unchanged (still 1).

3. **Group 8: Spending Cap Enforcement** — `describe('Spending Cap Enforcement', ...)`:
   - Setup: Create a fresh customer (second_customer) with known caps. Deposit sufficient funds.
   - `it('enforces per-task cap')`: Set max_per_task=10_000_000 (10 USDC). Attempt settle_task for 15_000_000. Assert ExceedsPerTaskCap error.
   - `it('enforces daily cap')`: Set max_per_day=25_000_000 (25 USDC). Settle for 20_000_000 (succeeds). Attempt settle for 10_000_000. Assert ExceedsDailyCap error (20+10=30 > 25).
   - `it('tests day boundary logic')`: If local validator supports clock manipulation via `bankrun` or `solana-test-validator --warp-slot`, advance time by 86400 seconds and verify daily spending resets. If not feasible, add a comment documenting the limitation and test the day-number computation logic in isolation.
</implementation_plan>

<validation>
Run `anchor test` and verify Groups 6-8 pass: 3 tests in Group 6 (settle with split, package counters, receipt fields), 2 tests in Group 7 (quality-not-met settlement, package counters), 3 tests in Group 8 (per-task cap, daily cap, day boundary). Total: 8 tests. Verify exact USDC lamport amounts for splits (30% author, 70% treasury). Verify error codes ExceedsPerTaskCap and ExceedsDailyCap match on-chain enum.
</validation>