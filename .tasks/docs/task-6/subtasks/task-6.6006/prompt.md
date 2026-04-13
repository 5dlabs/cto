<identity>
You are tess working on subtask 6006 of task 6.
</identity>

<context>
<scope>
Write validation.test.ts with 5 tests verifying that initialize_operator rejects invalid fee bps, deposit rejects wrong mint, create_customer_account validates cap relationships, and withdraw/settle reject amounts exceeding balance.
</scope>
</context>

<implementation_plan>
1. Create `tests/edge-cases/validation.test.ts`. Import helpers and error assertion utility.

2. Test 'initialize_operator with protocol_fee_bps > 10000 fails with InvalidFeeBps':
   - Attempt to call initialize_operator with protocol_fee_bps = 10_001. Assert error code === 'InvalidFeeBps'. Also test with protocol_fee_bps = 65_535 (max u16) to ensure boundary is enforced.

3. Test 'deposit with wrong mint token account fails':
   - Initialize program with mint A. Create a different mint B. Create customer with an ATA for mint B. Attempt deposit providing the mint B token account. Assert Anchor constraint error (likely `ConstraintTokenMint` or custom 'InvalidMint').

4. Test 'create_customer_account with max_per_task > max_per_day fails':
   - Attempt create_customer_account with max_per_task = 200_000_000 and max_per_day = 100_000_000. If the program validates this, assert the error. If the program does NOT validate this (some designs allow it), document this test as asserting the observed behavior. Use a try/catch and log whether it passes or fails to determine program behavior.
   - NOTE: This depends on Task 4 implementation. If the program doesn't validate this, change the test to verify the account is created (documenting the behavior) or skip with a comment.

5. Test 'withdraw more than balance fails with InsufficientBalance':
   - Create customer, deposit 50_000_000. Attempt withdraw 50_000_001. Assert error code === 'InsufficientBalance'.

6. Test 'settle more than customer balance fails with InsufficientBalance':
   - Create customer, deposit 50_000_000. Attempt settle for 50_000_001. Assert error code === 'InsufficientBalance'.

7. All error assertions must check the specific error code string, not just that the transaction failed. Use the pattern:
   ```typescript
   try {
     await program.methods.instruction().accounts({...}).signers([...]).rpc();
     throw new Error('Expected error but succeeded');
   } catch (e) {
     expect(e.error.errorCode.code).toBe('ExpectedErrorCode');
   }
   ```
</implementation_plan>

<validation>
Run `bun test tests/edge-cases/validation.test.ts` — all 5 tests pass. Verify 'InvalidFeeBps' is asserted for fee > 10000. Verify wrong mint test catches constraint error. Verify insufficient balance tests assert 'InsufficientBalance' for both withdraw and settle. Document the result of the max_per_task > max_per_day test regardless of outcome.
</validation>