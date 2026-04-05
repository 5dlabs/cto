Implement subtask 6009: End-to-end social media pipeline integration tests

## Objective
Write comprehensive integration tests validating the full social media pipeline: upload → curation → caption → approval → publish → sync.

## Steps
1. Set up a test harness with a real PostgreSQL test database (via testcontainers or test-specific DB) and mock S3 (using localstack or s3-mock).
2. Mock all external APIs: AI provider (OpenAI/Claude), Instagram, LinkedIn, TikTok, Facebook, Morgan, website portfolio.
3. Test the full happy path:
   a. Upload photos → verify stored in S3 and DB.
   b. Run curation → verify scores assigned and top images selected.
   c. Generate captions → verify platform-specific captions created.
   d. Create draft → verify persisted with correct status.
   e. Request approval → verify Morgan is called, status becomes pending_approval.
   f. Approve draft → verify status becomes approved.
   g. Publish to multiple platforms → verify all platform APIs called, published_posts created.
   h. Verify portfolio sync triggered → verify website API called, sync_status updated.
4. Test error scenarios:
   a. AI API failure during curation → verify graceful error response.
   b. One platform publish fails → verify other platforms still succeed, partial results returned.
   c. Approval rejection → verify draft cannot be published.
   d. Portfolio sync failure → verify retry mechanism.
5. Verify Effect.Schema validation rejects malformed requests at each endpoint.
6. Clean up test data between scenarios.

## Validation
All happy path steps complete successfully in sequence. Error scenarios produce correct error responses without corrupting state. Schema validation prevents invalid data. All database state is consistent after each test.