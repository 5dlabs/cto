<identity>
You are stitch working on subtask 6001 of task 6.
</identity>

<context>
<scope>
Create the demo/cli/ project with all dependencies, implement PDA derivation functions matching on-chain SHA-256 logic, build Explorer link generators, and set up chalk/ora formatting helpers.
</scope>
</context>

<implementation_plan>
1. Create `demo/cli/` directory with `package.json` (name: cto-demo-cli, type: module), `tsconfig.json` (target: ES2022, module: NodeNext, strict: true, outDir: dist).
2. Install dependencies: @coral-xyz/anchor, @solana/web3.js, @solana/spl-token, chalk (v5 ESM), ora, commander (for CLI arg parsing).
3. Implement `src/cli.ts`: use commander to parse `--cluster` (localnet|devnet, default localnet) and `--keypair` (operator keypair path, default ~/.config/solana/id.json). Map cluster to RPC URL (localhost:8899 for localnet, devnet URL for devnet).
4. Implement `src/pda.ts` with functions: `deriveOperatorConfig(programId)`, `deriveCustomerBalance(programId, customer)`, `deriveAgentPackage(programId, packageId)`, `deriveTaskReceipt(programId, taskId)`, `deriveVault(programId, mint)`. Each must use the SHA-256 hashing approach matching the on-chain program (hash relevant inputs with crypto.createHash('sha256'), use result as seed in PublicKey.findProgramAddressSync).
5. Implement `src/utils.ts` with: `explorerLink(signature, cluster)` returning `https://explorer.solana.com/tx/${sig}?cluster=${cluster}`, `formatUsdc(lamports: number)` returning formatted string with 2 decimal places (divide by 1_000_000), `phaseHeader(num, title)` using chalk.bold for section headers.
6. Implement `src/receipt.ts`: generate mock receipt JSON with task_id, customer pubkey, duration_seconds, infra_tier, provider, agent_used, cost breakdown object, total, ISO timestamp. Export function to compute SHA-256 hash of the JSON string as a Uint8Array for the receipt_hash parameter.
7. Import IDL from `../../target/idl/cto_billing.json` and set up Anchor Program instance creation in a helper.
8. Add npm scripts: `demo:localnet` (ts-node or tsx src/index.ts --cluster localnet), `demo:devnet` (tsx src/index.ts --cluster devnet).
</implementation_plan>

<validation>
Run `npx tsc --noEmit` — compiles with zero errors. Unit test PDA derivation: call each derive function with known inputs and verify output matches expected Pubkeys from Anchor test suite (Task 2). Unit test formatUsdc(10_000_000) returns '10.00'. Unit test explorerLink returns correctly formatted URL. Unit test receipt hash: compute hash of a fixed JSON string and verify it matches expected SHA-256 digest.
</validation>