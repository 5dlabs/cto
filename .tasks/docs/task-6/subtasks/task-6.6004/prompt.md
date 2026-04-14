<identity>
You are stitch working on subtask 6004 of task 6.
</identity>

<context>
<scope>
Build the receipt verification phase with formatted table output, customer withdrawal with final summary, global error handling, and wire all phases together in the main entry point.
</scope>
</context>

<implementation_plan>
1. Implement `src/phases/phase8.ts` — Verify On-Chain Receipts:
   a. Derive and fetch all three TaskReceipt PDAs (CTO-1234, CTO-1235, CTO-1236) using program.account.taskReceipt.fetch().
   b. Build a formatted console table (use string padding or a simple table library) with columns: Task ID, Amount (USDC), Author Earned (USDC), Quality, Status.
   c. Row 1: CTO-1234 | 10.00 | 3.00 | ✅ MET | Settled
   d. Row 2: CTO-1235 | 0.00 | 0.00 | ❌ NOT MET | Settled
   e. Row 3: CTO-1236 | 5.00 | 0.00 | ✅ MET | Settled
   f. Fetch AgentPackage PDA for smart-debug-agent-v1. Display: total_earned, task_count, success_count.

2. Implement `src/phases/phase9.ts` — Customer Withdraws:
   a. Fetch CustomerBalance to get remaining balance (should be 185_000_000 = 200 - 10 - 5).
   b. Call `withdraw` instruction signed by customer with the full remaining balance.
   c. Verify CustomerBalance.balance is now 0.
   d. Log withdrawal amount (185.00 USDC), final balance (0.00), Explorer link.
   e. Print final summary with chalk.bold: 'Demo complete! Customer deposited 200 USDC, was charged 15 USDC for 2 successful tasks, was protected from 1 failed task, and withdrew 185 USDC.'

3. Implement `src/index.ts` — Main entry point:
   a. Parse CLI args using commander.
   b. Create Solana Connection and Anchor Provider.
   c. Execute phases 0 through 9 sequentially, passing state (keypairs, addresses, mint) between phases.
   d. Wrap the entire sequence in a try/catch: on error, log the phase that failed, the error message, and if it's a SendTransactionError, log the program logs. Exit with process.exit(1).
   e. On success, log total execution time.

4. Ensure `npm run demo:localnet` and `npm run demo:devnet` scripts are correct and functional.
</implementation_plan>

<validation>
Run `npm run demo:localnet` end-to-end against solana-test-validator with cto-billing deployed. The script completes all 10 phases without errors in under 120 seconds. Verify: (1) Phase 8 table output contains all 3 rows with correct values — parse stdout for 'CTO-1234', '10.00', '3.00', '✅ MET'. (2) Phase 9 output contains '185.00' as withdrawal amount. (3) Final summary message appears in stdout. (4) Customer ATA balance after script = 185_000_000 (USDC returned). CustomerBalance PDA balance = 0. (5) Exit code is 0 on success. (6) Deliberately break one phase (e.g., wrong program ID) — verify error is caught, logged with context, and exit code is 1. (7) `npx tsc --noEmit` passes with zero errors.
</validation>