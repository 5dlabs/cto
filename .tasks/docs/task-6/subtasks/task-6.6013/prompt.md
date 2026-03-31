Implement subtask 6013: Write integration tests for the complete social media workflow

## Objective
Create end-to-end integration tests covering the full workflow: photo upload → AI curation → draft creation → approval → publish → portfolio sync.

## Steps
1. Create `tests/integration/` directory with test files using Bun's built-in test runner. 2. Set up test fixtures: mock S3 using a local MinIO container or in-memory mock, mock AI APIs using MSW (Mock Service Worker) or similar, mock social platform APIs, use a test PostgreSQL database. 3. Test full happy path: upload photo → verify storage → create draft (triggers AI) → verify draft with caption → submit for approval → approve → publish → verify published record and portfolio sync call. 4. Test rejection path: upload → draft → submit → reject with reason → verify cannot publish rejected draft. 5. Test error scenarios: AI service timeout during draft creation, platform API failure during publish (verify draft stays approved for retry), invalid file upload, duplicate publish attempt. 6. Test schema validation: malformed requests to each endpoint. 7. Add test for concurrent draft creation from same photo for multiple platforms. 8. Ensure all tests clean up database and storage state.

## Validation
All integration tests pass with `bun test`. Happy path test completes the full upload-to-publish workflow. Error scenario tests verify correct status codes and error messages. Tests are idempotent and can run in CI.