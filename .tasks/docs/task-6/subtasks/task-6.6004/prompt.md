Implement subtask 6004: Implement AI caption generation service

## Objective
Build the AI caption generation module that creates platform-appropriate captions for curated social media posts using OpenAI or Claude.

## Steps
1. Create a `services/ai-caption.ts` module. 2. Implement `generateCaption(draft: Draft): Effect.Effect<CaptionResult, AIError>` that: a) Takes the curated photos and event context. b) Generates platform-specific captions: Instagram (casual, hashtags, emoji), LinkedIn (professional, industry-relevant), Facebook (conversational, engagement-focused). c) Returns a `CaptionResult` with per-platform captions. 3. Use a system prompt that includes brand voice guidelines, event context, and platform-specific formatting rules. 4. Update the draft with generated captions and transition status to 'pending_approval'. 5. Implement caption regeneration: allow re-calling the endpoint to get alternative captions. 6. Use Effect.retry for API resilience. 7. Validate caption length against platform limits (Instagram: 2200 chars, LinkedIn: 3000 chars, Facebook: 63206 chars).

## Validation
Given a draft with curated photos and event context, caption generation returns platform-specific captions within character limits. Draft status transitions to 'pending_approval'. Captions contain appropriate formatting per platform (hashtags for Instagram, professional tone for LinkedIn). Effect.retry handles API failures. Caption regeneration produces different output on subsequent calls.