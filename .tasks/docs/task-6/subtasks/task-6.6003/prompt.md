Implement subtask 6003: Implement AI curation pipeline for image selection and caption generation

## Objective
Build the AI curation module that uses OpenAI or Claude to analyze uploaded images, select the best ones, and generate platform-appropriate captions.

## Steps
1. Create `src/ai/curation.ts` module.
2. Add openai SDK dependency (or anthropic SDK depending on dp choice — implement with an adapter pattern to support both).
3. Implement `curateImages(imageUrls: string[], context?: string) -> Effect<CurationResult>`:
   - Send images to the vision model for quality/relevance scoring.
   - Return ranked list with scores and selection recommendations (top N images).
4. Implement `generateCaption(imageUrl: string, platform: Platform, context?: string) -> Effect<Caption>`:
   - Generate platform-specific captions (Instagram: hashtags/emojis, LinkedIn: professional tone, Facebook: conversational).
   - Return caption text, suggested hashtags, and alt text.
5. Define Effect.Schema types for CurationResult and Caption.
6. Read OPENAI_API_KEY or ANTHROPIC_API_KEY from Kubernetes secrets.
7. Handle API errors, rate limits, and timeouts (30s for vision calls).
8. Add unit tests with mocked AI responses.

## Validation
Unit tests with mocked AI responses verify: image curation returns ranked list with scores; captions differ per platform (Instagram includes hashtags, LinkedIn is professional); API errors are handled gracefully; timeout produces a descriptive error.