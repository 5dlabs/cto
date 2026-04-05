Implement subtask 6005: Implement AI caption generation service

## Objective
Build an Effect.Service that uses OpenAI/Claude to generate social media captions, hashtags, and platform-specific variations for curated images. Support different tones and styles per platform.

## Steps
1. Create src/services/caption-generator.ts.
2. Define Effect.Service `CaptionGeneratorService` with methods: generateCaption(images: StoredImage[], context: CaptionContext) -> Effect<GeneratedCaption>, generatePlatformVariants(baseCaption: string, platforms: Platform[]) -> Effect<PlatformCaptions>.
3. CaptionContext: event_name, event_type, brand_voice, target_audience, key_messages, platform.
4. GeneratedCaption: caption_text, hashtags (string[]), call_to_action, emoji_enhanced.
5. PlatformCaptions: per-platform variations (Instagram: longer + hashtags, LinkedIn: professional tone, TikTok: trendy/short, Facebook: conversational).
6. Implement using OpenAI Chat Completions API with structured output (JSON mode).
7. System prompt includes brand voice guidelines, platform best practices, and character limits per platform.
8. Handle API errors, rate limits, and content moderation flags.
9. Implement mock for testing.

## Validation
Unit tests with mocked AI API verify correct prompt construction with context; generated captions respect platform character limits; platform variants differ in tone; hashtags are relevant; mock returns testable deterministic captions.