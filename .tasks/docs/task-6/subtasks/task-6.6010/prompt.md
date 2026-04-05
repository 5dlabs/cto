Implement subtask 6010: Write integration tests for all Social Media Engine endpoints

## Objective
Create comprehensive integration tests covering the full lifecycle: upload → curate → approve → publish, including error scenarios and edge cases.

## Steps
1. Set up test infrastructure: test database with migrations, mocked S3, mocked AI provider, mocked social platform APIs, mocked Signal client. 2. Test full happy path: upload images → AI curation creates drafts → approve via endpoint → publish to all platforms → verify in published list → verify portfolio manifest. 3. Test Signal approval flow: verify Signal message sent on draft creation, approve via Signal message, reject via Signal message. 4. Test error scenarios: upload invalid file types, approve already-rejected draft, publish unapproved draft, AI service unavailable, all platforms fail during publish. 5. Test pagination on GET /drafts and GET /published endpoints. 6. Test Effect.Schema validation: malformed requests return proper 400 errors with field-level details. 7. Verify >80% code coverage across all modules. 8. Test concurrent operations: multiple publishes in parallel don't create duplicate published_posts records.

## Validation
All tests pass with >80% code coverage. Full lifecycle test completes without errors. Error scenarios return appropriate HTTP status codes. No duplicate records created under concurrent access.