<identity>
You are stitch working on subtask 6002 of task 6.
</identity>

<context>
<scope>
Build the first four demo phases covering keypair generation, SOL airdrop, mock USDC mint/ATA creation, operator initialization, agent package registration, and customer account creation.
</scope>
</context>

<implementation_plan>
1. Implement `src/phases/phase0.ts` — Setup & Token Provisioning:
   a. Generate 3 keypairs: operator, customer, agentAuthor using Keypair.generate().
   b. If cluster is localnet: airdrop 10 SOL to each via connection.requestAirdrop(), confirm each.
   c. Create mock USDC mint with 6 decimals using createMint() from @solana/spl-token, mint authority = operator.
   d. Create ATAs: customer ATA, agentAuthor ATA, treasury ATA (for operator's treasury address) using getOrCreateAssociatedTokenAccount().
   e. Mint 1000 USDC (1_000_000_000 lamports) to customer's ATA using mintTo().
   f. Log all pubkeys, mint address, and Explorer links using chalk formatting.
   g. Return all keypairs, mint, and ATA addresses for use in subsequent phases.

2. Implement `src/phases/phase1.ts` — Initialize Operator:
   a. Build and send `initialize_operator` instruction via Anchor program with treasury ATA, mock USDC mint, protocol_fee_bps=0.
   b. Log OperatorConfig PDA (derived via helper), transaction signature, Explorer link.

3. Implement `src/phases/phase2.ts` — Register Agent Package:
   a. Build and send `register_agent_package` signed by agentAuthor keypair with package_id='smart-debug-agent-v1', split_bps=3000, source_uri='https://github.com/example/smart-debug-agent'.
   b. Log AgentPackage PDA, author wallet, '30% revenue split', Explorer link.

4. Implement `src/phases/phase3.ts` — Create Customer Account:
   a. Build and send `create_customer_account` signed by customer keypair with max_per_task=100_000_000, max_per_day=500_000_000.
   b. Log CustomerBalance PDA, spending caps formatted as USDC amounts.

5. Each phase function logs a header line using phaseHeader() utility, wraps the work in an ora spinner, and throws with context on failure.
</implementation_plan>

<validation>
Run phases 0-3 against a running solana-test-validator with cto-billing deployed. Verify: (1) Phase 0 — customer ATA balance is 1000 USDC (fetch token account, check amount). (2) Phase 1 — OperatorConfig PDA account exists on-chain, treasury field matches expected ATA. (3) Phase 2 — AgentPackage PDA exists, split_bps=3000, author matches agentAuthor pubkey. (4) Phase 3 — CustomerBalance PDA exists, max_per_task=100_000_000. All four phases complete without errors.
</validation>