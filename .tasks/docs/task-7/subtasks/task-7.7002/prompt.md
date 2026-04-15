<identity>
You are blaze working on subtask 7002 of task 7.
</identity>

<context>
<scope>
Create all reusable UI components (Header, BalanceCard, ReceiptTable, AgentCard, QualityBadge, SolanaExplorerLink, UsdcAmount) and configure the dark theme Tailwind styling for hackathon demo presentation quality.
</scope>
</context>

<implementation_plan>
1. Configure `tailwind.config.ts` with dark theme as default: dark color palette suitable for hackathon demo (deep backgrounds, vibrant accents). Set up custom color tokens.
2. Create `src/components/Header.tsx`: wallet connect button (using @solana/wallet-adapter-react-ui WalletMultiButton), network indicator badge (devnet/mainnet), navigation links to /, /receipts, /agents, /deposit, /withdraw. Responsive layout.
3. Create `src/components/BalanceCard.tsx`: displays balance, total_deposited, total_spent, task_count. Props accept CustomerBalance account data. Formats USDC amounts.
4. Create `src/components/ReceiptTable.tsx`: table component accepting array of TaskReceipt data. Columns: task_id, amount (USDC), author_earned, quality_met, agent_package, settled_at, status. Each row clickable linking to detail. Supports empty state.
5. Create `src/components/AgentCard.tsx`: card displaying package_id, author (truncated pubkey with copy button), split_bps as %, total_earned, task_count, success_rate, active badge, source_uri link.
6. Create `src/components/QualityBadge.tsx`: green checkmark icon for quality_met=true, red X for false. Uses lucide-react icons.
7. Create `src/components/SolanaExplorerLink.tsx`: generates explorer.solana.com URL for a given address/tx with correct cluster param. Opens in new tab.
8. Create `src/components/UsdcAmount.tsx`: formats u64 lamport-based amounts to human-readable USDC (divide by 10^6, add $ prefix, 2 decimal places).
9. Add global styles in `src/app/globals.css` for consistent dark theme baseline.
</implementation_plan>

<validation>
Each component renders without errors when imported into a test page with mock data. BalanceCard displays '100.00 USDC' when passed balance=100_000_000. UsdcAmount formats 1_500_000 as '$1.50'. QualityBadge shows green check for true and red X for false. SolanaExplorerLink generates correct URL with devnet cluster. Header shows wallet connect button and all navigation links.
</validation>