Implement subtask 3005: Write comprehensive unit and integration tests for hermes-research module

## Objective
Create the full test suite covering all fetchResearchMemo paths and the pipeline integration, using Bun's test runner and mocked HTTP responses.

## Steps
1. Create `src/hermes-research/__tests__/fetchResearchMemo.test.ts`.
2. Unit test - success path: Mock fetch to return a 200 response with valid JSON. Assert fetchResearchMemo returns a ResearchMemo with non-empty content, source, and valid Date timestamp.
3. Unit test - missing API key: Temporarily unset NOUS_API_KEY in the test env. Assert return is null. Assert the info log message 'Hermes integration skipped: NOUS_API_KEY not configured' was emitted (spy on logger).
4. Unit test - timeout: Mock fetch to delay beyond 30 seconds (use a fake timer or AbortController mock). Assert return is null and warning is logged.
5. Unit test - HTTP error: Mock fetch to return 500. Assert return is null and warning with status code is logged.
6. Create `src/hermes-research/__tests__/pipeline-integration.test.ts`. Set NOUS_API_KEY, mock the Hermes API globally, run the deliberation pipeline with sample task data, and verify the output tasks have populated research_memo fields.
7. Ensure all tests clean up environment variables and mocks properly.

## Validation
All 5 test cases pass: (1) success returns valid ResearchMemo, (2) missing key returns null with correct log, (3) timeout returns null with warning, (4) HTTP 500 returns null with warning, (5) integration test shows pipeline output with non-null research_memo. Run `bun test` and verify 100% of tests pass with no flaky behavior.