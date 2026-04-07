Implement subtask 9006: Build product detail screen with DateRangePicker and availability query

## Objective
Implement app/equipment/[id].tsx. Fetch product details via TanStack Query. Render a custom DateRangePicker component. On date range selection, fire a secondary useQuery to GET /api/v1/catalog/products/:id/availability with the selected dates. Display quantity_available. Render 'Add to Quote' Pressable that navigates to the quote tab with the product pre-filled via router.push params.

## Steps
Use `useLocalSearchParams` to get `id`. Query product detail: `useQuery({ queryKey: productKeys.detail(id), queryFn: () => runEffect(fetchProductDetail(id)) })`. DateRangePicker component: two calendar presses setting startDate and endDate state. On both dates set, fire: `useQuery({ queryKey: ['availability', id, startDate, endDate], queryFn: () => runEffect(fetchAvailability(id, startDate, endDate)), enabled: !!(startDate && endDate) })`. Show skeleton while loading availability. Show `quantity_available` badge once resolved. 'Add to Quote' button: `router.push({ pathname: '/(tabs)/quote', params: { productId: id, startDate, endDate } })`.

## Validation
RNTL test with msw: render detail screen for product id='p1'. Assert product name rendered. Simulate selecting start+end dates — assert msw received GET /api/v1/catalog/products/p1/availability with correct date params. Assert quantity_available text appears after mock resolves. Assert 'Add to Quote' press calls router.push with correct params.