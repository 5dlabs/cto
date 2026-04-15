<identity>
You are tess working on subtask 8003 of task 8.
</identity>

<context>
<scope>
Write the customer account test suite covering account creation, USDC deposits, zero-amount validation, withdrawals, overdraw prevention, pause-state checking, spending cap updates, and cap validation logic.
</scope>
</context>

<implementation_plan>
1. Create `tests/02-customer.test.ts` with describe block 'Customer Account & Escrow'.
2. Before-all: initialize operator (reuse or create fresh), create funded customer wallet, mint 1000 test USDC to customer.
3. Test: 'creates customer account' — call create_customer with max_per_task=100, max_per_day=500. Assert PDA exists with correct caps, balance=0, task_count=0.
4. Test: 'deposits 100 USDC' — call deposit with amount=100_000_000 (100 USDC in lamports). Assert customer balance=100_000_000, vault token account balance=100_000_000.
5. Test: 'fails to deposit 0' — call deposit with amount=0. Assert InvalidAmount error.
6. Test: 'withdraws 50 USDC' — call withdraw with amount=50_000_000. Assert customer balance=50_000_000, customer ATA increased by 50 USDC.
7. Test: 'fails to withdraw more than balance' — call withdraw with amount=999_000_000. Assert InsufficientBalance error.
8. Test: 'fails to withdraw while paused' — pause operator, attempt withdraw. Assert ProgramPaused error. Unpause after.
9. Test: 'updates spending caps' — call update_caps with max_per_task=200, max_per_day=1000. Assert caps updated on-chain.
10. Test: 'fails to update caps when max_per_task > max_per_day' — call update_caps with max_per_task=1000, max_per_day=500. Assert InvalidCaps error.
</implementation_plan>

<validation>
Run `anchor test --skip-build -- --grep 'Customer Account'` — all 8 tests pass. Token balances (customer ATA, vault) match expected values after deposits and withdrawals. Error codes match the program's defined error enum.
</validation>