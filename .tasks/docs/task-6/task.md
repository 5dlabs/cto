## End-to-End CLI Demo Script (Stitch - TypeScript/Solana-Web3.js/Anchor)

### Objective
Build the hackathon demo script that executes the full billing + marketplace flow end-to-end, producing console output and Solana Explorer links for the demo video (PRD Section 11, Deliverable #2).

### Ownership
- Agent: stitch
- Stack: TypeScript/Solana-Web3.js/@coral-xyz/anchor
- Priority: high
- Status: pending
- Dependencies: 1, 2

### Implementation Details
1. Set up project at `demo/cli/` with package.json, tsconfig.json. Dependencies: @coral-xyz/anchor, @solana/web3.js, @solana/spl-token, chalk (colored output), ora (spinners). Import IDL from `../../target/idl/cto_billing.json` and types from `../../target/types/cto_billing.ts`.

2. Configuration: accept `--cluster` flag (localnet|devnet, default localnet). Accept `--keypair` for operator keypair path. Use `@solana/web3.js` Connection with appropriate RPC URL.

3. Implement PDA derivation helpers matching on-chain logic: `deriveOperatorConfig()`, `deriveCustomerBalance(customer)`, `deriveAgentPackage(packageId)`, `deriveTaskReceipt(taskId)`, `deriveVault(mint)`. All use SHA-256 hashing per D2.

4. Implement 10 demo phases (each logs a section header with phase number):

   **Phase 0: Setup & Token Provisioning**
   - Generate keypairs: operator, customer, agentAuthor.
   - If localnet: airdrop SOL to all three.
   - Create a mock USDC mint (6 decimals) owned by operator.
   - Create ATAs for: customer, agentAuthor, treasury (operator's second ATA).
   - Mint 1000 USDC to customer's ATA.
   - Log: all pubkeys, mint address, Solana Explorer links.

   **Phase 1: Initialize Operator**
   - Call `initialize_operator` with treasury ATA, mock USDC mint, protocol_fee_bps=0.
   - Log: OperatorConfig PDA, transaction signature, Explorer link.

   **Phase 2: Register Agent Package**
   - Author calls `register_agent_package` with package_id='smart-debug-agent-v1', split_bps=3000 (30%), source_uri='https://github.com/example/smart-debug-agent'.
   - Log: AgentPackage PDA, author wallet, split percentage, Explorer link.

   **Phase 3: Create Customer Account**
   - Customer calls `create_customer_account` with max_per_task=100_000_000 (100 USDC), max_per_day=500_000_000 (500 USDC).
   - Log: CustomerBalance PDA, spending caps.

   **Phase 4: Customer Deposits USDC**
   - Customer calls `deposit` with 200 USDC (200_000_000).
   - Log: deposit amount, new balance, Explorer link.

   **Phase 5: Settle Task — Quality Met (Author Gets Paid)**
   - Operator calls `settle_task` with task_id='CTO-1234', amount=10_000_000 (10 USDC), receipt_hash (SHA-256 of mock receipt JSON), quality_met=true, agent_package PDA.
   - Log: task_id, amount charged, author earned (3 USDC), treasury received (7 USDC), TaskReceipt PDA, quality_met=true, Explorer link.
   - Highlight the SPLIT with colored output: '🟢 Quality MET — Author earned 3.00 USDC (30%), Treasury received 7.00 USDC (70%)'.

   **Phase 6: Settle Task — Quality NOT Met (Customer Protected)**
   - Operator calls `settle_task` with task_id='CTO-1235', amount=15_000_000 (15 USDC), quality_met=false, agent_package PDA.
   - Log: task_id, amount charged=0, author earned=0, customer balance unchanged, quality_met=false.
   - Highlight: '🔴 Quality NOT MET — Customer not charged. Author earned nothing.'

   **Phase 7: Settle Task — Default Agent (No Split)**
   - Operator calls `settle_task` with task_id='CTO-1236', amount=5_000_000 (5 USDC), no agent_package, quality_met=true.
   - Log: full amount to treasury, no split.

   **Phase 8: Verify On-Chain Receipts**
   - Fetch all three TaskReceipt PDAs. Display a formatted table showing:
     | Task ID | Amount | Author Earned | Quality | Status |
     |---------|--------|---------------|---------|--------|
     | CTO-1234 | 10.00 | 3.00 | ✅ MET | Settled |
     | CTO-1235 | 0.00 | 0.00 | ❌ NOT MET | Settled |
     | CTO-1236 | 5.00 | 0.00 | ✅ MET | Settled |
   - Fetch AgentPackage: show total_earned, task_count, success_count.

   **Phase 9: Customer Withdraws Remaining Balance**
   - Customer calls `withdraw` with remaining balance.
   - Log: withdrawal amount, final balance=0, Explorer link.
   - Final summary: 'Demo complete! Customer deposited 200 USDC, was charged 15 USDC for 2 successful tasks, was protected from 1 failed task, and withdrew 185 USDC.'

5. Add `npm run demo:localnet` and `npm run demo:devnet` scripts.

6. Handle errors gracefully: if any phase fails, log the error with context and the Solana transaction logs, then exit with non-zero code.

7. Generate a mock receipt JSON blob for Phase 5 with fields: task_id, customer, duration_seconds, infra_tier, provider, agent_used, cost breakdown, total, timestamp. Compute SHA-256 hash of this JSON string for the receipt_hash parameter.

### Subtasks
- [ ] Set up TypeScript project, CLI argument parsing, PDA derivation helpers, and formatting utilities: Create the demo/cli/ project with all dependencies, implement PDA derivation functions matching on-chain SHA-256 logic, build Explorer link generators, and set up chalk/ora formatting helpers.
- [ ] Implement Phases 0-3: Setup, token provisioning, operator init, agent registration, customer account: Build the first four demo phases covering keypair generation, SOL airdrop, mock USDC mint/ATA creation, operator initialization, agent package registration, and customer account creation.
- [ ] Implement Phases 4-7: Deposit, three settlement scenarios with colored split output: Build the deposit phase and three distinct settlement scenarios demonstrating quality-met with agent split, quality-not-met protection, and default agent (no split), with rich colored console output.
- [ ] Implement Phases 8-9: On-chain verification table, withdrawal, final summary, and error handling: Build the receipt verification phase with formatted table output, customer withdrawal with final summary, global error handling, and wire all phases together in the main entry point.