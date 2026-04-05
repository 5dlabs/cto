Implement subtask 6004: Implement AI caption generation service

## Objective
Build the AI-powered caption generation service that creates platform-specific captions for curated images using OpenAI/Claude.

## Steps
1. Create `src/services/caption.ts` module using Effect.Service pattern.
2. Define a `CaptionService` Effect.Service with method: `generateCaption(photoId: string, platform: Platform, context?: CaptionContext) -> Effect.Effect<GeneratedCaption, CaptionError>`.
3. Implement caption generation:
   - Fetch the photo and any associated metadata.
   - Construct a platform-specific prompt (Instagram: hashtags and emojis, LinkedIn: professional tone, TikTok: trending/casual, Facebook: engaging/shareable).
   - Call OpenAI/Claude text API with the prompt and optional image context.
   - Parse and validate the generated caption (length limits per platform, appropriate formatting).
4. Implement POST `/api/v1/social/captions/generate` endpoint:
   - Accept photo_id, platform, optional tone/context parameters.
   - Return generated caption text with platform-specific metadata (hashtag suggestions, character count).
5. Implement POST `/api/v1/social/drafts` endpoint:
   - Create a draft combining selected photo(s) with generated caption.
   - Persist to `drafts` table with status 'draft'.
6. Implement GET `/api/v1/social/drafts` — list drafts with filtering by status.
7. Implement PUT `/api/v1/social/drafts/:id` — edit draft caption or photos.
8. Make AI provider configurable via environment variable.

## Validation
Mock the AI API and verify captions are generated with correct platform-specific formatting. Create a draft and verify it persists correctly. Edit a draft and verify changes are saved. List drafts with status filter and verify results.