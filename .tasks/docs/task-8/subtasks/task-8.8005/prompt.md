Implement subtask 8005: Build equipment detail page (/equipment/:id) with availability checker

## Objective
Implement the /equipment/[id] dynamic route showing full equipment details, image gallery, specifications, and an interactive availability checker that queries the backend.

## Steps
1. Create `app/equipment/[id]/page.tsx` with `generateMetadata` for dynamic SEO (title, description, og:image from equipment data).
2. Create `app/equipment/[id]/equipment-detail.tsx` client component using `useEquipmentDetail(id)` hook.
3. Display: equipment name, full description, image gallery (use a simple lightbox or carousel), category, specifications table, daily/weekly rates.
4. Build an availability checker component: date range picker input, 'Check Availability' button, uses `useAvailability(id, dates)` hook, displays available/unavailable status with visual indicator.
5. Add a 'Request Quote' CTA button that links to /quote with the equipment pre-selected (via query param).
6. Add breadcrumb navigation: Home > Equipment > [Equipment Name].
7. Handle 404 state when equipment ID is not found.
8. Add loading skeleton matching the page layout.

## Validation
Page renders at /equipment/[valid-id] with correct equipment data. Image gallery displays and is navigable. Availability checker returns and displays status for selected dates. 'Request Quote' button navigates to /quote with equipment ID as query param. /equipment/[invalid-id] shows 404 state. Dynamic metadata (title, og tags) is correct in page source.