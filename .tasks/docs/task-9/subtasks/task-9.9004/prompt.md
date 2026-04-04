Implement subtask 9004: Implement Equipment tab with category browsing, product grid, and infinite scroll

## Objective
Build the Equipment tab screens: category list, product grid with infinite scroll pagination, product detail with availability display, and pull-to-refresh across list screens.

## Steps
1. **Category List** (`equipment/index.tsx`): Fetch categories from API, render as a scrollable list/grid of category cards with icons/images. Pull-to-refresh using `RefreshControl`.
2. **Product Grid** (`equipment/[categoryId].tsx`): Fetch products for selected category with cursor-based pagination. Use `FlatList` with `onEndReached` for infinite scroll. Render `ProductCard` components in 2-column grid. Pull-to-refresh. Show loading skeleton while fetching.
3. **Product Detail** (`equipment/product/[productId].tsx`): Fetch single product details. Display full-size image carousel (use `expo-image` or `react-native-reanimated-carousel`), description, specifications, pricing info, `AvailabilityBadge`. Add 'Add to Quote' button that navigates to Quote tab with product pre-selected.
4. Implement search bar at the top of category list for filtering equipment by name.
5. Handle empty states (no products in category) and error states (network failure) with retry option.
6. Use React Query (`@tanstack/react-query`) or SWR for data fetching with caching, background refetch, and stale-while-revalidate.
7. Optimize `FlatList` performance: `getItemLayout` for fixed-height items, `keyExtractor`, `windowSize` tuning.

## Validation
Component tests: category list renders mock categories; product grid renders 2-column layout with mock products; product detail shows all fields. Integration test: mock paginated API response, verify infinite scroll triggers next page fetch when scrolled to end. Pull-to-refresh test: verify API refetch on refresh gesture. Empty state test: mock empty product response, verify empty state component renders.