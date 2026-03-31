Implement subtask 4009: Accessibility audit and remediation for all Hermes components

## Objective
Run axe-core accessibility audits on all new Hermes pages and components, then fix any critical or serious violations to meet WCAG 2.1 AA compliance.

## Steps
1. Set up axe-core testing:
   - Install `@axe-core/react` or `axe-playwright` for automated accessibility testing
   - Configure to run against `/hermes` and `/hermes/[id]` pages with mock data
2. Audit each component/page:
   - `EnvironmentBanner`: verify text label is present (not color-only), sufficient color contrast ratio (4.5:1 for text)
   - `/hermes` dashboard: verify Card components are keyboard navigable, status badges have accessible names
   - `/hermes/[id]` detail page: verify tab navigation works with keyboard (Tab, Arrow keys), tab panels are properly associated with ARIA attributes
   - `ArtifactComparison`: all images have descriptive `alt` text (e.g., 'Current site screenshot of {url}'), thumbnail selector is keyboard navigable
   - `ArtifactViewer` Dialog: focus management verified (focus trapped, returns to trigger on close), Escape key closes
3. Fix all critical and serious violations found.
4. Specific checks:
   - All interactive elements have visible focus indicators
   - Tab order follows logical reading order
   - Color is not the only means of conveying information (status badges have text labels)
   - Images that are decorative have `alt=""`, informative images have descriptive alt text
5. Document any accepted minor violations with justification.

## Validation
Run axe-core audit on `/hermes` page: zero critical or serious violations. Run axe-core on `/hermes/[id]` page: zero critical or serious violations. Manual keyboard test: Tab through all interactive elements on both pages, verify logical tab order and visible focus indicators. Screen reader test: verify all images have appropriate alt text, status badges announce their text, and dialog focus management works. Color contrast: verify all text meets 4.5:1 contrast ratio.