Implement subtask 6013: Write end-to-end integration tests for full social media workflow

## Objective
Create comprehensive integration tests covering the complete flow: upload → AI curation → draft creation → approval → publish → portfolio sync, with mocked external services.

## Steps
1. Set up test harness with test PostgreSQL database and mocked S3 (using localstack or mock). 2. Mock all external services: AI API (OpenAI/Claude), Instagram API, LinkedIn API, TikTok API, Facebook API, Signal API, website portfolio API. 3. Test scenarios: a) Happy path: upload images → create draft (AI generates captions) → approve → publish to all platforms → portfolio synced. b) Rejection flow: upload → create draft → reject → verify cannot publish. c) Partial publish failure: one platform fails, others succeed. d) Invalid upload: wrong file type rejected. e) State machine validation: cannot publish unapproved, cannot approve already published. f) Effect.Schema validation: malformed requests are rejected with proper error messages. 4. Verify database state at each step. 5. Verify all mock services received expected calls.

## Validation
All test scenarios pass end-to-end; database state is correct after each step; all mock services receive expected API calls with correct payloads; Effect.Schema validation prevents invalid data; state transitions are enforced; no test pollution between runs.