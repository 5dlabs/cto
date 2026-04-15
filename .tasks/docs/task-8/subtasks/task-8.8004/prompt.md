<identity>
You are tess working on subtask 8004 of task 8.
</identity>

<context>
<scope>
Write the agent package registry test suite covering package registration, invalid split_bps validation, duplicate package prevention, author-only updates, unauthorized update rejection, and package deactivation.
</scope>
</context>

<implementation_plan>
1. Create `tests/03-agent-package.test.ts` with describe block 'Agent Package Registry'.
2. Before-all: initialize operator, create funded author wallet.
3. Test: 'registers package' — call register_package with package_id='test-pkg-1', split_bps=3000, source_uri='https://github.com/test'. Assert PDA created with correct fields, task_count=0, success_count=0, total_earned=0, active=true.
4. Test: 'fails to register with split_bps > 10000' — call register_package with split_bps=10001. Assert InvalidSplitBps or similar error.
5. Test: 'fails to register duplicate package_id' — call register_package with same package_id='test-pkg-1'. Assert error (PDA already exists).
6. Test: 'updates package by author' — call update_package with new split_bps=5000, new source_uri. Assert fields updated on-chain.
7. Test: 'fails to update package by non-author' — create different wallet, call update_package signed by non-author. Assert Unauthorized error.
8. Test: 'deactivates package' — call deactivate_package (or update with active=false). Assert active=false on-chain.
</implementation_plan>

<validation>
Run `anchor test --skip-build -- --grep 'Agent Package Registry'` — all 6 tests pass. Package account fields match expected values after each operation. Authorization checks correctly reject non-author signers.
</validation>