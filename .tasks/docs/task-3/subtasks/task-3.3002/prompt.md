Implement subtask 3002: Implement Hermes API client with NOUS_API_KEY reading and 30s timeout

## Objective
Create the core hermes-research module with the fetchResearchMemo function that reads NOUS_API_KEY from environment, calls the Hermes/NOUS API with the task context, and parses the response into a ResearchMemo.

## Steps
1. Create `src/hermes-research/index.ts` exporting `async function fetchResearchMemo(taskContext: TaskContext): Promise<ResearchMemo | null>`.
2. Read `NOUS_API_KEY` from `process.env.NOUS_API_KEY` (or `Bun.env`).
3. Construct the HTTP request to the Hermes API: use `fetch()` (native in Bun) with the API key in the Authorization header (Bearer token or whatever the API expects).
4. Send the task description and context as the research query in the request body.
5. Set `AbortController` with a 30-second timeout on the fetch call.
6. On successful response, parse the JSON body and map it to `ResearchMemo`: store the raw response content verbatim in `content`, set `source` to the API endpoint or identifier, and `timestamp` to the current Date.
7. Ensure the module interface is clean with a single exported function and no side effects on import, suitable for future extraction into a separate service.

## Validation
Unit test with a mocked Hermes API (using Bun's test utilities or a mock server) returning valid JSON content: verify fetchResearchMemo returns a ResearchMemo with non-empty content, correct source string, and a valid Date timestamp.