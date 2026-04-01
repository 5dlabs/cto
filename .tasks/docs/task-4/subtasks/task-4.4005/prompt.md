Implement subtask 4005: Responsive layout implementation for 320px to 1920px viewports

## Objective
Ensure the entire design-snapshots page, including the PR card grid and the diff viewer, is fully responsive across viewports from 320px to 1920px with no horizontal overflow or layout breakage.

## Steps
1. **Card grid breakpoints** (verify/refine from subtask 4002):
   - 320px–639px: single column, cards full width with appropriate padding (min 16px side padding).
   - 640px–1023px: 2-column grid.
   - 1024px–1920px: 3-column grid.
   - Ensure card content (especially long PR titles) wraps or truncates with ellipsis and a title tooltip.
2. **Diff viewer responsiveness**:
   - On viewports <768px, force inline view mode (side-by-side is unusable) and hide the split-view toggle or disable it with a tooltip explaining why.
   - Wrap the diff viewer in `overflow-x-auto` so wide lines scroll horizontally within the container.
   - File section headers should be sticky on scroll.
3. **Page-level layout**: Ensure the page header, breadcrumbs (if any), and content area have appropriate max-width (e.g., `max-w-7xl mx-auto`) and side padding.
4. Test that at 320px, no element causes horizontal page scroll (use `overflow-x: hidden` on body as a safeguard is NOT acceptable — fix the actual overflow).

## Validation
Visual regression: capture screenshots at 320px, 480px, 768px, 1024px, 1440px, and 1920px widths for both the list view and the diff viewer. Verify no horizontal overflow at any width. Verify card grid column count matches spec at each breakpoint. Verify diff viewer forces inline mode below 768px.