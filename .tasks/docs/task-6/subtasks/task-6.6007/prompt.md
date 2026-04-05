Implement subtask 6007: Implement CaptionService with AI-powered caption generation

## Objective
Create the CaptionService as an Effect service that generates platform-specific captions with relevant hashtags using OpenAI/Claude, incorporating event context and equipment details.

## Steps
1. Create `src/services/CaptionService.ts` as an Effect.Service.
2. Define the service interface:
   - `generateCaption(input: CaptionInput): Effect.Effect<CaptionOutput, CaptionError>`
   - `CaptionInput`: `{ eventName?: string, eventDescription?: string, imageDescriptions: string[], platforms: string[], tone?: string }`.
   - `CaptionOutput`: `{ caption: string, hashtags: string[], platformVariants?: Record<string, string> }` — a base caption plus optional per-platform variants (LinkedIn more professional, Instagram more casual, TikTok more trending).
3. Construct a prompt that:
   - Includes event context (name, description) if available.
   - References what's visible in the images (from image descriptions/AI curation metadata).
   - Requests hashtags relevant to the industry, event, and equipment.
   - Asks for platform-specific tone adjustments.
4. Parse the AI response into structured CaptionOutput.
5. Define `CaptionError` as a tagged Effect error.
6. Create `CaptionServiceLive` layer depending on OpenAI API key from environment.
7. Ensure captions respect platform character limits (Instagram 2200, LinkedIn 3000, TikTok 2200, Facebook 63206).

## Validation
Unit test with mocked OpenAI: verify generateCaption returns a caption containing the event name, at least 3 hashtags, and platform variants for each requested platform. Test that captions respect character limits for each platform. Test CaptionError is raised on malformed AI response.