<identity>
You are blaze working on subtask 7007 of task 7.
</identity>

<context>
<scope>
Create the useProgram() hook that returns an initialized Anchor Program instance using the connected wallet and IDL, and the usePdaDerivation() hook providing PDA derivation helpers using SHA-256 for all program accounts.
</scope>
</context>

<implementation_plan>
1. Copy the program IDL from `target/idl/cto_billing.json` to `src/idl/cto_billing.json` (or set up a path alias).
2. Create `src/hooks/useProgram.ts`:
   - Import the IDL JSON.
   - Use `useConnection()` from wallet-adapter-react to get the Connection.
   - Use `useAnchorWallet()` from wallet-adapter-react to get the wallet.
   - Create an `AnchorProvider` with the connection and wallet.
   - Return `new Program(idl, PROGRAM_ID, provider)` — define PROGRAM_ID as a constant (or from env var `NEXT_PUBLIC_PROGRAM_ID`).
   - Return null or undefined when wallet is not connected.
   - Memoize the program instance with useMemo to avoid recreation on every render.
3. Create `src/hooks/usePdaDerivation.ts`:
   - Import `PublicKey` from `@solana/web3.js`.
   - Implement `deriveCustomerBalance(wallet: PublicKey): [PublicKey, number]` using `PublicKey.findProgramAddressSync([Buffer.from('customer_balance'), wallet.toBuffer()], PROGRAM_ID)`.
   - Implement `deriveAgentPackage(packageId: string): [PublicKey, number]` using SHA-256 hash of packageId as seed.
   - Implement `deriveTaskReceipt(taskId: string): [PublicKey, number]` using SHA-256 hash of taskId as seed.
   - Implement `deriveVault(mint: PublicKey): [PublicKey, number]` with 'vault' + mint seeds.
   - Implement `deriveOperatorConfig(): [PublicKey, number]` with 'operator_config' seed.
   - For SHA-256: use `crypto.subtle.digest('SHA-256', ...)` or a synchronous library. Since PDA derivation with findProgramAddressSync is synchronous, use a synchronous SHA-256 (e.g., `js-sha256` or Node's `crypto.createHash`). Install `js-sha256` if needed.
   - Return all derivation functions from the hook, memoized.
4. Export both hooks from `src/hooks/index.ts`.
</implementation_plan>

<validation>
Run `npx tsc --noEmit` — zero type errors. Connect a Phantom wallet on devnet and verify useProgram() returns a non-null Program instance by logging `program.programId.toBase58()`. Verify usePdaDerivation() functions return valid PublicKey objects by calling `deriveCustomerBalance(wallet.publicKey)` and confirming the result is a PublicKey tuple. Verify `deriveOperatorConfig()` returns the same PDA on repeated calls (deterministic). Test that useProgram() returns null/undefined when wallet is disconnected.
</validation>