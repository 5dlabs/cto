<identity>
You are tess working on subtask 8005 of task 8.
</identity>

<context>
<scope>
Write the comprehensive settlement test suite covering all three settlement cases (no agent package, quality met with split, quality not met), edge cases for split_bps boundaries, and all error conditions (cap violations, pause state, insufficient balance, duplicate task, inactive package, unauthorized).
</scope>
</context>

<implementation_plan>
1. Create `tests/04-settlement.test.ts` with describe block 'Settlement'.
2. Before-all: initialize operator with protocol_fee_bps=500 (5%), create treasury ATA, create customer with caps (max_per_task=100_000_000, max_per_day=500_000_000), deposit 500 USDC, register agent package with split_bps=3000, create author ATA.
3. Test: 'Case 1 - settles without agent package' — call settle with task_id='task-no-agent', amount=10_000_000, quality_met=true, no agent_package. Assert: customer balance debited by 10 USDC, treasury ATA credited 10 USDC, TaskReceipt PDA created with correct fields, author_earned=0.
4. Test: 'Case 2 - settles with agent package, quality_met=true, split_bps=3000' — settle with amount=100_000_000 (100 USDC). After protocol fee (5 USDC), remaining 95 USDC split: author gets 30% (28.5 USDC), treasury gets 70% (66.5 USDC). Verify exact token amounts in author ATA and treasury ATA. Verify TaskReceipt.author_earned and TaskReceipt.protocol_fee.
5. Test: 'Case 2 edge - split_bps=0' — register new package with split_bps=0, settle. Assert author gets 0, treasury gets 100% after protocol fee.
6. Test: 'Case 2 edge - split_bps=10000' — register new package with split_bps=10000, settle. Assert author gets 100% after protocol fee, treasury gets only protocol fee.
7. Test: 'Case 3 - quality_met=false' — settle with quality_met=false. Assert customer NOT debited (balance unchanged), TaskReceipt.amount=0, author earns 0.
8. Test: 'fails when exceeding per-task cap' — settle with amount exceeding max_per_task. Assert ExceedsTaskCap error.
9. Test: 'fails when exceeding daily cap' — settle multiple tasks to approach daily cap, then settle one that exceeds. Assert ExceedsDailyCap error.
10. Test: 'fails while paused' — pause operator, attempt settle. Assert ProgramPaused. Unpause after.
11. Test: 'fails with insufficient balance' — create new customer with low balance, attempt large settlement. Assert InsufficientBalance.
12. Test: 'fails to settle same task_id twice' — attempt settle with previously used task_id. Assert error (PDA already exists).
13. Test: 'fails with inactive agent package' — deactivate package, attempt settle. Assert InactivePackage error.
14. Test: 'fails when non-operator settles' — create non-operator wallet, attempt settle. Assert Unauthorized.
</implementation_plan>

<validation>
Run `anchor test --skip-build -- --grep 'Settlement'` — all 12 tests pass. For Case 2 with split_bps=3000: verify exact USDC amounts — protocol_fee=5_000_000, author_earned=28_500_000, treasury_received=66_500_000 (or verify the exact arithmetic per the program's implementation). All error cases produce the correct Anchor error codes. TaskReceipt PDAs contain correct data for every settlement.
</validation>