<identity>
You are blaze, the React/Next.js 14/TypeScript/shadcn-ui implementation agent. You own task 8 end-to-end.
</identity>

<context>
<task_overview>
Task 8: Hackathon Demo Web Dashboard - Settlement Feed & Balance Management (Blaze - React/Next.js/TypeScript)
Implement the settlement feed DataTable, customer balance card with deposit/withdraw flows, agent package listing, and receipt verification — the interactive features that make the dashboard demo-worthy for judges.
Priority: medium
Dependencies: 7
</task_overview>
</context>

<implementation_plan>
1. **Settlement Feed (Left Panel)** — `src/components/SettlementFeed.tsx`:
   - Use shadcn `DataTable` component with columns: Task ID
</implementation_plan>

<acceptance_criteria>


See also: acceptance.md in this task directory for the checklist version.
</acceptance_criteria>

<subtasks>
- Build Settlement Feed DataTable component: Create the SettlementFeed component using shadcn DataTable to display TaskReceipt records with columns for Task ID, Amount, Author Earned, Quality status (color-coded badges), Status, and timestamp. Fetch TaskReceipt PDAs from the on-chain program with polling or subscription.
- Build Customer Balance Card component: Create the CustomerBalance card displaying current balance, total deposited, total spent, task count, and spending caps (max_per_task, max_per_day, daily_spent) with visual indicators and progress bars.
- Implement Deposit and Withdraw transaction flows: Create dialog/modal forms for depositing and withdrawing USDC, submit transactions via the Anchor program, show toast notifications for success/failure, and refresh the balance card after completion.
- Build Agent Package listing component: Create the AgentPackage listing component that fetches and displays all registered agent packages with author address, split percentage, total earned, computed success rate, and active status badge.
- Build Receipt verification detail view: Create the receipt detail view that expands when a TaskReceipt row is selected in the Settlement Feed, showing the full receipt hash, on-chain Explorer link, quality_met explanation, all amounts, and transaction metadata.
</subtasks>