Implement subtask 8001: Implement Hermes primary path validation tests (content presence, quality, and circuit breaker)

## Objective
Create the test file `e2e/hermes-research-validation.test.ts` and implement the test cases that validate Hermes-sourced research content when NOUS_API_KEY is available. This includes asserting memo presence with `source: 'hermes'`, content quality (length > 100), circuit breaker `closed` state, and absence of `action: 'skipped'` log entries for the hermes_research stage.

## Steps
1. Create `e2e/hermes-research-validation.test.ts` with a top-level `describe('Hermes Research Memo Validation')` block.
2. Add a helper function `hasApiKey()` that checks `process.env.NOUS_API_KEY` and returns a boolean.
3. Inside a `describe.runIf(hasApiKey())('Primary path — NOUS_API_KEY present')` block (or equivalent conditional using the test framework's skip/conditional API):
   a. **Test: Hermes content present** — Call `GET /api/pipeline/runs/{runId}/deliberation`. Parse the response JSON. Assert at least one item in the deliberation artifacts array has `source === 'hermes'` and a non-empty `content` field.
   b. **Test: Content quality** — For each memo with `source: 'hermes'`, assert `typeof content === 'string'` and `content.length > 100`. This ensures no placeholder or error stub.
   c. **Test: Circuit breaker closed** — Call `GET /api/pipeline/status`. Assert the response contains a Hermes circuit breaker entry with `state === 'closed'`.
   d. **Test: No skipped log entries** — Query or capture PM server logs (via log endpoint or captured stdout). Assert that entries matching `stage: 'hermes_research'` do NOT contain `action: 'skipped'`.
4. Use a `beforeAll` hook to trigger or retrieve the latest pipeline run ID needed for deliberation artifact queries.
5. Add appropriate timeout configuration for API calls (e.g., 30s for e2e).

## Validation
Run the test suite with NOUS_API_KEY set to a valid key. All four assertions (content presence, quality > 100 chars, circuit breaker closed, no skipped logs) must pass. Verify each test case produces a clear pass/fail and descriptive error messages on failure.