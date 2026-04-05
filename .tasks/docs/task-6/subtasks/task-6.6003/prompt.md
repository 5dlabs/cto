Implement subtask 6003: Implement AI curation pipeline for image selection

## Objective
Build the AI-powered curation pipeline that analyzes uploaded photos and selects the top images for social media posting using OpenAI/Claude vision capabilities.

## Steps
1. Create `src/services/curation.ts` module using Effect.Service pattern.
2. Define a `CurationService` Effect.Service with method: `curateImages(photoIds: string[]) -> Effect.Effect<CuratedResult, CurationError>`.
3. Implement the curation logic:
   - Fetch photo records and generate presigned S3 URLs or download images.
   - Send images to the AI vision API (OpenAI GPT-4V or Claude) with a prompt asking to score each image on composition, lighting, brand-suitability, and engagement potential (1-10 scale).
   - Parse AI response to extract scores per image.
   - Update `curation_score` on each photo record.
   - Mark top N images as `is_curated = true` based on score threshold.
4. Implement POST `/api/v1/social/curation/run` endpoint that triggers curation for a batch of photo IDs.
5. Implement GET `/api/v1/social/curation/results` — list curated (top-scored) images.
6. Handle AI API errors gracefully — retry with backoff, return partial results if some images fail.
7. Make the AI provider configurable (OpenAI vs Claude) via environment variable.

## Validation
Mock the AI API and verify curation scores are assigned to photos. Verify top images are marked as curated. Test with partial AI failures and verify graceful degradation. Verify the curation endpoint returns curated results.