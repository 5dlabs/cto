Implement subtask 8003: Set up Effect 3.x integration and TanStack Query data fetching layer

## Objective
Configure Effect 3.x for schema validation and data fetching. Integrate TanStack Query with Effect programs for client-side server state management. Create reusable API client utilities and Effect schemas for catalog, availability, and quote payloads.

## Steps
1. Install `effect`, `@effect/schema`, `@tanstack/react-query`, `@tanstack/react-query-devtools`.
2. Create `lib/effect/` directory with:
   - `api-client.ts`: Effect-based HTTP client wrapping fetch, reading `NEXT_PUBLIC_API_BASE_URL` from env. Handles JSON parsing, error mapping to Effect failures.
   - `schemas.ts`: Effect Schema definitions for Product, Category, AvailabilitySlot, QuoteRequest, QuoteResponse, matching backend API contracts.
3. Create `lib/queries/` directory with:
   - `catalog.ts`: TanStack Query hooks (`useCategories`, `useProducts`, `useProductById`) that internally run Effect programs via `Effect.runPromise`.
   - `availability.ts`: `useProductAvailability(productId, dateRange)` query hook.
   - `quote.ts`: `useSubmitQuote()` mutation hook.
4. Create `providers/query-provider.tsx`: wrap app in `QueryClientProvider` with default stale times, retry config.
5. Add QueryClientProvider to root layout.
6. Create `lib/effect/server-fetch.ts`: utility for Server Components to run Effect fetch programs (no hooks, direct `Effect.runPromise` in async server functions).
7. Export typed error types for API failures (NetworkError, ValidationError, NotFoundError).

## Validation
Unit test Effect schemas: validate that valid Product JSON parses successfully and invalid JSON returns decode errors. Unit test API client with mocked fetch: verify correct URL construction, header setting, and error mapping. Test TanStack Query hooks with `renderHook` and mocked query client: verify `useProducts` returns expected data shape.