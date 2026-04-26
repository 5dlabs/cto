# Codex UI Catalog - 005 Top Bar + Style Notes

- Captured: 2026-03-08 09:39:54 PDT
- Scope: top area of current Codex thread view
- Screenshots:
  - `../screenshots/005-skills-topbar-full.png`
  - `../screenshots/005-topbar-focus.png`

## Top Bar Functionality (Current View)

1. Left title area:
   - Active thread title is shown in the header.
   - Workspace/context tag appears adjacent to the title.
2. Right action cluster:
   - `Open` button with dropdown affordance.
   - `Hand off` button (disabled in this captured state).
   - `Commit` button with dropdown affordance.
3. Status indicators:
   - Positive/negative counters on the far right (green/red).
   - Additional icon buttons for secondary actions.

## Top Bar Interaction Model

1. Primary actions are grouped to the top-right and use pill-shaped controls.
2. Disabled controls remain visible to preserve layout and discoverability.
3. Header stays persistent while conversation content scrolls underneath.

## Visual Style and Gradient Notes

Observed from sampled pixels in this capture:

- Main content background: `#181818` (near-uniform dark field)
- Left navigation background: `#212f37` (blue-gray dark panel)
- Surface contrast is mostly achieved via opacity/edge lines, not large hue shifts.
- Visual style is effectively a low-contrast dark theme with subtle panel separation.

### Practical style summary

1. Main canvas uses a flat dark base rather than a dramatic multicolor gradient.
2. Sidebar appears as a cooler-toned dark slab against the warmer neutral content area.
3. Elevation cues come from rounded outlines, subtle borders, and muted glow/opacity, not shadows.

