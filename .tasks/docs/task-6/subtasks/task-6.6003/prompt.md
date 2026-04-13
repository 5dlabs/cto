<identity>
You are tess working on subtask 6003 of task 6.
</identity>

<context>
<scope>
Write pause-behavior.test.ts with 7 tests verifying that when the program is paused, create_customer_account/deposit/settle are blocked with ProgramPaused, while withdraw and refund_task succeed as safety valves, and unpause restores normal operation.
</scope>
</context>

<implementation_plan>
1. Create `tests/edge-cases/pause-behavior.test.ts`. Import helpers and error assertion utility.

2. Common setup for most tests: Initialize program, create a customer, deposit tokens, settle a task (for refund test), THEN call `pause` as operator.

3. Test 'create_customer_account fails when paused':
   - Pause the program. Attempt to create a new customer. Assert error code === 'ProgramPaused'.

4. Test 'deposit fails when paused':
   - Create customer before pausing. Pause. Attempt deposit. Assert 'ProgramPaused'.

5. Test 'settle_task fails when paused':
   - Create customer, deposit before pausing. Pause. Attempt settle. Assert 'ProgramPaused'.

6. Test 'withdraw SUCCEEDS when paused (safety valve)':
   - Create customer, deposit 100M before pausing. Pause. Call withdraw for 50M. Assert SUCCEEDS — customer balance decreases, customer ATA increases. No error thrown.

7. Test 'refund_task SUCCEEDS when paused (safety valve)':
   - Create customer, deposit, settle a task before pausing. Pause. Call refund on the settled task. Assert SUCCEEDS — customer balance increases, TaskReceipt status changes to Refunded.

8. Test 'after unpause, all operations resume normally':
   - Pause the program. Unpause. Attempt create_customer_account, deposit, settle_task. Assert all succeed without error.

9. Test 'only operator can pause and unpause':
   - Generate a non-operator keypair. Attempt pause with non-operator — assert authorization error. Pause with operator (succeeds). Attempt unpause with non-operator — assert authorization error.

10. The asymmetric pause behavior (withdraw/refund allowed) is a critical security property — ensure test names clearly document this as intentional safety-valve design.
</implementation_plan>

<validation>
Run `bun test tests/edge-cases/pause-behavior.test.ts` — all 7 tests pass. Critically verify that tests 4 and 5 (withdraw and refund while paused) assert SUCCESS, not failure. Verify the unpause test confirms state transitions back to operational. Verify all blocked operations assert exactly 'ProgramPaused'.
</validation>