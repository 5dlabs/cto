<identity>
You are blaze working on subtask 7004 of task 7.
</identity>

<context>
<scope>
Build the /deposit and /withdraw pages that allow users to deposit USDC into their CustomerBalance escrow and withdraw USDC back to their wallet, submitting Solana transactions via the wallet adapter.
</scope>
</context>

<implementation_plan>
1. Create `src/app/deposit/page.tsx`:
   a. Show current USDC wallet balance (fetch customer's USDC ATA balance via connection.getTokenAccountBalance).
   b. Amount input field with USDC formatting. Max button to fill wallet balance.
   c. On submit: build `deposit` instruction via Anchor program, amount as u64. Send transaction via wallet adapter's `sendTransaction`. Show loading spinner during confirmation.
   d. On success: display transaction signature with SolanaExplorerLink. Refetch CustomerBalance to show updated balance.
   e. Handle errors: insufficient USDC, transaction rejected, network errors.
2. Create `src/hooks/useDeposit.ts`: TanStack mutation hook wrapping the deposit instruction. Handles transaction building, sending, and confirmation.
3. Create `src/app/withdraw/page.tsx`:
   a. Show current CustomerBalance (via useCustomerBalance hook).
   b. Amount input with max button (max = current balance).
   c. On submit: build `withdraw` instruction, send via wallet adapter.
   d. On success: show tx signature with explorer link, refetch balance.
   e. Handle errors: InsufficientBalance, ProgramPaused, etc.
4. Create `src/hooks/useWithdraw.ts`: TanStack mutation hook for withdraw instruction.
5. Both pages: include back navigation, clear form after success, prevent double-submit.
</implementation_plan>

<validation>
On /deposit: enter 1 USDC, submit → Phantom popup appears, approve → transaction succeeds, explorer link shown, balance increases by 1 USDC. On /withdraw: enter 0.5 USDC, submit → transaction succeeds, balance decreases by 0.5. Entering amount greater than balance on withdraw shows appropriate error. Both pages display current balances correctly before and after transactions.
</validation>