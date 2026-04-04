Implement subtask 6006: Write component tests and accessibility tests

## Objective
Write comprehensive component tests for TaskCard, TaskList, and PipelineSummary, plus accessibility tests verifying WCAG 2.1 AA compliance using axe-core.

## Steps
1. Set up testing with @testing-library/react, @testing-library/jest-dom, and jest-axe (or vitest-axe if using Vitest).
2. TaskCard tests: (a) assigned agent renders green badge with name, (b) unresolved agent renders amber 'Unresolved' badge, (c) all metadata fields (title, agent, stack, priority, status) are visible.
3. TaskList tests: (a) 5 tasks render in dependency order, (b) dependency indicators are present for tasks with dependencies.
4. PipelineSummary tests: (a) correct counts for mixed assigned/unresolved tasks, (b) pipeline status badge shows correct variant.
5. Accessibility tests using jest-axe: (a) run axe on rendered TaskCard — expect no violations, (b) run axe on rendered TaskList — expect no violations, (c) verify all interactive elements are keyboard-focusable, (d) verify Avatar has appropriate alt text or aria-label.
6. Ensure all tests pass in CI with `npm test`.

## Validation
All component tests pass. Axe accessibility audit on TaskCard, TaskList, and PipelineSummary returns zero violations at WCAG 2.1 AA level. Keyboard navigation test confirms all interactive elements are reachable via Tab key.