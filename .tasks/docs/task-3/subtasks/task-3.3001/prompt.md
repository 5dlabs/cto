<identity>
You are tess working on subtask 3001 of task 3.
</identity>

<context>
<scope>
Set up the complete test infrastructure in tests/cto-billing.ts including all helper functions, PDA derivation utilities, keypair generation, Anchor workspace initialization, and the test:ci npm script. This is the foundation all test groups depend on.
</scope>
</context>

<implementation_plan>
1. Create `tests/cto-billing.ts` with Anchor test harness using `anchor.workspace.CtoBilling` and `@coral-xyz/anchor` Provider.
2. Implement helper functions:
   - `createMockMint(decimals=6)` — creates a new SPL token mint with the operator as mint authority.
   - `createAta(owner: PublicKey, mint: PublicKey)` — creates an associated token account.
   - `mintTo(ata: PublicKey, amount: number)` — mints tokens to a given ATA.
   - `derivePackagePda(packageId: string)` — derives PDA using seeds ['package', packageId] with SHA-256 consistent with on-chain logic (D2).
   - `deriveCustomerPda(customer: PublicKey)` — derives PDA using seeds ['customer', customer.toBytes()].
   - `deriveReceiptPda(taskId: string)` — derives PDA using seeds ['receipt', taskId].
   - `deriveVaultPda(mint: PublicKey)` — derives PDA using seeds ['vault', mint.toBytes()].
   - `airdropSol(pubkey: PublicKey, amount: number)` — airdrop SOL for transaction fees.
3. Generate test keypairs at module scope: operator, customer, author, second_customer, second_author.
4. Add a top-level `before()` hook that airdrops SOL to all keypairs and creates the mock USDC mint.
5. Add `test:ci` script to `package.json`: `"test:ci": "anchor test -- --timeout 60000"`.
6. Verify PDA derivation helpers produce deterministic results by deriving the same PDA twice and asserting equality.
</implementation_plan>

<validation>
Run `npx ts-node -e "require('./tests/cto-billing')"` to verify the file parses without syntax errors. Verify all helper functions are exported or accessible. Verify `npm run test:ci` command exists in package.json. Manually confirm PDA derivation helpers match expected seeds format by logging derived addresses for known inputs.
</validation>