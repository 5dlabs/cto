## Acceptance Criteria

- [ ] 1. Unit test (Effect): ImageCurationService given mock OpenAI Vision responses, verify top 5 images selected by score descending. 2. Unit test (Effect): CaptionService generates caption containing event name and at least 3 hashtags. 3. Integration test: POST /api/v1/social/upload with 10 test images → verify uploads stored in R2 (mock) and draft created with status 'pending_approval'. 4. Integration test: approve draft → publish → verify published_posts records created for each platform with platform_post_id. 5. Retry test: mock InstagramService to fail twice then succeed, verify Effect.retry produces success after 3 attempts and published_posts record exists. 6. Partial failure test: Instagram publish succeeds but LinkedIn fails → verify draft status reflects partial publication and individual platform statuses are recorded. 7. Effect Schema validation test: POST /api/v1/social/upload with missing required fields returns 422 with structured error. 8. Platform crop test: given a 4000x3000 image, verify CropService produces correct dimensions for Instagram square (1080x1080), Story (1080x1920), LinkedIn (1200x628).

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.