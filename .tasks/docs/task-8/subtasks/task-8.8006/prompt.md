<identity>
You are tess working on subtask 8006 of task 8.
</identity>

<context>
<scope>
Write the refund test suite covering successful refund of a settled task, prevention of double refunds, handling of non-existent tasks, and authorization checks.
</scope>
</context>

<implementation_plan>
1. Create `tests/05-refund.test.ts` with describe block 'Refund Flow'.
2. Before-all: initialize operator, create customer, deposit USDC, settle a task (task_id='refund-test-1') to create a TaskReceipt in Settled status. Record customer balance after settlement.
3. Test: 'refunds settled task' — call refund for task_id='refund-test-1'. Assert: TaskReceipt.status=Refunded, customer balance restored by the settlement amount, vault balance increased. Verify the refund amount matches the original TaskReceipt.amount.
4. Test: 'fails to refund already refunded task' — call refund again for task_id='refund-test-1'. Assert error (AlreadyRefunded or similar).
5. Test: 'fails to refund non-existent task' — call refund for task_id='nonexistent-task'. Assert error (AccountNotFound or PDA doesn't exist).
6. Test: 'fails when non-operator tries to refund' — create non-operator wallet, call refund. Assert Unauthorized error.
</implementation_plan>

<validation>
Run `anchor test --skip-build -- --grep 'Refund Flow'` — all 4 tests pass. After successful refund, customer balance equals pre-settlement balance. TaskReceipt status field updated to Refunded. Double-refund and unauthorized attempts produce correct error codes.
</validation>