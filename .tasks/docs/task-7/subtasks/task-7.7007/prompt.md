<identity>
You are blaze working on subtask 7007 of task 7.
</identity>

<context>
<scope>
Build the /agents page showing a card grid of all registered AgentPackage accounts with sorting, and the /agents/[packageId] detail page showing full stats and task history.
</scope>
</context>

<implementation_plan>
1. Create `src/app/agents/page.tsx`:
   a. Fetch all AgentPackage accounts via `program.account.agentPackage.all()`.
   b. Render card grid using AgentCard component for each package.
   c. Each card shows: package_id, author (truncated with copy), split_bps as percentage (e.g., '30%'), total_earned (UsdcAmount), task_count, success_rate (success_count/task_count * 100, or 'N/A' if task_count=0), active badge (green/gray), source_uri as external link.
   d. Add sort controls: sort by total_earned (default, descending), success_rate, task_count.
   e. Add active/inactive filter toggle.
   f. Each card links to `/agents/[packageId]`.
2. Create `src/hooks/useAgentPackages.ts`: TanStack Query hook fetching all AgentPackage accounts.
3. Create `src/app/agents/[packageId]/page.tsx`:
   a. Derive AgentPackage PDA from packageId. Fetch account data.
   b. Display all fields in full detail: package_id, author (full pubkey with copy), split_bps, total_earned, task_count, success_count, success_rate, active status, source_uri.
   c. Add a placeholder 'Earnings Over Time' chart section (recharts AreaChart with placeholder data and 'Coming Soon' label).
   d. Add task history section: fetch TaskReceipts where agent_package field matches this package PDA. Display in ReceiptTable.
4. Create `src/hooks/useAgentPackage.ts`: TanStack Query hook for single AgentPackage account.
5. Handle empty state on agents list: 'No agent packages registered yet.'
6. Handle package not found on detail page with 404.
</implementation_plan>

<validation>
Connect wallet, navigate to /agents. At least one AgentPackage card appears with correct stats (matching on-chain data). Success rate calculates correctly (e.g., 1 success out of 2 tasks = 50%). Sorting by total_earned reorders cards. Clicking a card navigates to /agents/[packageId] with full detail view. Placeholder earnings chart renders without errors. Task history section shows receipts associated with the agent package.
</validation>