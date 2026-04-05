Implement subtask 8006: Build Product Detail page with image gallery, specs table, and Add to Quote

## Objective
Implement the `/equipment/:id` route showing full product information: image gallery, specifications table (from JSONB data), day rate, and an 'Add to Quote' button that stores selection in quote builder state.

## Steps
1. Create `app/equipment/[id]/page.tsx` — dynamic route.
2. Server Component fetches `GET /api/v1/catalog/products/:id` and validates with Effect Schema.
3. Image gallery:
   - Primary image large display, thumbnail strip below.
   - Click thumbnail to swap primary image.
   - All images via `<Image>` with R2 CDN loader.
4. Product info section:
   - Product name (H1), category badge, day rate prominently displayed.
   - Description paragraph.
   - Specs table: render JSONB specs as key-value table rows using shadcn Table.
5. 'Add to Quote' button:
   - Stores product in client-side quote state (React context or localStorage-backed store).
   - Shows toast/notification on add.
   - Button text changes to 'Added ✓' if already in quote, with option to remove.
6. Quote state management: create `hooks/useQuoteStore.ts` (or context) that persists selected products and quantities to localStorage. Export `addProduct`, `removeProduct`, `getQuoteItems`.
7. SEO: `generateMetadata` with product name, description, image. Schema.org Product JSON-LD with name, image, offers (price, availability).
8. Breadcrumb navigation: Home > Equipment > [Product Name].

## Validation
Mock product API response with sample data including multiple images and JSONB specs. Render product detail page, verify: H1 shows product name, image gallery renders with thumbnails, specs table has correct rows, day rate displayed. Test 'Add to Quote' click: verify product added to localStorage quote state. Test Schema.org Product JSON-LD contains correct name, price, image URL.