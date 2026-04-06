Implement subtask 6004: Integrate AI service for caption generation and image curation

## Objective
Build an Effect.Service that interfaces with OpenAI or Claude to generate social media captions and curate/analyze uploaded images for suitability.

## Steps
1. In `src/services/ai-caption.ts`, create an Effect.Service `AICaptionService` with methods: a) `generateCaption(imageUrls: string[], context: CaptionContext) -> Effect<GeneratedCaption>` — sends image(s) and context (brand voice, target platform, product info) to AI model, returns generated caption with hashtags. b) `curateImage(imageUrl: string) -> Effect<CurationResult>` — analyzes image for quality, brand alignment, social media suitability, returns score and recommendations. 2. Define `CaptionContext` schema: brand_voice, platform, product_description, tone. 3. Define `GeneratedCaption` schema: caption_text, hashtags, platform_specific_variants. 4. Define `CurationResult` schema: quality_score, is_suitable, recommendations, suggested_crop. 5. Implement the AI client using OpenAI SDK or Anthropic SDK (based on dp-6-1 decision, default to OpenAI). 6. Use Effect for retry logic on transient API failures. 7. Provide a mock implementation for testing.

## Validation
Unit tests with mocked AI responses verify caption generation returns properly structured output; curation returns quality scores; retry logic handles transient failures; Effect.Schema validates AI response parsing; mock implementation works for integration tests.