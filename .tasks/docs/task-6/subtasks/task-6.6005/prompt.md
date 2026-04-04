Implement subtask 6005: Implement AI photo scoring pipeline with OpenAI Vision API

## Objective
Build the AI photo scoring service that uses OpenAI Vision API to evaluate each uploaded photo for composition, lighting quality, and brand relevance, then selects the top photos based on score threshold.

## Steps
1. Install `openai` npm package.
2. Create `src/services/AIScoringService.ts` as an Effect.Service:
   - Constructor takes OPENAI_API_KEY from environment.
   - Method `scorePhoto(imageUrl: string, eventContext: string)`: Effect<PhotoScore, AIScoringError>
     - Calls OpenAI Vision API (gpt-4o or gpt-4-vision-preview) with the image URL.
     - Prompt: Score this event photo on a scale of 1-10 for: composition, lighting, brand_relevance. Return JSON: { composition: number, lighting: number, brand_relevance: number, overall: number, reasoning: string }.
     - Parse response with Effect Schema, calculate overall = weighted average (composition 0.3, lighting 0.3, brand_relevance 0.4).
     - Wrap in Effect.retry with Schedule.exponential for rate limit handling (3 retries, 1s base).
   - Method `scoreAndSelectPhotos(photos: Photo[], eventContext: string, threshold: number = 6.0, maxSelect: number = 10)`: Effect<SelectedPhotos, AIScoringError>
     - Score all photos with controlled concurrency (Effect.forEach concurrency: 3 to respect rate limits).
     - Update ai_score in photos table for each.
     - Select photos with overall >= threshold, sorted by score desc, take top maxSelect.
     - Mark selected photos with `selected = true` in DB.
     - Return { selected: Photo[], rejected: Photo[] }.
3. Create `src/services/AIScoringService.live.ts` — Effect Layer.
4. Error handling: If OpenAI returns a non-parseable response, log warning and assign score 0 (exclude from selection).
5. Define `PhotoScore` type in `src/schemas/ai.ts`.

## Validation
Unit test with mocked OpenAI client: (1) Provide mock Vision API response, verify score calculation with correct weights. (2) Test selection logic: 15 photos scored, threshold 6.0, max 10 — verify correct top photos selected. (3) Test rate limit retry: mock 429 response then success, verify retry works. (4) Test malformed AI response: verify photo gets score 0 and is excluded. (5) Verify DB updates: ai_score set on all photos, selected=true only on chosen ones.