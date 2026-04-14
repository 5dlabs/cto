<identity>
You are blaze working on subtask 8004 of task 8.
</identity>

<context>
<scope>
Create the AgentPackage listing component that fetches and displays all registered agent packages with author address, split percentage, total earned, computed success rate, and active status badge.
</scope>
</context>

<implementation_plan>
1. Create `src/components/AgentPackageList.tsx` as a client component.
2. Use `useProgram()` to fetch all AgentPackage accounts: `program.account.agentPackage.all()`.
3. Display as a list of shadcn `Card` components or as a secondary DataTable. Each item shows:
   - `packageId`: displayed as a title or heading.
   - `author`: shortened PublicKey via `shortenAddress()` with copy-to-clipboard on click.
   - `splitBps`: displayed as percentage (e.g., `splitBps / 100`% → '15%').
   - `totalEarned`: formatted via `formatUsdc()`.
   - Success rate: computed as `(successCount / taskCount * 100).toFixed(1)%`. Handle zero taskCount (show 'N/A' or '—').
   - `active`: shadcn Badge — green 'Active' or gray 'Inactive'.
4. Sort packages by totalEarned descending by default.
5. Handle empty state: 'No agent packages registered yet'.
6. Handle loading state with card skeletons.
7. Place this component either in a separate tab within the right panel (using shadcn `Tabs` alongside the balance card) or below the balance card in the right panel — whichever fits better within the 40% panel at 1920×1080.
8. Style with dark theme: bg-solana-card cards, subtle borders, solana-purple accents for headings.
</implementation_plan>

<validation>
Connect wallet on devnet. If AgentPackage accounts exist, verify they render with correct packageId, shortened author address, split percentage, total earned, success rate, and active badge. Verify success rate calculation: for a package with taskCount=10 and successCount=8, verify '80.0%' is displayed. Verify 'N/A' shows for packages with zero taskCount. Verify copy-to-clipboard works on author address. Verify empty state message when no packages exist. Verify sorting by totalEarned descending.
</validation>