Implement task 7: Add Research Memo Display to Web Frontend (Blaze - React/Next.js)

## Goal
Extend the pipeline dashboard to display Hermes research memos associated with each task. Memos are shown as collapsible sections within task cards, displaying the content, source, and timestamp. Contingent on D5 resolution.

## Task Context
- Agent owner: blaze
- Stack: React/Next.js
- Priority: medium
- Dependencies: 3, 6

## Implementation Plan
1. Extend the `TaskCard` component to include a collapsible `ResearchMemo` section using shadcn/ui Collapsible (or Accordion) component.
2. When `research_memo` is non-null on a task, display a 'Research' indicator badge on the task card. Clicking it expands the collapsible section.
3. The expanded section shows: `content` rendered as markdown (use a lightweight markdown renderer like react-markdown), `source` displayed as a subtle metadata line, `timestamp` formatted as relative time (e.g., '2 hours ago').
4. When `research_memo` is null, display a muted 'No research data' text or hide the section entirely.
5. Update the summary header to include a research memo count: 'N of M tasks have research memos'.
6. Ensure the collapsible interaction is keyboard-accessible (Enter/Space to toggle) via Radix primitives.
7. Write component tests for: memo present (expanded and collapsed states), memo absent, markdown rendering, summary count.

## Acceptance Criteria
1. Component test: TaskCard with a non-null `research_memo` shows a 'Research' badge; clicking it reveals content, source, and timestamp. 2. Component test: TaskCard with `research_memo=null` does not render the Research badge or collapsible section. 3. Component test: Markdown content in research memo (e.g., headers, links, code blocks) renders correctly via react-markdown. 4. Component test: Timestamp '2024-01-15T10:30:00Z' renders as a human-readable relative time string. 5. Component test: Summary header with 3 of 5 tasks having memos displays '3 of 5 tasks have research memos'. 6. Accessibility test: Collapsible section toggles with Enter and Space keys; expanded state is announced via aria-expanded.

## Subtasks
- Create ResearchMemo collapsible component with shadcn/ui: Build a self-contained ResearchMemo component using shadcn/ui Collapsible (Radix-based) that accepts research_memo data as props and renders a collapsible section with proper aria-expanded state management.
- Integrate react-markdown for memo content rendering: Install react-markdown and integrate it into the ResearchMemo component to render the `content` field as formatted markdown, including headers, links, and code blocks.
- Add relative timestamp formatting for memo timestamp: Format the research memo timestamp as a human-readable relative time string (e.g., '2 hours ago') using a lightweight date utility.
- Integrate ResearchMemo into TaskCard component: Extend the existing TaskCard component to conditionally render the ResearchMemo collapsible section when research_memo data is present on a task, and show a muted 'No research data' text when absent.
- Add research memo count to summary header: Update the pipeline dashboard summary header to display a count of how many tasks have research memos out of the total, e.g., '3 of 5 tasks have research memos'.
- Write comprehensive component and accessibility tests: Write component tests covering all memo display states (present/absent, expanded/collapsed), markdown rendering, timestamp formatting, summary count, and keyboard accessibility of the collapsible section.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.