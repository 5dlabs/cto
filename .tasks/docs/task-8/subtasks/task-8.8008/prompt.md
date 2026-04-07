Implement subtask 8008: Build product detail page (app/equipment/[id]/page.tsx) with real-time availability date picker

## Objective
Implement the individual product detail page with server-side product data fetch and client-side date range picker that updates the availability badge in real time via TanStack Query.

## Steps
1. Create app/equipment/[id]/page.tsx as a Server Component. Call runApiEffect(getProductById(params.id)) for initial render. generateMetadata export for product name in page title.
2. Create components/equipment/ProductDetail.tsx as 'use client':
   - Props: product (Product), initialAvailability (Availability | null).
   - Display: product name (h1), day_rate as currency/day, large next/image.
   - Date picker: import DayPicker from react-day-picker. Allow range selection (from/to). On range change: invalidate and refetch TanStack Query: useQuery({ queryKey: ['availability', product.product_id, from, to], queryFn: () => runApiEffect(getAvailability(id, from, to)), enabled: !!from && !!to }).
   - Availability badge: renders quantity_available from query data. Green badge if > 0, red if === 0. Shows Skeleton while loading.
   - 'Add to Quote' button: on click, stores {product_id, quantity: 1, days: diffDays(from, to)} to localStorage key sigma1_quote_items and navigates to /quote.
3. Handle 404: if getProductById throws, call notFound() to render Next.js 404 page.

## Validation
Playwright: GET /equipment/<valid_id> shows product name and day_rate. Click date range picker, select dates 3 days apart — availability badge updates without full page reload (assert badge text changes, no network navigation). GET /equipment/nonexistent-id returns 404 page. TanStack Query devtools (in dev mode) shows availability query with correct params.