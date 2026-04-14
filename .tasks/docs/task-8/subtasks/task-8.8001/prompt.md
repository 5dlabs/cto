<identity>
You are blaze working on subtask 8001 of task 8.
</identity>

<context>
<scope>
Create the SettlementFeed component using shadcn DataTable to display TaskReceipt records with columns for Task ID, Amount, Author Earned, Quality status (color-coded badges), Status, and timestamp. Fetch TaskReceipt PDAs from the on-chain program with polling or subscription.
</scope>
</context>

<implementation_plan>
1. Create `src/components/SettlementFeed.tsx` as a client component.
2. Define the DataTable column configuration using shadcn's column helper pattern:
   - `taskId`: string, truncated with tooltip for full ID.
   - `amount`: formatted via `formatUsdc()` from Task 7 types.
   - `authorEarned`: formatted via `formatUsdc()`.
   - `qualityMet`: render a shadcn `Badge` — green (`bg-solana-green/20 text-solana-green`) for `true`, red (`bg-red-500/20 text-red-500`) for `false`.
   - `status`: render as Badge with appropriate color per TaskStatus enum.
   - `settledAt` or `createdAt`: formatted via `formatTimestamp()`.
3. Use `useProgram()` hook to get the Anchor Program instance.
4. Fetch all TaskReceipt accounts using `program.account.taskReceipt.all()` or with filters if needed.
5. Implement a polling mechanism: `useEffect` with `setInterval` every 5 seconds to refresh the list, or use `connection.onProgramAccountChange` for real-time updates.
6. Store receipts in state, sorted by timestamp descending (newest first).
7. Handle empty state: show a message 'No settlements yet' with an icon.
8. Handle loading state: show a skeleton or spinner.
9. Handle wallet disconnected: show a prompt to connect wallet.
10. Make rows clickable — clicking a row should set a selectedReceipt state (used by the receipt verification subtask 8005). Use a callback prop or React context for cross-component communication.
11. Style the table with dark theme: `bg-solana-card` background, subtle row hover effects, solana-purple header accents.
</implementation_plan>

<validation>
Connect wallet on devnet. If TaskReceipt accounts exist on-chain, verify they render in the DataTable with correct columns and formatting. Verify quality badges show green for qualityMet=true and red for false. Verify the table is sorted newest-first. Verify empty state message shows when no receipts exist. Verify loading spinner appears during initial fetch. Verify polling refreshes data (create a new receipt on-chain and confirm it appears within the polling interval). Verify clicking a row highlights it or triggers selection callback.
</validation>