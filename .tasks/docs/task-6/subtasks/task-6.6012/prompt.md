Implement subtask 6012: Write comprehensive integration and end-to-end tests for social media engine

## Objective
Create a full test suite covering all social media engine endpoints and pipelines end-to-end, with mocked external services (AI, social platforms, Signal, S3), verifying the complete content lifecycle from upload to publish.

## Steps
1. Set up test infrastructure: vitest config, test database with migrations, mock servers for all external APIs.
2. Mock setup: mock S3 (using mock-aws-s3 or similar), mock OpenAI responses, mock each social platform API, mock Signal webhook.
3. Test scenarios:
   a. Full lifecycle: upload images -> AI curates and generates caption -> draft created -> approve -> publish to all platforms -> verify published records.
   b. Rejection flow: upload -> draft -> reject with reason -> verify status and reason stored.
   c. Partial publish failure: one platform fails -> verify others succeed, failed platform has error recorded.
   d. AI curation with low-quality images -> verify selection logic.
   e. Multi-platform caption variants are generated correctly.
   f. Pagination and filtering on drafts and published endpoints.
   g. Concurrent uploads don't interfere with each other.
   h. Invalid file types rejected at upload.
4. Verify database state after each scenario.
5. Check code coverage >= 80%.

## Validation
All integration tests pass; complete content lifecycle (upload -> curate -> caption -> approve -> publish) works end-to-end with mocks; partial failures are handled correctly; code coverage report shows >= 80%; tests are deterministic and CI-ready.