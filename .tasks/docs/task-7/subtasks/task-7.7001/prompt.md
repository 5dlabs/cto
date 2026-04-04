Implement subtask 7001: Create ResearchMemo collapsible component with shadcn/ui

## Objective
Build a self-contained ResearchMemo component using shadcn/ui Collapsible (Radix-based) that accepts research_memo data as props and renders a collapsible section with proper aria-expanded state management.

## Steps
1. Create `components/research-memo.tsx`. 2. Import and use shadcn/ui Collapsible (CollapsibleTrigger + CollapsibleContent) which wraps Radix primitives. 3. Accept props: `researchMemo: { content: string; source: string; timestamp: string } | null`. 4. When `researchMemo` is non-null, render a 'Research' Badge (shadcn/ui Badge) as the CollapsibleTrigger. 5. Manage open/closed state via React useState. 6. When `researchMemo` is null, render nothing (return null) — the null-state text will be handled at the TaskCard level if needed. 7. Ensure CollapsibleTrigger responds to Enter and Space keys natively via Radix primitives. 8. Export the component for use in TaskCard.

## Validation
Render ResearchMemo with non-null props; verify Badge is visible, clicking toggles CollapsibleContent visibility, aria-expanded toggles between true/false. Render with null props; verify nothing is rendered.