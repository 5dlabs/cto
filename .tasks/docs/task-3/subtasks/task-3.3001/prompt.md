Implement subtask 3001: Implement Hermes API client with conditional NOUS_API_KEY check and timeout

## Objective
Create a TypeScript module that checks for the NOUS_API_KEY environment variable, constructs requests to the Hermes research endpoint, handles the 30-second timeout, and parses/filters the response by relevance score.

## Steps
1. Create a new file `src/deliberation/hermes-client.ts`.
2. Export a function `fetchHermesResearch(query: { title: string; description: string }): Promise<HermesResult[] | null>`.
3. At the top of the function, check `process.env.NOUS_API_KEY`. If not set, log an info-level message (e.g., 'NOUS_API_KEY not set, skipping Hermes research') and return `null`.
4. Construct the request URL from `process.env.NOUS_API_BASE` + `/research`.
5. Use `fetch` (Bun native) with:
   - Method: POST
   - Headers: `Authorization: Bearer ${NOUS_API_KEY}`, `Content-Type: application/json`
   - Body: `{ query: "${title} ${description}", max_results: 10 }`
   - Signal: `AbortSignal.timeout(30_000)`
6. Define a TypeScript interface `HermesResult { title: string; summary: string; url: string; relevance_score: number }`.
7. Parse the JSON response body, validate it is an array of `HermesResult` objects.
8. Filter the array to only include items where `relevance_score >= 0.5`.
9. Wrap the entire fetch+parse in a try/catch. On timeout (AbortError/TimeoutError), log a warning and return `null`. On any other error, log the error and return `null` (graceful degradation).
10. Return the filtered results array.

## Validation
Unit test: mock fetch to return valid HermesResult array with mixed relevance scores; verify only items with score >= 0.5 are returned. Unit test: unset NOUS_API_KEY, call function, verify it returns null and logs info message. Unit test: mock fetch to delay 35 seconds, verify function returns null and logs a timeout warning within ~30 seconds.