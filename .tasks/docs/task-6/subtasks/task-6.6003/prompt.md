Implement subtask 6003: Implement AI curation pipeline for selecting top images

## Objective
Build the AI image curation module that analyzes uploaded event photos using a vision-capable AI model and selects the top images based on quality, composition, and relevance.

## Steps
1. Create a `services/ai-curation.ts` module. 2. Implement a provider-agnostic AI client interface (pending dp-15). For v1, implement against OpenAI GPT-4o vision API. 3. Implement `curateImages(draft: Draft): Effect.Effect<CurationResult, AIError>` that: a) Downloads or generates signed URLs for all photos in the draft. b) Sends photos in batches to the vision API with a prompt: 'Analyze these event photos. Score each 1-10 for quality, composition, lighting, and relevance to a business event. Return the top 3-5 images ranked by score with brief reasoning.' c) Parses the AI response into structured rankings. d) Updates the draft's selected_photos array and transitions status to 'pending_caption'. 4. Use Effect.retry for AI API calls with backoff. 5. Handle: API rate limits, oversized payloads (resize/compress before sending), model token limits. 6. Store curation scores/reasoning in a metadata field on the draft for transparency.

## Validation
Given a draft with 10 mock photo references, the curation pipeline calls the AI API and returns a ranked selection of 3-5 photos. Draft status transitions to 'pending_caption'. Curation metadata (scores, reasoning) is stored. Effect.retry handles transient API failures. Malformed AI responses are handled gracefully with a fallback (select first N photos).