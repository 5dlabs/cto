Implement subtask 6003: Define Effect.Schema types and Effect.Service interfaces

## Objective
Define all Effect.Schema request/response types (UploadEventPhotosRequest, DraftResponse, PublishRequest) and the Effect.Service interfaces for InstagramService, LinkedInService, TikTokService, and FacebookService with retry and timeout configuration.

## Steps
Create src/schemas.ts: use Schema.Struct from effect/Schema to define UploadEventPhotosRequest {event_id: Schema.UUID, photo_count: Schema.Number}, DraftResponse {id, event_id, status, selected_photo_urls, caption, hashtags, created_at}, PublishRequest {draft_id, platforms: Schema.Array(Schema.Literal('instagram','linkedin','tiktok','facebook'))}. Create src/services/platform-services.ts: define Effect.Service classes for each platform. Each service method signature: publishPost(draft: DraftResponse): Effect.Effect<{external_post_id: string}, PlatformError>. Add Schedule.exponential('1 second') with Schedule.recurs(5) as the retry policy. Add Effect.timeout('30 seconds') wrapping each service call. Export a PlatformServiceLayer combining all four services for use in the Elysia routes.

## Validation
TypeScript compilation with `tsc --noEmit` exits 0 with all schema and service types resolving. Unit test: instantiate each Effect.Service with a mock implementation and verify the service tag resolves via Effect.runPromise with a test layer.