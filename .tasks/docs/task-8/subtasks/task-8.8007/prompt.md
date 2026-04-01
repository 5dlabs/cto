Implement subtask 8007: Implement Test Case 4: Hermes Research Integration with Conditional Assertions

## Objective
Write the E2E test that fetches deliberation artifacts and conditionally asserts the presence or absence of Hermes research findings based on NOUS_API_KEY availability.

## Steps
1. Test Case 4 (`it('includes Hermes research findings when NOUS_API_KEY is configured')`):
   a. Call `GET ${PM_SERVER_URL}/api/pipeline/${runId}/deliberation`.
   b. Parse response — expect an object with a `researchMemo` or similar field containing the deliberation text.
   c. If `NOUS_API_KEY` is set (check env):
      - Assert: `researchMemo` contains the string 'Hermes Research Findings' (case-insensitive or exact match per spec).
      - Assert: there is at least one finding entry (e.g., a list item or structured entry under that section).
      - Log the number of findings for debugging.
   d. If `NOUS_API_KEY` is NOT set:
      - Assert: `researchMemo` does NOT contain 'Hermes Research Findings'.
      - Assert: the deliberation response has no error fields related to Hermes/Nous.
2. This test validates graceful degradation: the pipeline should work with or without the Hermes research service.

## Validation
With NOUS_API_KEY set: test passes when research memo contains Hermes findings section with >= 1 entry. Without NOUS_API_KEY: test passes when memo lacks Hermes section and no Hermes-related errors are present.