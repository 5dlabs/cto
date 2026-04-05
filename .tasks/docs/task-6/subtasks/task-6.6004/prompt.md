Implement subtask 6004: Implement AI curation pipeline for image selection and quality assessment

## Objective
Build an Effect.Service that uses OpenAI's vision API (or Claude) to analyze uploaded images, assess quality, select the best images for social media posting, and suggest cropping/composition improvements.

## Steps
1. Create src/pipelines/ai-curation.ts.
2. Define Effect.Service `AICurationService` with methods: curateImages(images: StoredImage[], context: CurationContext) -> Effect<CurationResult>, assessImageQuality(image: StoredImage) -> Effect<QualityAssessment>.
3. CurationContext: event_type, brand_guidelines, target_platforms, max_selections.
4. CurationResult: selected_images (ranked), quality_scores, crop_suggestions, rejection_reasons for unselected images.
5. QualityAssessment: overall_score (0-1), sharpness, composition, lighting, relevance_to_brand.
6. Implement using OpenAI GPT-4 Vision API: send image URLs with a structured prompt asking for quality assessment and selection rationale.
7. Parse structured JSON responses from the AI model.
8. Handle API rate limits with Effect.retry and exponential backoff.
9. Implement batch processing: if many images, process in groups of 4-5 per API call.
10. Provide a mock implementation for testing that returns deterministic scores.

## Validation
Unit tests with mocked OpenAI API verify correct prompt construction and response parsing; curation selects top-scored images; batch processing correctly groups images; retry logic handles rate limits; mock implementation returns consistent test data.