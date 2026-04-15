<identity>
You are blaze working on subtask 7003 of task 7.
</identity>

<context>
<scope>
Build the root `/` dashboard page that shows CustomerBalance summary when wallet is connected, a create account CTA when no customer account exists, and a wallet connect prompt when disconnected.
</scope>
</context>

<implementation_plan>
1. Create `src/app/page.tsx` as the dashboard home.
2. Use `useWallet()` hook to detect connection state. Three states:
   a. **Not connected**: Render centered wallet connect prompt with WalletMultiButton and explanatory text.
   b. **Connected, no CustomerBalance PDA**: Derive customer PDA from wallet pubkey, attempt to fetch. If not found, render 'Create Account' CTA card with instructions to deposit first.
   c. **Connected, account exists**: Fetch CustomerBalance account data via useProgram. Render BalanceCard with balance, total_deposited, total_spent, task_count.
3. Create `src/hooks/useCustomerBalance.ts`: TanStack Query hook that fetches CustomerBalance account. Derives PDA from connected wallet pubkey. Returns { data, isLoading, error, refetch }. Refetches on wallet change.
4. Add quick-action cards below balance: 'View Receipts' → /receipts, 'Browse Agents' → /agents, 'Deposit USDC' → /deposit.
5. Handle loading states with skeleton UI. Handle errors with user-friendly messages.
6. Ensure the page works with static export (all data fetched client-side).
</implementation_plan>

<validation>
With no wallet connected, page shows wallet connect prompt. Connect Phantom wallet on devnet: if no customer account exists, 'Create Account' CTA appears. If customer account exists (after deposit via CLI), BalanceCard displays correct balance matching on-chain state queried via `solana account` CLI. Quick action cards link to correct routes. Page loads without console errors.
</validation>