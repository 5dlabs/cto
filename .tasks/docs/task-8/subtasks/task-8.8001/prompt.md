<identity>
You are tess working on subtask 8001 of task 8.
</identity>

<context>
<scope>
Create the test infrastructure foundation: TypeScript config for Mocha/Chai tests, shared helper utilities (createFundedWallet, mintTestUsdc, findPda, fetchAccount, assertError, sleep), sample receipt fixtures, and test reporter configuration.
</scope>
</context>

<implementation_plan>
1. Create `tests/tsconfig.json` extending the root tsconfig with test-specific settings (mocha globals, chai types).
2. Create `tests/helpers/index.ts` exporting all helper functions:
   a. `createFundedWallet(connection, lamports?)`: generates a new Keypair, airdrops SOL, returns the funded wallet.
   b. `mintTestUsdc(connection, authority, recipient, amount)`: creates a USDC-like SPL token mint (or uses existing devnet USDC), mints specified amount to recipient's ATA. Handles ATA creation if needed.
   c. `findPda(seeds, programId)`: wrapper around PublicKey.findProgramAddressSync for cleaner test code. Include specific PDA finders: findOperatorPda(), findCustomerPda(customer), findTaskReceiptPda(taskId, customer), findAgentPackagePda(packageId).
   d. `fetchAccount(program, accountType, pda)`: typed account fetch with null handling.
   e. `assertError(fn, errorCode)`: executes async fn, asserts it throws AnchorError with expected error code string. Provides clear failure messages.
   f. `sleep(ms)`: promisified setTimeout for timing-sensitive tests.
3. Create `tests/fixtures/sample-receipt.json`: a sample itemized receipt JSON matching the expected Arweave schema.
4. Create `tests/fixtures/test-constants.ts`: shared test values (default caps, deposit amounts, task IDs, package IDs) to avoid magic numbers.
5. Verify `anchor.workspace.CtoBilling` loads correctly in a minimal test file.
6. Configure mocha-junit-reporter in `.mocharc.yml` for CI output alongside spec reporter for local dev.
</implementation_plan>

<validation>
Run a minimal test file that imports all helpers and verifies: createFundedWallet returns a keypair with SOL balance > 0, mintTestUsdc creates tokens in the recipient's ATA, findPda returns deterministic addresses, assertError correctly catches and validates Anchor errors. All helpers compile without TypeScript errors.
</validation>