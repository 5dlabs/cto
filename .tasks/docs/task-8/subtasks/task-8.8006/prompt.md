Implement subtask 8006: Build Product Detail page with specifications and Add to Quote functionality

## Objective
Implement the `/equipment/[id]` dynamic product detail page showing product images (from R2 CDN), JSONB specifications, day rate pricing, and an 'Add to Quote' button that adds the item to the quote builder's state. Include Effect Schema validation on API responses.

## Steps
1. Create `app/equipment/[id]/page.tsx`.
2. Data fetching: use `generateMetadata` for dynamic SEO (product name, description, image in OG tags). Fetch product data server-side for initial render, hydrate client-side with TanStack Query for freshness.
3. Define Effect Schema for product API response in `lib/schemas/equipment.ts` — validate all fields including JSONB specs.
4. Product detail layout:
   - Image gallery: primary image large, thumbnails below (if multiple images). Use `next/image` with R2 CDN URLs.
   - Product name, category badge, day rate prominently displayed.
   - Specifications table: render JSONB specs as key-value pairs in a clean table/description list.
5. 'Add to Quote' button:
   - On click, add product { id, name, dayRate, quantity: 1 } to a shared quote state.
   - Quote state management: use a React Context + useReducer at the layout level (`QuoteProvider`), persisted to sessionStorage.
   - Show toast notification on add: "[Product name] added to quote".
   - If item already in quote, increment quantity or show a notice.
6. Schema.org Product structured data in JSON-LD.
7. Breadcrumb navigation: Home > Equipment > [Product Name].

## Validation
Component test: render product detail with mock product data, verify image renders with correct src, specifications table shows all key-value pairs, day rate displays formatted currency. Test 'Add to Quote': click button, verify QuoteProvider context state contains the product. Test Effect Schema validation: pass malformed API response, verify error is caught and displayed gracefully.