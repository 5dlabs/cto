Implement subtask 6005: Implement AI curation service with OpenAI/Claude for caption generation

## Objective
Create an Effect.Service abstraction for AI-powered caption generation. Integrate with OpenAI and/or Claude APIs to analyze uploaded photos and generate platform-specific captions.

## Steps
1. Install `openai` SDK (and optionally `@anthropic-ai/sdk`). 2. Create `src/services/AICurationService.ts` as an Effect.Service tag with methods: generateCaption(photoUrl: string, platform: Platform, context?: string) → Effect<CaptionResult, AICurationError>, analyzPhoto(photoUrl: string) → Effect<PhotoAnalysis, AICurationError>. 3. Create `src/layers/AICurationLayer.ts` reading OPENAI_API_KEY (and optionally ANTHROPIC_API_KEY) from env. 4. CaptionResult schema: { caption: string, hashtags: string[], suggestedPostTime: string, platform: Platform, confidence: number }. 5. PhotoAnalysis schema: { description: string, mood: string, colors: string[], subjects: string[], suggestedPlatforms: Platform[] }. 6. For caption generation, send the photo (via presigned URL or base64) to the vision model with a system prompt tailored per platform (Instagram: casual/hashtag-heavy, LinkedIn: professional, Facebook: community-oriented, TikTok: trendy/short). 7. Use Effect.retry with exponential backoff for transient API failures. 8. Add tagged errors: AICurationRateLimitError, AICurationAPIError, AICurationTimeoutError. 9. Add Prometheus histogram for AI API latency (social_ai_caption_duration_seconds).

## Validation
Call generateCaption with a test photo URL for each platform and verify returned CaptionResult matches the schema with non-empty caption and hashtags. Verify that a simulated API failure triggers retry logic. Verify rate limit errors are properly tagged.