<identity>
You are tess working on subtask 5001 of task 5.
</identity>

<context>
<scope>
Create the foundational test infrastructure including Bankrun context initialization, SPL token helpers, program deployment helpers, customer factory, SHA-256 task_id_hash utility, and standard test wallet keypairs that all happy-path test files will import.
</scope>
</context>

<implementation_plan>
1. Create `tests/setup.ts`:
   - Initialize Bankrun context with `startAnchor()` pointing to the built program .so.
   - Export a `getTestContext()` function that returns a fresh BanksClient, payer, and recent blockhash.
   - Define standard keypairs: `operator`, `treasury`, `customer1`, `customer2` — generated deterministically or via `Keypair.generate()`.

2. Create `tests/helpers.ts`:
   - `createMint(context, authority, decimals=6)`: Creates an SPL token mint, returns mint pubkey.
   - `createTokenAccount(context, mint, owner)`: Creates an associated token account, returns ATA pubkey.
   - `mintTo(context, mint, dest, authority, amount)`: Mints tokens to a destination ATA.
   - `initializeProgram(context)`: Deploys program via Bankrun, calls `initialize_operator` with operator keypair, treasury pubkey, mint, and a default `protocol_fee_bps` (500), returns { operatorConfig PDA, vault PDA, mint, context }.
   - `createCustomer(context, programState, caps)`: Generates a new keypair, creates its ATA, mints 1_000_000_000 tokens (1000 USDC at 6 decimals), calls `create_customer_account` with given caps, returns { keypair, ata, customerBalancePDA }.
   - `computeTaskIdHash(taskIdString)`: Uses Node.js `crypto.createHash('sha256').update(taskIdString).digest()` to return a 32-byte Buffer.
   - `deriveCustomerBalancePDA(programId, operatorConfig, customerPubkey)`: Computes the PDA address.
   - `deriveTaskReceiptPDA(programId, operatorConfig, taskIdHash)`: Computes the PDA address.

3. Ensure all helpers handle Bankrun's transaction submission pattern (processTransaction, confirmTransaction).
4. Export all helpers as named exports for clean imports in test files.
5. Add a shared `SLOTS_PER_DAY = 216_000` constant for slot warp calculations.
</implementation_plan>

<validation>
Import helpers into a minimal smoke test: call `initializeProgram()` and `createCustomer()`, assert both return non-null PDAs and the customer's token account has 1_000_000_000 tokens. Verify `computeTaskIdHash('TASK-42')` returns a 32-byte Buffer matching the expected SHA-256 hex digest.
</validation>