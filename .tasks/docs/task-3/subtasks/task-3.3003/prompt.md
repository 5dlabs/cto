Implement subtask 3003: Write comprehensive unit and integration tests for Hermes research integration

## Objective
Create test files covering all branches: API available with valid results, API key missing, timeout handling, low-relevance filtering, empty results, and artifact persistence.

## Steps
1. Create `src/deliberation/__tests__/hermes-client.test.ts`:
   a. Test: NOUS_API_KEY set, mock fetch returns 5 results (3 with score >= 0.5, 2 below). Assert returned array has exactly 3 items.
   b. Test: NOUS_API_KEY not set. Assert function returns null. Assert console/logger output includes info-level skip message.
   c. Test: Mock fetch to throw AbortError after 30s timeout. Assert function returns null and logs warning.
   d. Test: Mock fetch to return 500 error. Assert function returns null (graceful degradation).
   e. Test: Mock fetch returns empty array. Assert function returns empty array (not null).
   f. Test: Mock fetch returns malformed JSON. Assert function returns null and logs error.
2. Create `src/deliberation/__tests__/research-memo.test.ts`:
   a. Test: Format 2 results, verify Markdown output matches expected structure with header and bullet points.
   b. Test: Empty array input returns empty string.
   c. Test: Verify special characters in title/summary are not corrupted in output.
3. Create `src/deliberation/__tests__/deliberation-hermes-integration.test.ts`:
   a. Test: Full pipeline with mocked Hermes returning results — deliberation output contains '## Hermes Research Findings'.
   b. Test: Full pipeline with NOUS_API_KEY unset — deliberation output does NOT contain '## Hermes Research Findings' and no errors thrown.
   c. Test: Verify `hermes-raw-response.json` is written to a temp artifacts directory.
   d. Test: Verify artifacts directory is created if it doesn't exist.
4. Use Bun's built-in test runner (`bun:test`). Mock `fetch` using `mock()` from `bun:test` or a helper that replaces `globalThis.fetch`.

## Validation
Run `bun test` on all three test files. All tests must pass. Verify coverage of: happy path, missing API key, timeout, HTTP error, malformed response, empty results, relevance filtering threshold, memo formatting, artifact file creation, and directory creation.