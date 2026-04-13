<identity>
You are tess working on subtask 6004 of task 6.
</identity>

<context>
<scope>
Write overflow-edge.test.ts with 7 tests covering u64::MAX deposits, near-overflow total_spent, large-amount fee computation, and zero-amount rejection for deposit/settle/withdraw.
</scope>
</context>

<implementation_plan>
1. Create `tests/edge-cases/overflow-edge.test.ts`. Import helpers and error assertion utility.

2. Define `U64_MAX = BigInt('18446744073709551615')` (2^64 - 1) for boundary testing.

3. Test 'deposit of u64::MAX fails gracefully with checked arithmetic':
   - Create customer. Attempt deposit of U64_MAX. The SPL token transfer or the program's checked_add should fail. Assert an error is thrown (either Anchor arithmetic overflow or SPL insufficient funds — the customer won't have this many tokens). Document whichever error occurs.

4. Test 'settle with amount that would overflow total_spent fails':
   - This requires pre-seeding near-max values. Strategy: Use Bankrun's `setAccount()` to directly write a CustomerBalance account with total_spent = U64_MAX - 1_000_000. Then attempt settle for 2_000_000. Assert checked arithmetic overflow error.
   - Alternative if direct account manipulation is complex: Just attempt a very large settlement and verify the error.

5. Test 'daily spending near u64::MAX boundary':
   - Similar to above: pre-seed daily_spent near max, attempt settlement that would overflow. Assert error.

6. Test 'fee computation with large amount does not overflow':
   - The computation is `amount * protocol_fee_bps / 10_000`. If amount = 10^18 and bps = 10_000, intermediate value = 10^22 which overflows u64. Create setup with large amount. Assert the program handles this (either via u128 intermediate or by rejecting).
   - If using Bankrun's setAccount to pre-seed a large balance, attempt settle for 10^18. Verify no silent overflow (fee should be correct or error thrown).

7. Test 'zero amount deposit fails with ZeroAmount':
   - Create customer. Attempt deposit of 0. Assert error code === 'ZeroAmount'.

8. Test 'zero amount settlement fails with ZeroAmount':
   - Create customer, deposit 100M. Attempt settle for 0. Assert 'ZeroAmount'.

9. Test 'zero amount withdrawal fails with ZeroAmount':
   - Create customer, deposit 100M. Attempt withdraw of 0. Assert 'ZeroAmount'.

10. For the pre-seeded account tests: Use Bankrun's `context.setAccount(pda, accountInfo)` to write a CustomerBalance with near-max values. Construct the account data buffer matching the Anchor account discriminator + serialized struct layout.
</implementation_plan>

<validation>
Run `bun test tests/edge-cases/overflow-edge.test.ts` — all 7 tests pass. Verify zero-amount tests assert exactly 'ZeroAmount'. Verify overflow tests confirm the program uses checked arithmetic (no silent wrapping). If pre-seeded account tests are used, verify the account layout matches the program's expected discriminator and serialization.
</validation>