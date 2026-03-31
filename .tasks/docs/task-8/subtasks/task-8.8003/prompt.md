Implement subtask 8003: Set up TanStack Query + Effect data fetching layer and API client services

## Objective
Create the shared data fetching infrastructure: TanStack Query provider, Effect-based API client services for the Equipment Catalog API and Morgan API, and Effect.Schema definitions for all API response types.

## Steps
1. Install TanStack Query (`@tanstack/react-query`, `@tanstack/react-query-devtools`).
2. Create `app/providers.tsx` with QueryClientProvider wrapping children, and integrate into `app/layout.tsx`.
3. Create `@/services/equipment-api.ts` using Effect to define typed API calls: `getEquipmentList(filters)`, `getEquipmentById(id)`, `checkAvailability(id, dates)`. Use `@effect/platform/HttpClient` or fetch wrapper.
4. Create `@/services/quote-api.ts` with Effect service for `submitQuote(quoteData)` and `getQuoteStatus(id)`.
5. Create `@/services/morgan-api.ts` with Effect service for Morgan chat endpoints if needed beyond widget embedding.
6. Define Effect.Schema types in `@/schemas/equipment.ts` (Equipment, EquipmentListResponse, EquipmentDetail, Availability), `@/schemas/quote.ts` (QuoteRequest, QuoteResponse), and `@/schemas/common.ts` (PaginatedResponse, ApiError).
7. Create `@/lib/api-helpers.ts` with a utility to run an Effect program and return the result for use in TanStack Query `queryFn` callbacks.
8. Create custom hooks: `useEquipmentList()`, `useEquipmentDetail(id)`, `useAvailability(id, dates)`, `useSubmitQuote()` that combine TanStack Query with Effect services.
9. Add TanStack Query Devtools in development mode.

## Validation
Write unit tests for each Effect.Schema that validate correct parsing of mock API responses and reject malformed data. Verify custom hooks can be called in a test component and that TanStack Query devtools appear in dev mode. Mock API responses and confirm the Effect pipeline correctly decodes them.