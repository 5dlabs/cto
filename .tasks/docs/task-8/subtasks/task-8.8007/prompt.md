<identity>
You are blaze working on subtask 8007 of task 8.
</identity>

<context>
<scope>
Implement the equipment catalog browser as a server-rendered page with client-side category filtering, debounced search, and product cards displaying image, name, day rate, and availability badge.
</scope>
</context>

<implementation_plan>
1. Create app/equipment/page.tsx as a Server Component. Fetch categories and initial product list server-side using runApiEffect(getCategories()) and runApiEffect(getProducts()).
2. Pass initial data to a client component EquipmentCatalog via props.
3. Create components/equipment/EquipmentCatalog.tsx as 'use client' component:
   - State: selectedCategory (string | null), searchQuery (string).
   - Category filter: horizontally scrollable row of Badge buttons. Click sets selectedCategory, triggers client-side re-fetch via TanStack Query: useQuery({ queryKey: ['products', category, search], queryFn: () => runApiEffect(getProducts(search)) }).
   - Search input: Input component, onChange with 300ms debounce using setTimeout/clearTimeout.
   - Product grid: responsive grid (2 cols mobile, 3 cols md, 4 cols lg).
4. Create components/equipment/ProductCard.tsx: next/image for product image (objectFit: cover, aspect-ratio: 4/3), product name (h3), day_rate formatted as currency, availability badge (green Sold or In Stock — static for catalog view).
5. Loading state: 8 Skeleton cards while TanStack Query is fetching.
6. Link product card to /equipment/[id].
</implementation_plan>

<validation>
Playwright: GET /equipment shows product grid with >= 1 card. Click 'Lighting' category badge — URL or state updates, only lighting products visible. Type 'truss' in search — product list updates to matching items within 500ms. Network tab shows GET /api/v1/catalog/products with search and category query params.
</validation>