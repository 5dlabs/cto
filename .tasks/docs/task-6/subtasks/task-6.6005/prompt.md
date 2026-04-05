Implement subtask 6005: Implement ImageCurationService with OpenAI Vision API scoring

## Objective
Create the ImageCurationService as an Effect service that scores uploaded images using OpenAI Vision API on composition quality, lighting, and subject clarity, returning scores 0-100 and selecting the top images from a batch.

## Steps
1. Install `openai` SDK.
2. Create `src/services/ImageCurationService.ts` as an Effect.Service.
3. Define the service interface:
   - `scoreImages(imageKeys: string[]): Effect.Effect<ScoredImage[], AICurationError>` — for each image key, get a presigned URL, send to OpenAI Vision API with a prompt asking to evaluate composition (0-100), lighting (0-100), subject clarity (0-100), and overall score.
   - `selectTopImages(scored: ScoredImage[], count?: number): ScoredImage[]` — pure function, sort by overall score descending, return top `count` (default 5, max 10).
4. Define `ScoredImage` type: `{ uploadId: string, key: string, scores: { composition: number, lighting: number, clarity: number, overall: number } }`.
5. Define `AICurationError` as a tagged Effect error.
6. OpenAI Vision prompt should be structured to return JSON with the score fields.
7. Parse OpenAI response robustly — handle malformed JSON with fallback parsing.
8. Use `Effect.forEach` with `{ concurrency: 3 }` to score images in parallel with bounded concurrency.
9. Create `ImageCurationServiceLive` layer that depends on OpenAI API key from environment.

## Validation
Unit test: mock OpenAI client to return structured scores for 10 images. Verify scoreImages returns all 10 with parsed scores. Verify selectTopImages returns top 5 sorted by overall score descending. Test malformed OpenAI response handling — verify AICurationError is raised or fallback score is assigned.