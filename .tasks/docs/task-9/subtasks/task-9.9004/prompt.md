<identity>
You are tap working on subtask 9004 of task 9.
</identity>

<context>
<scope>
Create lib/api.ts defining typed Effect-based fetch functions for Product, Opportunity, and SocialPost. Define Effect.Schema decoders for all three types. Create a TanStack Query v5 queryClient singleton. Implement query factory functions that wrap Effect runners for use in useQuery/useMutation hooks. Include Effect.catchAll error mapping to typed AppError variants displayed as user-friendly messages.
</scope>
</context>

<implementation_plan>
In lib/schemas.ts define: `const Product = Schema.Struct({ id: Schema.String, name: Schema.String, day_rate: Schema.Number, image_url: Schema.String, available: Schema.Boolean, barcode: Schema.optional(Schema.String) })` — similarly for Opportunity and SocialPost. In lib/api.ts: `const fetchProducts = (search?: string) => Effect.tryPromise(() => fetch(`${API_URL}/api/v1/catalog/products?q=${search??''}`).then(r => r.json())).pipe(Effect.flatMap(Schema.decodeUnknown(Schema.Array(Product))), Effect.catchAll(e => Effect.fail(new AppError(e))))`. Export `runEffect = <A>(eff: Effect.Effect<A, AppError>) => Effect.runPromise(eff)`. In lib/queryClient.ts export a singleton QueryClient with staleTime: 30_000. Write queryKeys factory: `productKeys = { all: ['products'], list: (q: string) => [...productKeys.all, q], detail: (id: string) => [...productKeys.all, id] }`.
</implementation_plan>

<validation>
Jest unit tests: (1) `Schema.decodeUnknown(Product)` on a valid object resolves, on invalid object rejects with ParseError. (2) `fetchProducts` with msw intercepting GET /api/v1/catalog/products returns decoded Product[] array. (3) `fetchProducts` with msw returning 500 rejects with AppError. Coverage of lib/ >= 80%.
</validation>