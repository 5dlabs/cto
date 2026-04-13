<identity>
You are tess working on subtask 6002 of task 6.
</identity>

<context>
<scope>
Write authorization.test.ts with 7 tests verifying that all operator-gated instructions (settle, refund, pause, unpause) reject non-operator signers and all customer-gated instructions (withdraw, update_caps) reject non-owner signers.
</scope>
</context>

<implementation_plan>
1. Create `tests/edge-cases/authorization.test.ts`. Import helpers and error assertion utility.

2. For each test, create a fresh program context with a proper operator, then attempt the operation with a different (unauthorized) keypair as signer.

3. Test 'non-operator calling settle_task fails':
   - Initialize with operator A. Create customer, deposit. Generate random keypair B.
   - Call settle_task with B as signer instead of A. Assert Anchor constraint error (likely `ConstraintHasOne` or `ConstraintRaw` or a custom 'Unauthorized' error — check the program's error handling).

4. Test 'non-operator calling refund_task fails':
   - Settle a task with operator A. Attempt refund with signer B. Assert authorization error.

5. Test 'non-operator calling pause fails':
   - Attempt pause with signer B. Assert authorization error.

6. Test 'non-operator calling unpause fails':
   - Pause with operator A. Attempt unpause with signer B. Assert authorization error.

7. Test 'non-customer calling withdraw on another customer balance fails':
   - Create customer1. Deposit 100M. Generate customer2 keypair. Attempt withdraw from customer1's balance signed by customer2. Assert error (PDA derivation mismatch or signer check).

8. Test 'non-customer calling update_spending_caps on another account fails':
   - Create customer1. Attempt update_spending_caps on customer1's PDA signed by customer2. Assert error.

9. Test 'settling task against wrong customer balance fails':
   - Create customer1 and customer2. Deposit to both. Attempt settle referencing customer1's PDA but with a mismatched customer pubkey or account. Assert error.

10. Note: Anchor may throw different error types for constraint violations (e.g., `2003` for `ConstraintHasOne`). The tests should identify which Anchor error code is thrown and assert on it specifically. If the program uses custom errors for authorization, assert on those.
</implementation_plan>

<validation>
Run `bun test tests/edge-cases/authorization.test.ts` — all 7 tests pass. Each test must catch a specific Anchor error code or CtoPayError variant, not just 'SendTransactionError'. Verify the unauthorized signer's transaction is fully rejected (no state changes occur).
</validation>