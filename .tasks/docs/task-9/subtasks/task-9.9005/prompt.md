Implement subtask 9005: Build equipment catalog tab with FlatList, debounced search, expo-image product cards, and pull-to-refresh

## Objective
Implement app/(tabs)/index.tsx as the equipment catalog. Render a search TextInput with 300ms debounce, a FlatList of ProductCard components using expo-image for R2 image URLs, day_rate badge, and availability indicator. Data fetched via TanStack Query useQuery wrapping the Effect fetchProducts function. Implement pull-to-refresh via FlatList refreshControl prop.

## Steps
Use `useQuery({ queryKey: productKeys.list(debouncedSearch), queryFn: () => runEffect(fetchProducts(debouncedSearch)) })`. Implement debounce with useRef+useEffect clearing a setTimeout at 300ms. ProductCard: `<Pressable onPress={() => router.push('/equipment/' + item.id)}><Image source={{ uri: item.image_url }} style={{ width: '100%', height: 200 }} contentFit='cover' /><Text className='font-semibold'>{item.name}</Text><Badge>{item.day_rate}/day</Badge><AvailabilityDot available={item.available} /></Pressable>`. RefreshControl triggers `queryClient.invalidateQueries(productKeys.all)`. Show skeleton placeholders while `isLoading`. Show error banner on `isError`.

## Validation
RNTL test: render catalog screen with msw returning 3 products — assert 3 ProductCard elements rendered. Type 'cam' in search input — after 300ms debounce assert msw received GET with `q=cam`. Pull-to-refresh gesture triggers refetch (spy on queryClient.invalidateQueries). Empty state renders when msw returns [].