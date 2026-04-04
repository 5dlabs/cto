Implement subtask 4005: Implement accessibility compliance across all design PR components

## Objective
Audit and enhance the DesignSnapshotPRList and DesignSnapshotPRDetail components for full accessibility: semantic HTML, ARIA labels, keyboard navigation, and focus management.

## Steps
1. Ensure all interactive elements (links, buttons, expandable rows) are reachable via keyboard Tab navigation and activatable via Enter/Space.
2. Add `aria-label` attributes to:
   - Each PR card/row (e.g., `aria-label="Pull request: {title}, status: {status}"`)
   - The GitHub link button (e.g., `aria-label="Open PR {title} on GitHub"`)
   - The Retry button (e.g., `aria-label="Retry loading design PRs"`)
3. Use semantic HTML: `<nav>` for navigation, `<main>` for content, `<table>` or `<ul>` for lists, `<h2>`/`<h3>` for section headings with proper hierarchy.
4. Status badges should not rely solely on color — include text labels (the badge already shows text like 'open', 'merged', 'closed').
5. Ensure focus is managed when transitioning from list to detail view (focus moves to the detail heading).
6. Run axe-core programmatically on each rendered component state (populated, empty, error) and resolve all violations.

## Validation
Run axe-core (via jest-axe or @axe-core/react) on: (1) DesignSnapshotPRList in populated state — zero violations. (2) DesignSnapshotPRList in empty state — zero violations. (3) DesignSnapshotPRList in error state — zero violations. (4) DesignSnapshotPRDetail in populated state — zero violations. (5) Manual verification: Tab through all interactive elements in sequence without getting stuck.