<identity>
You are tess working on subtask 3005 of task 3.
</identity>

<context>
<scope>
Implement Mocha describe blocks for Groups 9-12 covering pause/unpause circuit breaker behavior, refund lifecycle (settle then refund), idempotency protection (duplicate task_id), and authorization checks (non-operator signers).
</scope>
</context>

<implementation_plan>
1. **Group 9: Circuit Breaker** — `describe('Circuit Breaker', ...)`:
   - Setup: Ensure operator config exists, customer has balance.
   - `it('pauses the program')`: Call pause instruction with operator signer. Fetch operator config, verify paused=true.
   - `it('blocks settlement when paused')`: Attempt settle_task. Assert ProgramPaused error.
   - `it('allows deposit when paused')`: Call deposit. Assert success (per PRD: only settlement/refund blocked). If PRD intent is unclear, add a comment noting this is an assumption pending decision_point resolution.
   - `it('unpauses the program')`: Call unpause with operator signer. Verify paused=false.
   - `it('allows settlement after unpause')`: Call settle_task. Assert success.

2. **Group 10: Refund Flow** — `describe('Refund Flow', ...)`:
   - Setup: Deposit funds, settle a task ('TASK-REFUND-001') to create a receipt.
   - `it('refunds a settled task')`: Call refund_task with task_id='TASK-REFUND-001'. Verify: customer balance restored by the settlement amount. Fetch TaskReceipt: status=Refunded.
   - `it('fails refund on already-refunded task')`: Call refund_task again for 'TASK-REFUND-001'. Assert error (already refunded).

3. **Group 11: Idempotency** — `describe('Idempotency', ...)`:
   - `it('fails settling duplicate task_id')`: Attempt settle_task with a task_id that was already settled (e.g., 'TASK-001' from Group 5). Assert error (PDA already exists / DuplicateTaskId).

4. **Group 12: Authorization** — `describe('Authorization', ...)`:
   - `it('blocks settle_task by non-operator')`: Call settle_task signed by customer keypair instead of operator. Assert Unauthorized error.
   - `it('blocks pause by non-operator')`: Call pause signed by customer keypair. Assert Unauthorized error.
   - `it('blocks unpause by non-operator')`: Call unpause signed by author keypair. Assert Unauthorized error.

5. Ensure all error-path tests use try/catch pattern and verify the specific Anchor error code, not just that an error was thrown.
</implementation_plan>

<validation>
Run `anchor test` and verify Groups 9-12 pass: 5 tests in Group 9 (pause, blocked settle, allowed deposit, unpause, post-unpause settle), 2 tests in Group 10 (refund success, double-refund fail), 1 test in Group 11 (duplicate task_id), 3 tests in Group 12 (non-operator settle, pause, unpause). Total: 11 tests. Verify all error codes match on-chain definitions. Verify refund restores exact balance. Full suite (all 12 groups, 30+ tests) completes in under 60 seconds.
</validation>