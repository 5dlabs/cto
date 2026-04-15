## Build Hackathon Demo CLI Script — Full E2E Flow (Nova - Bun/TypeScript)

### Objective
Create a TypeScript CLI script that demonstrates the complete hackathon flow end-to-end: register an agent package, create customer, deposit USDC, settle with quality met (author gets paid), settle with quality NOT met (customer not charged), verify receipts on-chain and on Arweave, and withdraw remaining balance. This is hackathon deliverable #2 and the basis for the demo video.

### Ownership
- Agent: nova
- Stack: Bun/TypeScript
- Priority: high
- Status: pending
- Dependencies: 2, 3, 4

### Implementation Details
1. Create `cli/` directory with a Bun TypeScript project:
   - `package.json` with dependencies: `@coral-xyz/anchor` (0.30.x), `@solana/web3.js` (^1.95), `@solana/spl-token` (^0.4), `@cto-billing/receipt-uploader` (workspace link), `commander` (^12), `chalk` (^5), `ora` (^8) for CLI UX.
   - Import the IDL from `target/idl/cto_billing.json`.
2. Implement `cli/src/demo.ts` as the main demo script with the following sequential steps, each with colorful console output and Solana explorer links:
   **Step 0 — Setup:**
   - Load operator keypair from env or file.
   - Connect to devnet RPC.
   - Airdrop SOL to operator, author, and customer wallets if balances are low.
   - Ensure test USDC mint exists and mint tokens to customer wallet.
   **Step 1 — Initialize Operator:**
   - Call `initialize_operator` with treasury pubkey and protocol_fee_bps=500 (5%).
   - Print OperatorConfig PDA address and explorer link.
   **Step 2 — Register Agent Package (Author):**
   - Generate or load an author keypair.
   - Call `register_agent_package` with package_id="rex-code-agent", split_bps=3000 (30%), source_uri="https://arweave.net/mock-package-hash".
   - Print AgentPackage PDA and author wallet.
   **Step 3 — Create Customer Account:**
   - Generate or load a customer keypair.
   - Call `create_customer_account` with max_per_task=50_000_000 (50 USDC), max_per_day=200_000_000 (200 USDC).
   - Print CustomerBalance PDA.
   **Step 4 — Deposit USDC:**
   - Call `deposit` with 100_000_000 (100 USDC).
   - Print new balance and vault balance.
   **Step 5 — Settle Task (Quality MET):**
   - Build a sample receipt JSON for task "TASK-001" with billing items (coderun: 15 min @ $0.75, infra: standard tier).
   - Upload receipt to Arweave via receipt-uploader module.
   - Call `settle_task` with amount=10_000_000 (10 USDC), receipt_hash, agent_package PDA, quality_met=true.
   - Print: TaskReceipt PDA, customer balance (should be 90 USDC), author earned (3 USDC = 30%), treasury received (7 USDC), Arweave receipt link.
   **Step 6 — Settle Task (Quality NOT MET):**
   - Build receipt for task "TASK-002".
   - Upload to Arweave.
   - Call `settle_task` with amount=10_000_000, quality_met=false.
   - Print: TaskReceipt PDA shows amount=0, customer balance unchanged (still 90 USDC), author earned=0.
   **Step 7 — Verify Receipts:**
   - Fetch both TaskReceipt PDAs and print their full contents.
   - Fetch Arweave receipts and verify hashes match on-chain receipt_hash.
   - Print verification status (✓ or ✗).
   **Step 8 — Withdraw Remaining Balance:**
   - Call `withdraw` with full remaining balance.
   - Print final customer balance (0) and customer ATA balance.
   **Step 9 — Summary:**
   - Print a summary table showing all transactions with Solana explorer links.
3. Add `cli/src/commands/` with individual command files for each operation (for manual use beyond the demo).
4. Add `cli/package.json` script: `"demo": "bun run src/demo.ts"`.
5. Handle errors gracefully with descriptive messages and transaction links for debugging.

### Subtasks
- [ ] Scaffold CLI project with dependencies and setup utilities: Create the cli/ directory structure, package.json with all dependencies, TypeScript config, IDL import, and shared utility functions for wallet loading, devnet connection, SOL airdrop, and test USDC minting.
- [ ] Implement demo steps 0-4: Setup, initialize operator, register agent, create customer, deposit: Implement the first half of demo.ts covering environment setup (Step 0), operator initialization (Step 1), agent package registration (Step 2), customer account creation (Step 3), and USDC deposit (Step 4).
- [ ] Implement demo steps 5-9: Settle (quality met/not met), verify receipts, withdraw, summary: Implement the second half of demo.ts covering quality-met settlement (Step 5), quality-not-met settlement (Step 6), receipt verification (Step 7), withdrawal (Step 8), and the summary table (Step 9).
- [ ] Implement individual CLI command modules for standalone usage: Create separate command files in cli/src/commands/ for each Solana program operation, wired up with Commander, so users can invoke operations individually beyond the scripted demo flow.