<identity>
You are tess working on subtask 6001 of task 6.
</identity>

<context>
<scope>
Write spending-caps.test.ts with 7 tests verifying that the program enforces per-task and daily spending caps correctly, including boundary exact-match cases and daily reset after warping past SLOTS_PER_DAY.
</scope>
</context>

<implementation_plan>
1. Create `tests/edge-cases/spending-caps.test.ts`. Import helpers from Task 5's setup.ts/helpers.ts.

2. Add a shared error assertion helper at the top: `async function expectError(fn: () => Promise<any>, errorCode: string)` that catches the Anchor error, parses the `AnchorError` object, and asserts `error.error.errorCode.code === errorCode`. Fail the test if no error is thrown.

3. Test 'settle_task exceeding max_per_task fails with SpendingCapPerTaskExceeded':
   - Create customer with max_per_task=50_000_000, max_per_day=200_000_000. Deposit 100_000_000.
   - Attempt settle for 50_000_001. Assert error code === 'SpendingCapPerTaskExceeded'.

4. Test 'settle_task exceeding daily cap fails with SpendingCapDailyExceeded':
   - Create customer with max_per_task=100_000_000, max_per_day=150_000_000. Deposit 200_000_000.
   - Settle 100_000_000 (succeeds). Attempt settle 60_000_000 (daily_spent would be 160M > 150M). Assert 'SpendingCapDailyExceeded'.

5. Test 'sequential settlements individually under per-task but cumulatively over daily cap':
   - max_per_task=80_000_000, max_per_day=200_000_000. Deposit 300_000_000.
   - Settle 80M (ok, daily=80M), settle 80M (ok, daily=160M), settle 80M (fail, daily would be 240M > 200M). Assert 'SpendingCapDailyExceeded' on third.

6. Test 'after daily cap hit, warping past SLOTS_PER_DAY allows new settlement':
   - Hit daily cap as above. Warp to currentSlot + 216_001. Settle 80M again. Assert succeeds, daily_spent === 80_000_000.

7. Test 'updating caps to lower values does not retroactively fail':
   - Settle 40M under max_per_task=50M. Update caps to max_per_task=30M. Assert no error during update. Next settlement of 30M should succeed even though total_spent > new per-task cap.

8. Test 'settlement exactly at per-task cap boundary succeeds':
   - max_per_task=50_000_000. Settle exactly 50_000_000. Assert succeeds.

9. Test 'settlement exactly at daily cap boundary succeeds':
   - max_per_day=100_000_000. Settle 60M, then 40M. Assert second succeeds (daily_spent === 100M).
</implementation_plan>

<validation>
Run `bun test tests/edge-cases/spending-caps.test.ts` — all 7 tests pass. Verify each failure test asserts the exact CtoPayError variant string. Verify boundary tests (exact cap amounts) pass without error. Verify slot-warp test confirms daily_spent reset.
</validation>