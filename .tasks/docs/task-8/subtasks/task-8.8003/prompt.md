<identity>
You are blaze working on subtask 8003 of task 8.
</identity>

<context>
<scope>
Create dialog/modal forms for depositing and withdrawing USDC, submit transactions via the Anchor program, show toast notifications for success/failure, and refresh the balance card after completion.
</scope>
</context>

<implementation_plan>
1. Create `src/components/DepositDialog.tsx` as a client component:
   - Trigger: a 'Deposit' button on the CustomerBalanceCard (or below it).
   - Use shadcn `Dialog` for the modal with a form containing:
     - USDC amount input (shadcn `Input`, type number, min 0, step 0.01).
     - Display the user's current USDC token balance (fetch via `getTokenAccountsByOwner` or `getAssociatedTokenAddress` + `getTokenAccountBalance`).
     - 'Deposit' submit button styled with solana-purple.
   - On submit: build and send the `deposit` instruction via `useProgram()`. Include the amount converted to lamports (multiply by 10^6 for USDC).
   - Show a loading spinner on the button during transaction.
   - On success: show a shadcn `toast` with 'Deposit successful' in green. Close the dialog. Call the balance refetch function.
   - On failure: show a toast with the error message in red. Keep dialog open.
2. Create `src/components/WithdrawDialog.tsx` similarly:
   - Trigger: a 'Withdraw' button on the CustomerBalanceCard.
   - Same dialog pattern with amount input.
   - Show current on-chain balance as max withdrawable.
   - Validate amount ≤ current balance before submission.
   - Build and send the `withdraw` instruction.
   - Same success/failure toast and refetch pattern.
3. For both dialogs, use `useWallet()` to get the `sendTransaction` or use Anchor's `program.methods.deposit(...).accounts({...}).rpc()` pattern.
4. Add the Deposit and Withdraw buttons to the CustomerBalanceCard component — either as props/callbacks or import directly.
5. Ensure proper error handling for: wallet not connected, insufficient SOL for fees, insufficient USDC balance, program errors (e.g., paused).
</implementation_plan>

<validation>
Connect Phantom wallet on devnet with test USDC tokens. Open Deposit dialog, enter an amount, and submit. Verify the transaction is sent (check Solana Explorer), the success toast appears, the dialog closes, and the balance card refreshes to show the new balance. Open Withdraw dialog, enter an amount within balance, submit, and verify same flow. Test error cases: enter amount exceeding USDC wallet balance for deposit — verify error toast. Enter amount exceeding on-chain balance for withdraw — verify client-side validation prevents submission. Test with wallet disconnected — verify buttons are disabled or show appropriate message.
</validation>