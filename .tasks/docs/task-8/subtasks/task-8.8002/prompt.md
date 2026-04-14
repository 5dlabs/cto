<identity>
You are blaze working on subtask 8002 of task 8.
</identity>

<context>
<scope>
Create the CustomerBalance card displaying current balance, total deposited, total spent, task count, and spending caps (max_per_task, max_per_day, daily_spent) with visual indicators and progress bars.
</scope>
</context>

<implementation_plan>
1. Create `src/components/CustomerBalanceCard.tsx` as a client component.
2. Use the `useCustomerBalance(wallet.publicKey)` hook from Task 7 to fetch balance data.
3. Layout using shadcn `Card` with `bg-solana-card` background:
   - Header: 'Your Balance' title with a wallet icon.
   - Primary display: large formatted USDC balance (e.g., '150.00 USDC') in solana-green if positive, muted if zero.
   - Stats grid (2x2 or 2x3): `Total Deposited`, `Total Spent`, `Task Count` each in smaller cards or stat blocks.
4. Spending caps section:
   - 'Max Per Task' with the cap value.
   - 'Daily Spending' with a progress bar showing `dailySpent / maxPerDay`. Use a shadcn-compatible progress bar or a custom Tailwind bar. Color green when under 70%, yellow 70-90%, red over 90%.
   - Show 'Resets at [time]' based on `lastDayReset` + 24h.
5. Handle null/empty state: if the CustomerBalance PDA doesn't exist, show 'No account found — deposit to get started' with a call-to-action.
6. Handle loading state with skeleton placeholders matching the card layout.
7. Add a subtle pulse animation on the balance amount when it updates (use CSS transition).
8. Make the card responsive: full width on mobile, fit within the 40% right panel on desktop.
</implementation_plan>

<validation>
Connect wallet on devnet. If CustomerBalance PDA exists, verify the card shows correct balance, total deposited, total spent, and task count values matching on-chain data. Verify formatUsdc renders amounts correctly. Verify spending cap progress bar reflects dailySpent/maxPerDay ratio. Verify color changes at 70% and 90% thresholds. Verify 'No account found' state renders when PDA doesn't exist. Verify loading skeletons appear during fetch. Test on 1920×1080 and 375px viewports.
</validation>