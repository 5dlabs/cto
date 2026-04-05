Implement subtask 6003: Implement AI curation pipeline for image scoring and caption generation

## Objective
Build the AI curation pipeline that uses OpenAI/Claude to score uploaded images for social media suitability and generate platform-appropriate captions.

## Steps
1. Create a `services/ai-curation` module. 2. Define an AI provider interface using Effect Service pattern: ImageScorer (score images 0-1 for social media appeal), CaptionGenerator (generate captions for given image + context). 3. Implement the chosen AI provider (OpenAI GPT-4 Vision or Claude) using their respective SDKs. Load API key from Kubernetes secrets. 4. Image scoring: send each image URL to the vision model with a prompt asking to rate social media appeal (composition, lighting, relevance) on a 0-1 scale. Parse the numeric score from the response. 5. Caption generation: for top-scoring images (threshold configurable, default > 0.6), generate platform-specific captions (Instagram with hashtags, LinkedIn professional tone, Facebook casual). 6. Create a pipeline function: accept an array of image URLs, score all images, select top N (configurable, default 5), generate captions for selected images, return curated set as draft records. 7. Persist drafts to the database with status='draft', ai_score, and generated captions. 8. Implement token/cost tracking to log API usage.

## Validation
Unit test with mocked AI responses: verify scoring sorts images correctly, top N selection works, captions are generated for each target platform. Integration test with real AI API: submit 3 test images and verify scores and captions are returned. Verify drafts are persisted to DB.