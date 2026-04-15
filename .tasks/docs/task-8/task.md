## Comprehensive Integration & E2E Test Suite (Tess - Test Frameworks)

### Objective
Build a comprehensive test suite that validates the entire on-chain billing system end-to-end: Anchor program integration tests covering all instruction paths, edge cases, error conditions, and the full hackathon demo flow as an automated test. This ensures the program is robust for the hackathon demo and establishes the test foundation for production hardening.

### Ownership
- Agent: tess
- Stack: Test Frameworks
- Priority: high
- Status: pending
- Dependencies: 2, 3, 4

### Implementation Details
1. Set up test infrastructure in `tests/` directory:
   - `tests/tsconfig.json` for TypeScript Mocha/Chai tests.
   - `tests/helpers/` with shared utilities: `createFundedWallet()`, `mintTestUsdc()`, `findPda()`, `fetchAccount()`, `assertError()`, `sleep()`.
   - `tests/fixtures/` with sample receipt JSONs and test data.
   - Use `@coral-xyz/anchor` test framework with `anchor.workspace.CtoBilling`.
2. Implement test suites:
   **`tests/01-operator.test.ts` — Operator lifecycle:**
   - Initialize operator with valid params → success.
   - Initialize operator twice → fails (PDA already exists).
   - Non-authority tries to pause → fails with Unauthorized.
   - Authority pauses → paused=true. Authority unpauses → paused=false.
   - Initialize with protocol_fee_bps > 10000 → fails.
   **`tests/02-customer.test.ts` — Customer account & escrow:**
   - Create customer account → PDA exists with correct caps.
   - Deposit 100 USDC → balance=100, vault balance=100.
   - Deposit 0 → fails with InvalidAmount.
   - Withdraw 50 → balance=50, customer ATA +50.
   - Withdraw more than balance → fails with InsufficientBalance.
   - Withdraw while paused → fails with ProgramPaused.
   - Update spending caps → caps updated.
   - Update caps: max_per_task > max_per_day → fails with InvalidCaps.
   **`tests/03-agent-package.test.ts` — Agent package registry:**
   - Register package → PDA created, counters at 0.
   - Register with split_bps > 10000 → fails.
   - Register duplicate package_id → fails (PDA collision).
   - Update package by author → success.
   - Update package by non-author → fails with Unauthorized.
   - Deactivate package (active=false) → active flag changes.
   **`tests/04-settlement.test.ts` — Settlement (the core test suite):**
   - **Case 1:** Settle without agent package → customer debited, treasury credited, TaskReceipt correct.
   - **Case 2:** Settle with agent package, quality_met=true, split_bps=3000 → 30% to author, 70% to treasury. Verify exact token amounts.
   - **Case 2 edge:** split_bps=0 → author gets 0, treasury gets 100%.
   - **Case 2 edge:** split_bps=10000 → author gets 100%, treasury gets 0.
   - **Case 3:** quality_met=false → customer not debited, author earns 0, TaskReceipt.amount=0.
   - Settle exceeding per-task cap → fails.
   - Settle exceeding daily cap → fails.
   - Settle while paused → fails.
   - Settle with insufficient balance → fails.
   - Settle same task_id twice → fails (PDA exists).
   - Settle with inactive agent package → fails.
   - Non-operator tries to settle → fails with Unauthorized.
   **`tests/05-refund.test.ts` — Refund flow:**
   - Refund settled task → status=Refunded, customer balance restored.
   - Refund already refunded task → fails.
   - Refund non-existent task → fails.
   - Non-operator tries to refund → fails.
   **`tests/06-daily-reset.test.ts` — Spending cap reset:**
   - Simulate daily_spent near cap, advance slots past SLOTS_PER_DAY, settle again → daily_spent reset, settlement succeeds.
   **`tests/07-e2e-demo-flow.test.ts` — Full hackathon demo as automated test:**
   - Replicate the exact CLI demo flow (Task 6) as an automated test. Initialize operator, register package, create customer, deposit, settle with quality met, settle with quality not met, verify receipts, withdraw. All in one test with assertions at every step.
3. Add a `scripts/run-tests.sh` that starts localnet, deploys the program, runs all test suites, and exits with appropriate code.
4. Configure test reporter for CI-friendly output (mocha-junit-reporter).

### Subtasks
- [ ] Set up test infrastructure with helpers, fixtures, and configuration: Create the test infrastructure foundation: TypeScript config for Mocha/Chai tests, shared helper utilities (createFundedWallet, mintTestUsdc, findPda, fetchAccount, assertError, sleep), sample receipt fixtures, and test reporter configuration.
- [ ] Implement operator lifecycle test suite (01-operator.test.ts): Write the operator lifecycle test suite covering initialization with valid and invalid params, duplicate initialization prevention, authorization checks for pause/unpause operations, and protocol fee validation.
- [ ] Implement customer account and escrow test suite (02-customer.test.ts): Write the customer account test suite covering account creation, USDC deposits, zero-amount validation, withdrawals, overdraw prevention, pause-state checking, spending cap updates, and cap validation logic.
- [ ] Implement agent package registry test suite (03-agent-package.test.ts): Write the agent package registry test suite covering package registration, invalid split_bps validation, duplicate package prevention, author-only updates, unauthorized update rejection, and package deactivation.
- [ ] Implement settlement test suite (04-settlement.test.ts): Write the comprehensive settlement test suite covering all three settlement cases (no agent package, quality met with split, quality not met), edge cases for split_bps boundaries, and all error conditions (cap violations, pause state, insufficient balance, duplicate task, inactive package, unauthorized).
- [ ] Implement refund flow test suite (05-refund.test.ts): Write the refund test suite covering successful refund of a settled task, prevention of double refunds, handling of non-existent tasks, and authorization checks.
- [ ] Implement daily spending cap reset test suite (06-daily-reset.test.ts): Write the daily spending cap reset test suite that simulates time advancement past the daily boundary to verify the daily_spent counter resets, allowing new settlements to proceed.
- [ ] Implement end-to-end demo flow test (07-e2e-demo-flow.test.ts): Write the comprehensive E2E test that replicates the exact hackathon CLI demo flow as a single automated test: initialize operator, register agent package, create customer, deposit, settle with quality met, settle with quality not met, verify all receipts and state, then withdraw.
- [ ] Create test runner script and CI integration configuration: Build the scripts/run-tests.sh script that orchestrates the full test lifecycle (start localnet, deploy program, run all test suites, report results) and configure CI-friendly output with mocha-junit-reporter.