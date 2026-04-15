## Build Web Dashboard for Receipts, Balances & Agent Packages (Blaze - React/Next.js)

### Objective
Create a web dashboard that lets customers view their on-chain balance, browse task receipts with Arweave verification links, and explore registered agent packages with stats (total earned, success rate). This provides the visual verification layer that makes the hackathon demo compelling — a customer can see their billing history read directly from the Solana blockchain.

### Ownership
- Agent: blaze
- Stack: React/Next.js
- Priority: medium
- Status: pending
- Dependencies: 2, 3

### Implementation Details
1. Scaffold a Next.js 14 app in `app/dashboard/` using App Router:
   - `npx create-next-app@latest --typescript --tailwind --app --src-dir`
   - Dependencies: `@coral-xyz/anchor` (0.30.x), `@solana/web3.js` (^1.95), `@solana/wallet-adapter-react` (^0.15), `@solana/wallet-adapter-wallets` (^0.19), `@solana/wallet-adapter-react-ui` (^0.9), `@tanstack/react-query` (^5), `lucide-react`, `recharts` (for stats charts).
2. Set up Solana wallet connection:
   - `src/providers/SolanaProvider.tsx`: WalletProvider with Phantom, Solflare, Backpack adapters. Network toggle (devnet/mainnet).
   - `src/hooks/useProgram.ts`: Initialize Anchor program from IDL (import from `target/idl/cto_billing.json`). Derive all PDAs.
3. Implement pages:
   - **`/` (Dashboard Home):** If wallet connected, show CustomerBalance summary card (balance, total deposited, total spent, task count). If no customer account, show "Create Account" CTA. If not connected, show wallet connect prompt.
   - **`/receipts`:** List all TaskReceipts for the connected customer. Fetch via `getProgramAccounts` with filter on customer pubkey. Display table: task_id, amount (USDC formatted), author_earned, quality_met (✓/✗ icon), agent_package name, settled_at (formatted date), status badge (Settled/Refunded). Each row links to receipt detail.
   - **`/receipts/[taskId]`:** Receipt detail page. Show all TaskReceipt fields. "Verify on Solana" link to `explorer.solana.com/address/{pda}?cluster=devnet`. "View Itemized Receipt" link that fetches the Arweave URL (derived from receipt_hash or stored off-chain mapping), parses the JSON, and displays the billing line items. "Verify Hash" button that re-hashes the fetched Arweave receipt and compares to on-chain receipt_hash (green checkmark or red X).
   - **`/agents`:** Browse all registered AgentPackage accounts. Display card grid: package_id, author (truncated pubkey), split_bps as percentage, total_earned (USDC), task_count, success_rate (success_count/task_count * 100%), active badge, source_uri link. Sort by total_earned or success_rate.
   - **`/agents/[packageId]`:** Agent detail page with full stats, earnings chart (placeholder for future), and task history.
   - **`/deposit`:** Form to deposit USDC into CustomerBalance. Amount input with USDC balance shown. Calls `deposit` instruction via wallet adapter. Shows transaction confirmation with explorer link.
   - **`/withdraw`:** Form to withdraw USDC. Shows current balance. Calls `withdraw` instruction.
4. Shared components in `src/components/`:
   - `BalanceCard.tsx`, `ReceiptTable.tsx`, `AgentCard.tsx`, `QualityBadge.tsx` (green check / red X), `SolanaExplorerLink.tsx`, `UsdcAmount.tsx` (format u64 lamports to human-readable USDC).
   - `Header.tsx` with wallet connect button, network indicator, navigation.
5. Styling: Tailwind CSS with a dark theme (suitable for hackathon demo video). Use shadcn/ui components if desired for rapid development.
6. Environment config: `NEXT_PUBLIC_SOLANA_RPC_URL`, `NEXT_PUBLIC_PROGRAM_ID`, `NEXT_PUBLIC_SOLANA_NETWORK=devnet`.
7. Static export compatible: `next.config.js` with `output: 'export'` for easy deployment to Vercel or static hosting.

### Subtasks
- [ ] Scaffold Next.js 14 app with Solana wallet providers and program hooks: Initialize the Next.js 14 project with App Router, install all Solana and UI dependencies, configure SolanaProvider with wallet adapters, create the useProgram hook for Anchor IDL integration, implement PDA derivation utilities, and set up environment configuration.
- [ ] Build shared UI components library and dark theme Tailwind configuration: Create all reusable UI components (Header, BalanceCard, ReceiptTable, AgentCard, QualityBadge, SolanaExplorerLink, UsdcAmount) and configure the dark theme Tailwind styling for hackathon demo presentation quality.
- [ ] Implement dashboard home page with balance summary and wallet state handling: Build the root `/` dashboard page that shows CustomerBalance summary when wallet is connected, a create account CTA when no customer account exists, and a wallet connect prompt when disconnected.
- [ ] Implement deposit and withdraw pages with wallet transaction submission: Build the /deposit and /withdraw pages that allow users to deposit USDC into their CustomerBalance escrow and withdraw USDC back to their wallet, submitting Solana transactions via the wallet adapter.
- [ ] Implement receipts list page with on-chain data fetching and filtering: Build the /receipts page that lists all TaskReceipt accounts for the connected customer wallet, fetched via getProgramAccounts with memcmp filter on the customer pubkey field.
- [ ] Implement receipt detail page with Arweave verification and hash checking: Build the /receipts/[taskId] detail page that displays all TaskReceipt fields, provides Solana explorer links, fetches the Arweave receipt JSON, and implements the hash verification button that re-hashes the fetched receipt to compare against the on-chain receipt_hash.
- [ ] Implement agent packages list and detail pages with stats display: Build the /agents page showing a card grid of all registered AgentPackage accounts with sorting, and the /agents/[packageId] detail page showing full stats and task history.