<identity>
You are tess working on subtask 6005 of task 6.
</identity>

<context>
<scope>
Write double-operations.test.ts with 4 tests verifying that creating the same customer twice, settling the same task_id twice, refunding an already-refunded task, and refunding a non-existent task all fail with appropriate errors.
</scope>
</context>

<implementation_plan>
1. Create `tests/edge-cases/double-operations.test.ts`. Import helpers and error assertion utility.

2. Test 'creating same customer account twice fails':
   - Create customer1 via `create_customer_account`. Attempt to call `create_customer_account` again with the same customer pubkey and operator config (same PDA seeds). Assert Anchor's `init` constraint error — the account already exists. Expected error: Anchor error code `0` (AccountAlreadyInUse) or similar init constraint violation.

3. Test 'settling same task_id_hash twice fails':
   - Create customer, deposit. Settle task 'TASK-1' (creates TaskReceipt PDA). Attempt to settle 'TASK-1' again. The TaskReceipt PDA already exists, so Anchor's `init` constraint fails. Assert the init constraint error.

4. Test 'refunding an already-refunded task fails with TaskNotSettled':
   - Settle task 'TASK-1'. Refund it (succeeds, status = Refunded). Attempt refund again on the same TaskReceipt. Assert error code === 'TaskNotSettled' (since the program should check status === Settled before allowing refund).

5. Test 'refunding a non-existent task fails':
   - Compute a TaskReceipt PDA for a task_id_hash that was never settled. Attempt to call refund_task with this PDA. Assert error — the account doesn't exist, so Anchor's account deserialization should fail (AccountNotInitialized or similar).

6. For each test, verify that no state changes occurred after the failed operation (customer balance unchanged, vault balance unchanged).
</implementation_plan>

<validation>
Run `bun test tests/edge-cases/double-operations.test.ts` — all 4 tests pass. Verify duplicate creation test catches Anchor init constraint error. Verify duplicate settlement catches init constraint error. Verify double refund asserts 'TaskNotSettled'. Verify non-existent refund asserts account validation error.
</validation>