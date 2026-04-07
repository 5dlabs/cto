Implement subtask 8005: Implement Effect ApiClient service and Effect.Schema type definitions for all backend response shapes

## Objective
Create lib/api.ts with an Effect.Service-based ApiClient and define Effect.Schema schemas for all backend data types used across the site.

## Steps
1. Create apps/website/lib/api.ts.
2. Define ApiClient using Effect.Service pattern:
   ```ts
   import { Effect, Layer } from 'effect'
   import * as S from '@effect/schema/Schema'
   
   class ApiClient extends Effect.Service<ApiClient>()('ApiClient', {
     effect: Effect.sync(() => ({
       baseUrl: process.env.NEXT_PUBLIC_API_URL ?? ''
     }))
   }) {}
   ```
3. Define Effect.Schema types for all response shapes:
   - Product: { product_id: S.String, name: S.String, day_rate: S.Number, category: S.String, image_url: S.optionalWith(S.String, {default: () => ''}) }
   - Category: { category_id: S.String, name: S.String }
   - Availability: { product_id: S.String, quantity_available: S.Number, from: S.String, to: S.String }
   - Opportunity: { opportunity_id: S.String, status: S.String, total_cents: S.Number }
   - PublishedPost: { post_id: S.String, image_url: S.String, caption: S.String, hashtags: S.Array(S.String), published_at: S.String }
4. Define typed fetch helpers using Effect.tryPromise with schema decoding:
   - getCategories(): Effect.Effect<Category[]>
   - getProducts(search?: string): Effect.Effect<Product[]>
   - getProductById(id: string): Effect.Effect<Product>
   - getAvailability(id: string, from: string, to: string): Effect.Effect<Availability>
   - createOpportunity(body: OpportunityCreate): Effect.Effect<Opportunity>
   - getPublishedPosts(): Effect.Effect<PublishedPost[]>
5. Export runApiEffect helper: wraps Effect.runPromise with ApiClient.Default layer.
6. For client components needing TanStack Query: export queryFn wrappers that call runApiEffect.

## Validation
`tsc --noEmit` on lib/api.ts exits 0. Unit test: mock fetch returning valid Product JSON, call getProducts() via runApiEffect, assert decoded array has correct types. Unit test: mock fetch returning invalid JSON (missing day_rate), assert Effect fails with ParseError.