<identity>
You are rex working on subtask 2006 of task 2.
</identity>

<context>
<scope>
Write and execute all 8 test cases covering settle_task (3 cases, caps, pause, duplicate) and refund_task as specified in the test strategy.
</scope>
</context>

<implementation_plan>
1. Create or extend test file `tests/settlement.ts`.
2. Setup: Reuse program initialization from Task 1 tests. Create operator, customer, agent author wallets. Mint mock USDC. Create ATAs for treasury, customer, author. Initialize operator config. Create customer account with caps.
3. Test 1 — Settle no agent package: Deposit 100 USDC. Call settle_task(task_id='T-001', amount=10_000_000, receipt_hash, quality_met=true) with no agent package. Assert customer balance == 90_000_000, treasury_ata balance += 10_000_000, TaskReceipt.amount == 10_000_000, .author_earned == 0.
4. Test 2 — Settle with agent package, quality_met=true: Register agent package split_bps=3000. Deposit 100 USDC (fresh customer or use remaining balance). Settle task_id='T-002' for 10_000_000 with agent package. Assert author_ata += 3_000_000, treasury += 7_000_000, customer balance -= 10_000_000, AgentPackage.total_earned == 3_000_000, .success_count == 1.
5. Test 3 — Settle with agent package, quality_met=false: Settle task_id='T-003' with quality_met=false. Assert customer balance unchanged, no transfers, TaskReceipt.amount == 0, AgentPackage.task_count incremented, success_count unchanged.
6. Test 4 — Refund: Use TaskReceipt from Test 1. Call refund_task. Assert customer balance += 10_000_000 (restored), treasury -= 10_000_000, TaskReceipt.status == Refunded.
7. Test 5 — ExceedsPerTaskCap: Set max_per_task=5_000_000. Attempt settle for 10_000_000. Expect ExceedsPerTaskCap error.
8. Test 6 — ExceedsDailyCap: Set max_per_day=15_000_000. Settle 10_000_000 successfully. Attempt second settle for 10_000_000. Expect ExceedsDailyCap error.
9. Test 7 — ProgramPaused: Call pause(). Attempt settle_task. Expect ProgramPaused error. Call unpause() for cleanup.
10. Test 8 — Duplicate task_id: Settle task_id='T-DUP'. Attempt second settle with same task_id='T-DUP'. Expect PDA-already-exists error.
11. Run all 8 tests with `anchor test`.
</implementation_plan>

<validation>
Execute `anchor test` — all 8 tests pass with green checkmarks. Total execution time < 30 seconds on local validator. Each test verifies on-chain account state after the instruction, not just transaction success. Error cases verify the specific Anchor error code returned.
</validation>