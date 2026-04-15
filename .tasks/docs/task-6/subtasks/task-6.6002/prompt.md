<identity>
You are nova working on subtask 6002 of task 6.
</identity>

<context>
<scope>
Implement the first half of demo.ts covering environment setup (Step 0), operator initialization (Step 1), agent package registration (Step 2), customer account creation (Step 3), and USDC deposit (Step 4).
</scope>
</context>

<implementation_plan>
1. In `cli/src/demo.ts`, implement the main async function with sequential steps:
2. **Step 0 — Setup:**
   - Use `ora` spinner while setting up.
   - Call `getConnection()` for devnet.
   - Load or generate 3 keypairs: operator, author, customer.
   - Call `ensureSol` for all three (airdrop if needed).
   - Call `ensureUsdcMint` to create test USDC mint.
   - Call `mintUsdc` to mint 100 USDC (100_000_000 lamports) to customer's ATA.
   - Print all wallet addresses and balances using `chalk` colored output.
3. **Step 1 — Initialize Operator:**
   - Call `program.methods.initializeOperator(treasuryPubkey, 500).accounts({...}).signers([operator]).rpc()`.
   - Print OperatorConfig PDA address and `explorerLink(txSig)`.
   - Use `printStep(1, "Initialize Operator")` header.
4. **Step 2 — Register Agent Package:**
   - Call `program.methods.registerAgentPackage("rex-code-agent", 3000, "https://arweave.net/mock-package-hash").accounts({...}).signers([author]).rpc()`.
   - Print AgentPackage PDA, author wallet, split percentage.
5. **Step 3 — Create Customer Account:**
   - Call `program.methods.createCustomerAccount(new BN(50_000_000), new BN(200_000_000)).accounts({...}).signers([customer]).rpc()`.
   - Print CustomerBalance PDA and spending limits.
6. **Step 4 — Deposit USDC:**
   - Call `program.methods.deposit(new BN(100_000_000)).accounts({...}).signers([customer]).rpc()`.
   - Fetch and print new CustomerBalance (should show 100 USDC).
   - Fetch and print vault token account balance.
7. Store all transaction signatures in an array for the summary step.
8. Between each step, add a brief `console.log` separator and the step timing.
9. Export the state (keypairs, program, mint, PDAs, signatures) for use by subsequent steps.
</implementation_plan>

<validation>
Run `bun run src/demo.ts` with a `--stop-after=4` flag (or comment out later steps) against devnet. Steps 0-4 complete without error. Verify on-chain: OperatorConfig PDA exists with protocol_fee_bps=500. AgentPackage PDA exists with package_id="rex-code-agent" and split_bps=3000. CustomerBalance PDA exists with balance=100_000_000. Vault token account holds 100 USDC. Console output shows colored step headers, wallet addresses, PDA addresses, and explorer links.
</validation>