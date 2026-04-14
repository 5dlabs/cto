## Hackathon Demo Web Dashboard - Settlement Feed & Balance Management (Blaze - React/Next.js/TypeScript)

### Objective
Implement the settlement feed DataTable, customer balance card with deposit/withdraw flows, agent package listing, and receipt verification — the interactive features that make the dashboard demo-worthy for judges.

### Ownership
- Agent: blaze
- Stack: React/Next.js 14/TypeScript/shadcn-ui
- Priority: medium
- Status: pending
- Dependencies: 7

### Implementation Details
1. **Settlement Feed (Left Panel)** — `src/components/SettlementFeed.tsx`:
   - Use shadcn `DataTable` component with columns: Task ID

### Subtasks
- [ ] Build Settlement Feed DataTable component: Create the SettlementFeed component using shadcn DataTable to display TaskReceipt records with columns for Task ID, Amount, Author Earned, Quality status (color-coded badges), Status, and timestamp. Fetch TaskReceipt PDAs from the on-chain program with polling or subscription.
- [ ] Build Customer Balance Card component: Create the CustomerBalance card displaying current balance, total deposited, total spent, task count, and spending caps (max_per_task, max_per_day, daily_spent) with visual indicators and progress bars.
- [ ] Implement Deposit and Withdraw transaction flows: Create dialog/modal forms for depositing and withdrawing USDC, submit transactions via the Anchor program, show toast notifications for success/failure, and refresh the balance card after completion.
- [ ] Build Agent Package listing component: Create the AgentPackage listing component that fetches and displays all registered agent packages with author address, split percentage, total earned, computed success rate, and active status badge.
- [ ] Build Receipt verification detail view: Create the receipt detail view that expands when a TaskReceipt row is selected in the Settlement Feed, showing the full receipt hash, on-chain Explorer link, quality_met explanation, all amounts, and transaction metadata.