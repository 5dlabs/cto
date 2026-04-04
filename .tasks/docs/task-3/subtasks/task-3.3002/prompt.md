Implement subtask 3002: Implement ResearchProvider interface and HermesProvider concrete implementation

## Objective
Define the ResearchProvider abstraction (TypeScript interface) and implement HermesProvider that calls the in-cluster Hermes agent with a 30-second timeout and returns structured research results.

## Steps
1. Create `src/research/types.ts` with:
   - `ResearchResult = { content: string; provider: 'hermes' | 'nous' | 'skip'; skipped: boolean; reason?: string; responseTimeMs: number; contentLength: number }`
   - `ResearchProvider = { name: string; execute(query: ResearchQuery): Promise<ResearchResult> }`
   - `ResearchQuery = { prdContent: string; projectContext?: string }`
2. Create `src/research/providers/hermes-provider.ts`.
3. HermesProvider constructor takes `{ baseUrl: string }`.
4. `execute()` method: POST to `${baseUrl}/research` with JSON body `{ query: researchQuery.prdContent, context: researchQuery.projectContext }`. Use `AbortSignal.timeout(30_000)` for the 30-second timeout.
5. On success (2xx), parse JSON response body and return `ResearchResult` with `provider: 'hermes'`, `skipped: false`, measured response time, and content length.
6. On non-2xx or timeout, throw a typed error `ResearchProviderError` with the status code and provider name — the caller (selector) will handle fallback.
7. Export `ResearchProviderError` class extending Error with `providerName` and `statusCode` fields.

## Validation
Unit test with mocked fetch: (1) Successful 200 response returns ResearchResult with provider='hermes', skipped=false, and correct content. (2) 500 response throws ResearchProviderError with statusCode=500. (3) Timeout (simulated via AbortSignal) throws ResearchProviderError. (4) Verify responseTimeMs is a positive number and contentLength matches actual content length.