<identity>
You are nova working on subtask 6001 of task 6.
</identity>

<context>
<scope>
Create the cli/ directory structure, package.json with all dependencies, TypeScript config, IDL import, and shared utility functions for wallet loading, devnet connection, SOL airdrop, and test USDC minting.
</scope>
</context>

<implementation_plan>
1. Create directory structure:
   ```
   cli/
   ├── package.json
   ├── tsconfig.json
   ├── src/
   │   ├── demo.ts          (entry point, stubbed)
   │   ├── utils/
   │   │   ├── connection.ts  (RPC connection setup)
   │   │   ├── wallets.ts     (keypair loading/generation)
   │   │   ├── airdrop.ts     (SOL airdrop with retry)
   │   │   ├── usdc.ts        (test USDC mint creation and minting)
   │   │   ├── pda.ts         (PDA derivation helpers for all account types)
   │   │   ├── display.ts     (chalk-based colored output, explorer link formatter, table printer)
   │   │   └── idl.ts         (IDL loading and Anchor program initialization)
   │   └── commands/          (empty directory for subtask 6004)
   ```
2. `package.json` dependencies:
   - `@coral-xyz/anchor`: "^0.30.0"
   - `@solana/web3.js`: "^1.95.0"
   - `@solana/spl-token`: "^0.4.0"
   - `commander`: "^12.0.0"
   - `chalk`: "^5.0.0"
   - `ora`: "^8.0.0"
   - Workspace link to receipt-uploader if monorepo, otherwise relative import.
3. `connection.ts`: export `getConnection(rpcUrl?: string)` returning a `Connection` with `confirmed` commitment. Default to devnet.
4. `wallets.ts`: export `loadKeypair(envVar: string, filePath?: string)` that tries env var first (base58 or JSON array), then file. Export `generateKeypair()` for ephemeral test wallets.
5. `airdrop.ts`: export `ensureSol(connection, pubkey, minBalance=2)` that airdrops 2 SOL if balance < minBalance, with retry logic (devnet faucet can be flaky).
6. `usdc.ts`: export `ensureUsdcMint(connection, payer)` that creates a test SPL token mint (6 decimals) if it doesn't exist, and `mintUsdc(connection, mint, payer, destination, amount)` to mint test tokens.
7. `pda.ts`: export PDA derivation for all program accounts — `deriveOperatorConfig`, `deriveCustomerBalance`, `deriveTaskReceipt`, `deriveAgentPackage`, `deriveVault`.
8. `display.ts`: export `explorerLink(signature: string, cluster?: string)` returning `https://explorer.solana.com/tx/{sig}?cluster=devnet`, `printStep(stepNum, title)` with colored header, `printSuccess(msg)`, `printError(msg)`.
9. `idl.ts`: export `getProgram(connection, wallet)` that loads the IDL from `../../target/idl/cto_billing.json` and returns an initialized Anchor `Program` instance.
</implementation_plan>

<validation>
Run `bun install` in cli/ — all dependencies resolve. Run `bun run src/utils/connection.ts` — no import errors. Verify `deriveOperatorConfig` with a known pubkey produces the expected PDA (compare against Anchor's derivation in tests). Verify `loadKeypair` reads from a test JSON file and returns a valid Keypair. Verify `ensureSol` on devnet airdrops SOL to a fresh wallet.
</validation>