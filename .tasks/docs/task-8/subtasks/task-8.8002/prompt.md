Implement subtask 8002: Implement equipment catalog pages with API integration

## Objective
Build the /equipment listing page and /equipment/:id detail page, integrating with the Equipment Catalog API for product data, search, filtering, and real-time availability checking.

## Steps
Step 1: Create the /equipment route with a server component that fetches the initial catalog listing via TanStack Query + Effect calling the Equipment Catalog API. Step 2: Implement the catalog listing UI: grid/list view toggle, category filters (tents, tables, chairs, lighting, etc.), search bar, and pagination. Step 3: Implement the equipment card component: product image (from object storage), name, category, price range, and availability indicator. Step 4: Create the /equipment/[id] dynamic route for equipment detail pages. Step 5: Implement the detail page UI: image gallery/carousel, full description, specifications table, pricing tiers, and a real-time availability calendar (calling sigma1_check_availability via the API). Step 6: Add a 'Request Quote' CTA on the detail page that navigates to the quote builder with the item pre-selected. Step 7: Implement loading states, error boundaries, and empty states for all catalog views. Step 8: Add Schema.org Product structured data to equipment detail pages for SEO.

## Validation
Equipment listing page loads and displays products from the API; search and category filters return correct results; equipment detail page shows full product info with images; availability calendar reflects real-time data; 'Request Quote' CTA navigates to quote builder with item context; structured data is present in page source.