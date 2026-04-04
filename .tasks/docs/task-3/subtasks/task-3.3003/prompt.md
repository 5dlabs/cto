Implement subtask 3003: Implement NousProvider concrete implementation

## Objective
Implement NousProvider that calls the external NOUS API using NOUS_API_KEY with a 30-second timeout and returns structured research results following the ResearchProvider interface.

## Steps
1. Create `src/research/providers/nous-provider.ts`.
2. Constructor takes `{ apiKey: string; baseUrl?: string }`. Default base URL should be read from `process.env.NOUS_API_BASE_URL` or a sensible default.
3. `execute()` method: POST to `${baseUrl}/research` (or appropriate NOUS endpoint) with JSON body and `Authorization: Bearer ${apiKey}` header. Use `AbortSignal.timeout(30_000)`.
4. On success (2xx), parse response and return `ResearchResult` with `provider: 'nous'`, `skipped: false`.
5. On non-2xx or timeout, throw `ResearchProviderError` with provider name 'nous'.
6. Ensure the API key is never logged — use `[REDACTED]` in any log statements that reference the key.

## Validation
Unit test with mocked fetch: (1) Successful 200 response returns ResearchResult with provider='nous'. (2) 401 response throws ResearchProviderError. (3) Timeout throws ResearchProviderError. (4) Verify Authorization header is sent with correct Bearer token. (5) Verify no log output contains the raw API key value.