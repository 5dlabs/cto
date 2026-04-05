Implement subtask 8012: Accessibility audit and WCAG 2.1 AA compliance verification

## Objective
Run comprehensive accessibility audit across all pages using axe-core, verify WCAG 2.1 AA compliance including keyboard navigation, screen reader labels, color contrast, and focus management. Fix any violations found.

## Steps
1. Install `@axe-core/react` (dev) and `axe-playwright` for automated testing.
2. Run axe-core audit on every page: /, /equipment, /equipment/:id, /quote, /portfolio.
3. Run axe-core on ChatWidget in open state.
4. Check and fix:
   - Color contrast: all text meets 4.5:1 ratio against dark backgrounds (especially on dark/moody theme).
   - Keyboard navigation: Tab through all interactive elements on each page, verify logical order, visible focus indicators.
   - Screen reader: all images have meaningful alt text, form inputs have labels, buttons have accessible names, dynamic content has aria-live regions.
   - Focus management: when ChatWidget opens, focus moves to input. When Dialog opens, focus traps inside. When closed, focus returns to trigger.
   - Skip navigation: 'Skip to main content' link at top of page.
5. Test with screen reader (VoiceOver or NVDA instructions for manual verification).
6. Fix all critical and serious axe violations. Document any minor/moderate ones as known issues.
7. Add `aria-label` to icon-only buttons (chat trigger, close buttons, filter pills).
8. Ensure responsive zoom: site usable at 200% zoom without horizontal scroll.

## Validation
Run automated axe-core scan on all pages — zero critical and zero serious violations. Verify skip navigation link present and functional. Tab through home page and equipment page — verify all interactive elements reachable and focus visible. Verify color contrast tool reports all text meets 4.5:1 ratio. Verify ChatWidget has correct focus trap behavior.