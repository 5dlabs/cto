Implement subtask 3001: Implement Hermes API client module with NOUS_API_KEY authentication

## Objective
Create a standalone TypeScript module that encapsulates all HTTP communication with the Hermes SaaS API, including authentication, request formatting, and response parsing into a structured research content type.

## Steps
1. Create `src/clients/hermes.ts` exporting a `HermesClient` class (or factory function).
2. Define TypeScript interfaces: `HermesResearchRequest { query: string; context?: string }`, `HermesResearchResponse { source: 'hermes'; content: string; metadata?: Record<string, unknown> }`.
3. Read `NOUS_API_KEY` from environment (injected via external-secrets). Accept the API base URL from `HERMES_API_URL` env var with a sensible default.
4. Implement `async research(query: string): Promise<HermesResearchResponse>` — constructs the HTTP request using `fetch` (Bun-native), sets `Authorization: Bearer ${apiKey}` header (or `x-api-key` depending on resolved API contract), sends a POST with the query payload, and parses the JSON response into the typed interface.
5. Handle HTTP-level errors (non-2xx status codes) by throwing typed errors (`HermesApiError`) that include status code and response body for upstream handling.
6. Export the interfaces and error types so the circuit breaker and pipeline modules can import them.

## Validation
Unit test with mocked fetch: (1) mock a 200 response with valid JSON, assert returned object matches `HermesResearchResponse` shape with `source: 'hermes'`. (2) Mock a 401 response, assert `HermesApiError` is thrown with status 401. (3) Mock a 500 response, assert `HermesApiError` is thrown. (4) Mock a malformed JSON response, assert a parse error is thrown.