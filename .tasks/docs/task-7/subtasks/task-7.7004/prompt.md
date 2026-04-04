Implement subtask 7004: Integrate ResearchMemo into TaskCard component

## Objective
Extend the existing TaskCard component to conditionally render the ResearchMemo collapsible section when research_memo data is present on a task, and show a muted 'No research data' text when absent.

## Steps
1. In `TaskCard` component, import ResearchMemo. 2. Check if `task.research_memo` is non-null: if yes, render `<ResearchMemo researchMemo={task.research_memo} />` within the card body. 3. If `task.research_memo` is null, optionally render `<p className='text-xs text-muted-foreground'>No research data</p>` or hide the section entirely based on the design preference (start with hiding entirely for a cleaner look). 4. Ensure the ResearchMemo section is positioned after the existing task details but before any action buttons.

## Validation
Component test: render TaskCard with a task that has research_memo set; verify 'Research' badge appears and collapsible section is present. Render TaskCard with research_memo=null; verify no Research badge or collapsible is rendered.