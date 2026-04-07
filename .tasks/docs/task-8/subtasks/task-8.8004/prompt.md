Implement subtask 8004: Build root layout with sidebar navigation and mobile hamburger menu

## Objective
Implement app/layout.tsx with the shadcn/ui Sidebar for desktop and Sheet-based hamburger menu for mobile, plus nav links to all pages.

## Steps
1. Create app/layout.tsx as a Server Component. Import Sidebar from components/ui/sidebar, Sheet from components/ui/sheet.
2. Desktop sidebar (>= md breakpoint): fixed left sidebar 240px wide. Logo at top (Sigma-1 wordmark or SVG). Nav items: Home (/), Equipment (/equipment), Quote (/quote), Portfolio (/portfolio), Contact (/#contact). Each nav item uses Next.js Link. Active state via usePathname() in a client sub-component.
3. Mobile (< md breakpoint): hamburger icon button (Menu icon from lucide-react) in sticky top bar. On click: open Sheet from left with same nav links. Sheet closes on nav item click.
4. Main content area: ml-[240px] on desktop, ml-0 on mobile. Padding: px-6 py-8 on desktop, px-4 py-6 on mobile.
5. Add metadata export: title: 'Sigma-1 – Event Lighting & Visual Production', description for SEO.
6. Morgan web chat script placeholder (next/script tag for widget — implemented in subtask 8010).
7. Schema.org JSON-LD script tag (implemented in subtask 8010).

## Validation
Playwright: navigate to / on 1280px viewport — sidebar is visible with 5 nav links. Navigate to / on 375px viewport — sidebar is hidden, hamburger button is visible. Click hamburger — Sheet opens with nav links. Click 'Equipment' link in Sheet — navigates to /equipment and Sheet closes.