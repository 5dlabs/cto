<identity>
You are stitch working on subtask 6003 of task 6.
</identity>

<context>
<scope>
Build the deposit phase and three distinct settlement scenarios demonstrating quality-met with agent split, quality-not-met protection, and default agent (no split), with rich colored console output.
</scope>
</context>

<implementation_plan>
1. Implement `src/phases/phase4.ts` — Customer Deposits USDC:
   a. Build and send `deposit` instruction signed by customer with amount=200_000_000 (200 USDC).
   b. Fetch CustomerBalance PDA to confirm new balance.
   c. Log deposit amount, new balance, Explorer link.

2. Implement `src/phases/phase5.ts` — Settle Task: Quality Met (Author Gets Paid):
   a. Generate mock receipt JSON using receipt helper, compute SHA-256 hash.
   b. Build and send `settle_task` with: task_id='CTO-1234', amount=10_000_000 (10 USDC), receipt_hash, quality_met=true, include agent_package PDA (smart-debug-agent-v1) and agentAuthor ATA.
   c. After confirmation, fetch TaskReceipt PDA to get actual amounts.
   d. Calculate expected split: author_earned = 10 * 30% = 3.00 USDC, treasury_received = 7.00 USDC.
   e. Log with chalk.green: '🟢 Quality MET — Author earned 3.00 USDC (30%), Treasury received 7.00 USDC (70%)'.
   f. Log TaskReceipt PDA, Explorer link.

3. Implement `src/phases/phase6.ts` — Settle Task: Quality NOT Met:
   a. Build and send `settle_task` with: task_id='CTO-1235', amount=15_000_000, quality_met=false, include agent_package PDA.
   b. Fetch CustomerBalance to confirm balance unchanged.
   c. Log with chalk.red: '🔴 Quality NOT MET — Customer not charged. Author earned nothing.'
   d. Log that amount_charged=0, author_earned=0.

4. Implement `src/phases/phase7.ts` — Settle Task: Default Agent (No Split):
   a. Build and send `settle_task` with: task_id='CTO-1236', amount=5_000_000 (5 USDC), quality_met=true, NO agent_package (pass null/None).
   b. Log: '💰 Full 5.00 USDC to treasury (no agent split)' using chalk.yellow.
   c. Log TaskReceipt PDA, Explorer link.

5. Each phase uses ora spinners and phaseHeader formatting. Error handling: catch failures, log Solana transaction logs (program logs from the error), then re-throw.
</implementation_plan>

<validation>
Run phases 4-7 sequentially after phases 0-3 against local validator. Verify: (1) Phase 4 — CustomerBalance.balance = 200_000_000 after deposit. (2) Phase 5 — TaskReceipt for CTO-1234 exists with amount=10_000_000, quality_met=true. AgentAuthor ATA balance increased by 3_000_000. Treasury ATA balance increased by 7_000_000. CustomerBalance.balance decreased by 10_000_000. (3) Phase 6 — TaskReceipt for CTO-1235 exists with amount=0 or quality_met=false. CustomerBalance.balance unchanged from phase 5's end. (4) Phase 7 — TaskReceipt for CTO-1236 exists with amount=5_000_000. Treasury ATA balance increased by 5_000_000. Console output contains expected emoji and colored strings.
</validation>