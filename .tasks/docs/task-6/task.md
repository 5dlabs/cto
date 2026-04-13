# Task 6: Anchor Program Tests — Edge Cases, Error Paths, and Security Scenarios (Tess - TypeScript/Bankrun)

## Overview
Write negative test cases covering all error paths, spending cap enforcement, unauthorized access attempts, paused state behavior, overflow scenarios, and double-operation guards. These tests validate the program's security properties.

## Implementation Details
1. **Test file structure**:
   ```
   tests/
     edge-cases/
       spending-caps.test.ts
       authorization.test.ts
       pause-behavior.test.ts
       overflow-edge.test.ts
       double-operations.test.ts
       validation.test.ts
   ```

2. **spending-caps.test.ts** — Spending cap enforcement (DF-3 from PRD):
   - Test: `settle_task` with amount > `max_per_task` fails with `SpendingCapPerTaskExceeded`.
   - Test: `settle_task` where cumulative `daily_spent + amount > max_per_day` fails with `SpendingCapDailyExceeded`.
   - Test: Sequential settlements that individually pass per-task cap but cumulatively exceed daily cap (settle 3x 80 USDC with daily cap 200 → third fails).
   - Test: After daily cap is hit, warping past `SLOTS_PER_DAY` slots allows new settlement.
   - Test: Updating caps to lower values doesn't retroactively fail (caps apply to future settlements only).
   - Test: Settlement exactly at cap boundary succeeds (amount == max_per_task).
   - Test: Settlement exactly at daily boundary succeeds (daily_spent + amount == max_per_day).

3. **authorization.test.ts** — Unauthorized access:
   - Test: Non-operator calling `settle_task` fails (wrong signer).
   - Test: Non-operator calling `refund_task` fails.
   - Test: Non-operator calling `pause` fails.
   - Test: Non-operator calling `unpause` fails.
   - Test: Non-customer calling `withdraw` on another customer's balance fails (wrong PDA derivation or signer).
   - Test: Non-customer calling `update_spending_caps` on another customer's account fails.
   - Test: Settling a task against wrong customer's balance (mismatched customer pubkey).

4. **pause-behavior.test.ts** — Circuit breaker:
   - Test: When paused, `create_customer_account` fails with `ProgramPaused`.
   - Test: When paused, `deposit` fails with `ProgramPaused`.
   - Test: When paused, `settle_task` fails with `ProgramPaused`.
   - Test: When paused, `withdraw` SUCCEEDS (safety valve).
   - Test: When paused, `refund_task` SUCCEEDS (returns funds to customers).
   - Test: After `unpause`, all operations resume normally.
   - Test: Only operator can pause/unpause.

5. **overflow-edge.test.ts** — Arithmetic edge cases:
   - Test: Deposit of `u64::MAX` (should succeed or fail gracefully — checked arithmetic).
   - Test: Settle with amount that would overflow `total_spent` (pre-seed with near-max values).
   - Test: Daily spending with values near `u64::MAX` boundary.
   - Test: `protocol_fee_bps` computation doesn't overflow: large amount (e.g., 10^18) * 10000 bps.
   - Test: Zero amount deposit fails with `ZeroAmount`.
   - Test: Zero amount settlement fails with `ZeroAmount`.
   - Test: Zero amount withdrawal fails with `ZeroAmount`.

6. **double-operations.test.ts** — Idempotency and state guards:
   - Test: Creating same customer account twice fails (Anchor init constraint).
   - Test: Settling same `task_id_hash` twice fails (PDA already exists — Anchor init constraint).
   - Test: Refunding an already-refunded task fails with `TaskNotSettled`.
   - Test: Refunding a task that was never settled (doesn't exist — should fail on account validation).

7. **validation.test.ts** — Input validation:
   - Test: `initialize_operator` with `protocol_fee_bps > 10000` fails with `InvalidFeeBps`.
   - Test: Deposit with wrong mint token account fails with `InvalidMint`.
   - Test: `create_customer_account` with `max_per_task > max_per_day` fails (if validated).
   - Test: Withdraw more than balance fails with `InsufficientBalance`.
   - Test: Settle more than balance fails with `InsufficientBalance`.

8. Each test must assert the specific error code, not just "transaction failed". Use Anchor's `AnchorError` parsing to validate error code matches expected `CtoPayError` variant.

## Dependencies
Tasks: 4

## Subtasks
- **Spending cap enforcement tests — per-task limits, daily limits, boundary conditions, and cap reset via slot warp**: Write spending-caps.test.ts with 7 tests verifying that the program enforces per-task and daily spending caps correctly, including boundary exact-match cases and daily reset after warping past SLOTS_PER_DAY.
- **Authorization failure tests — wrong signer rejection for all operator and customer instructions**: Write authorization.test.ts with 7 tests verifying that all operator-gated instructions (settle, refund, pause, unpause) reject non-operator signers and all customer-gated instructions (withdraw, update_caps) reject non-owner signers.
- **Pause behavior tests — circuit breaker blocking and safety valve pass-through verification**: Write pause-behavior.test.ts with 7 tests verifying that when the program is paused, create_customer_account/deposit/settle are blocked with ProgramPaused, while withdraw and refund_task succeed as safety valves, and unpause restores normal operation.
- **Overflow and arithmetic edge case tests — u64 boundary, fee computation overflow, and zero amount rejection**: Write overflow-edge.test.ts with 7 tests covering u64::MAX deposits, near-overflow total_spent, large-amount fee computation, and zero-amount rejection for deposit/settle/withdraw.
- **Double-operation guard tests — duplicate account creation, duplicate settlement, and double refund prevention**: Write double-operations.test.ts with 4 tests verifying that creating the same customer twice, settling the same task_id twice, refunding an already-refunded task, and refunding a non-existent task all fail with appropriate errors.
- **Input validation tests — invalid fee bps, wrong mint, cap constraints, and insufficient balance**: Write validation.test.ts with 5 tests verifying that initialize_operator rejects invalid fee bps, deposit rejects wrong mint, create_customer_account validates cap relationships, and withdraw/settle reject amounts exceeding balance.