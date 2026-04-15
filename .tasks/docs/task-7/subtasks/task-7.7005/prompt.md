<identity>
You are blaze working on subtask 7005 of task 7.
</identity>

<context>
<scope>
Build the /receipts page that lists all TaskReceipt accounts for the connected customer wallet, fetched via getProgramAccounts with memcmp filter on the customer pubkey field.
</scope>
</context>

<implementation_plan>
1. Create `src/app/receipts/page.tsx`:
   a. Require wallet connection (redirect or prompt if disconnected).
   b. Fetch all TaskReceipt accounts where customer field matches connected wallet pubkey.
2. Create `src/hooks/useCustomerReceipts.ts`: TanStack Query hook that calls `program.account.taskReceipt.all([{ memcmp: { offset: 8, bytes: wallet.publicKey.toBase58() } }])`. Note: verify the correct offset for the customer field in the TaskReceipt account layout by checking the IDL discriminator + field ordering.
3. Render ReceiptTable component with fetched receipts. Columns: task_id, amount (via UsdcAmount), author_earned, quality_met (QualityBadge), agent_package name (truncated), settled_at (formatted date from Unix timestamp), status badge (Settled green / Refunded yellow).
4. Each row is a clickable link to `/receipts/[taskId]`.
5. Add sort controls: sort by settled_at (newest first default), amount, task_id.
6. Handle empty state: 'No receipts found. Complete a task to see billing receipts here.'
7. Handle loading state with skeleton rows.
8. Show total count of receipts and sum of amounts at the top.
</implementation_plan>

<validation>
Connect wallet that has at least 2 TaskReceipt accounts (created via CLI demo flow). /receipts page shows both receipts in a table with correct task_id, amount, quality_met status. Sorting by amount works. Clicking a row navigates to /receipts/[taskId]. Empty state shows when wallet has no receipts. Receipt count and total amount sum are accurate.
</validation>