Implement subtask 8005: Build Equipment Catalog listing page with filtering, search, and pagination

## Objective
Implement the `/equipment` route with a server-rendered product grid, category sidebar filter, search-by-name input, and pagination. Each ProductCard displays image, name, day rate, and availability indicator. Data fetched from catalog API endpoints.

## Steps
1. Create `app/equipment/page.tsx` — dynamic route (search params for category, search term, page).
2. Server Component fetches initial data: `GET /api/v1/catalog/categories` for sidebar, `GET /api/v1/catalog/products?category=X&search=Y&page=N` for grid.
3. Category sidebar:
   - List all categories with count badges.
   - Clicking a category updates URL search params (shallow navigation).
   - 'All Categories' option to clear filter.
4. Search bar: text input with debounced client-side search that updates URL search params.
5. Product grid:
   - Responsive grid: 1 col mobile, 2 cols tablet, 3-4 cols desktop.
   - ProductCard custom component: R2 CDN image via `<Image>`, product name, day rate formatted as currency, availability Badge (green/red).
   - Skeleton loading states using shadcn Skeleton.
6. Pagination: page number buttons at grid bottom, updates search params.
7. Client-side interactivity: wrap search/filter in a Client Component that uses `useRouter` and `useSearchParams` to update URL, triggering server re-render or TanStack Query refetch.
8. SEO: `generateMetadata` with dynamic title based on category, canonical URL.
9. Schema.org: ItemList on the listing page.

## Validation
Mock catalog API responses. Render `/equipment` page, verify product grid shows correct number of ProductCards. Test category filter: click a category, verify URL search params update and grid re-renders. Test search: type a query, verify debounced URL update. Test pagination: click page 2, verify page param in URL. Verify ProductCard displays image, name, and price. Responsive test at 375px (1 col), 768px (2 cols), 1440px (3+ cols).