Implement subtask 7006: Write comprehensive component and accessibility tests

## Objective
Write component tests covering all memo display states (present/absent, expanded/collapsed), markdown rendering, timestamp formatting, summary count, and keyboard accessibility of the collapsible section.

## Steps
1. Create test file `__tests__/research-memo.test.tsx` (or co-located test file). 2. Test cases: (a) TaskCard with non-null research_memo shows Research badge, click expands to show content/source/timestamp. (b) TaskCard with null research_memo does not render badge or collapsible. (c) Markdown content with headers, links, code blocks renders correctly. (d) Timestamp renders as relative time. (e) Summary header shows correct 'N of M' count. (f) Accessibility: collapsible toggle responds to Enter and Space keydown events; aria-expanded is true when open, false when closed. 3. Use React Testing Library with userEvent for interactions. 4. Mock date for deterministic timestamp tests.

## Validation
All 6 test cases pass. Keyboard accessibility tests use fireEvent.keyDown with Enter and Space and verify aria-expanded attribute toggles. Coverage report shows ResearchMemo component and summary count logic covered.