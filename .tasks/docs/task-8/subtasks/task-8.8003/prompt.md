Implement subtask 8003: Write component and accessibility tests for SnapshotPR

## Objective
Write comprehensive component tests covering all PR display states (open, merged, closed, null) and accessibility requirements for external links.

## Steps
1. Create test file `__tests__/snapshot-pr.test.tsx`. 2. Test cases: (a) Valid PR with status='open' displays title, URL, green 'Open' badge, file count, branch. (b) Status='merged' displays purple 'Merged' badge. (c) Status='closed' displays red 'Closed' badge. (d) prResult=null displays 'No snapshot PR created'. (e) PR URL link element has `target='_blank'` and `rel='noopener noreferrer'` attributes. (f) Accessibility: PR link has appropriate aria-label or visually hidden text indicating external link for screen readers; verify ExternalLink icon is present. 3. Use React Testing Library. 4. Use getByRole, getByText for semantic queries.

## Validation
All 6 test cases pass. Link attributes verified via `expect(link).toHaveAttribute('target', '_blank')` and `expect(link).toHaveAttribute('rel', 'noopener noreferrer')`. Screen reader test verifies aria-label or sr-only text on external link.