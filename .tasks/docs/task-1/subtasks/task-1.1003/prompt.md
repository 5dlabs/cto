Implement subtask 1003: Implement layout shell with sticky header, main content slot, and footer

## Objective
Build `app/layout.tsx` with a sticky header containing logo and navigation, a `<main>` content slot, and a semantic `<footer>`. Use semantic HTML landmarks throughout. Configure font loading via next/font.

## Steps
1. Implement `app/layout.tsx` as the root layout.
2. Configure font loading based on decision point resolution (default: `next/font/google` with Inter or similar sans-serif). Apply the font class to `<html>` or `<body>`.
3. Add a `<header>` element with `className="sticky top-0 z-50"` positioning. Include a text logo placeholder and a horizontal `<nav>` with 3-4 placeholder `<a>` links. Use basic Tailwind spacing/color — if tokens from 1002 are not yet available, use sensible fallback values that will be replaced when tokens are ready.
4. The header should have a background color and bottom border for visual separation.
5. Add a `<main>` element wrapping `{children}` with appropriate vertical padding.
6. Add a `<footer>` element with copyright text, placeholder links, and semantic `<footer>` tag.
7. All landmark roles are implicit with semantic HTML: `<header>` → banner, `<main>` → main, `<footer>` → contentinfo.
8. Add `data-testid="header"` and `data-testid="footer"` attributes for testing.
9. NOTE: This subtask can proceed in parallel with 1002 (tokens). Once 1002 completes, update color/spacing references to use design tokens. The layout structure and semantic HTML do not depend on token values.

## Validation
Navigate to `/` and verify `<header>`, `<main>`, and `<footer>` semantic elements are present in the DOM using `document.querySelector`. Header has `position: sticky` computed style. Axe-core reports no landmark-related violations. `data-testid="header"` and `data-testid="footer"` are queryable. Font loads without layout shift (no CLS regression).