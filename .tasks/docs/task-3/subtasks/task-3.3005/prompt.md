Implement subtask 3005: Implement provider selection logic with health check and cached health status

## Objective
Create the provider selector that implements the three-tier fallback strategy: check Hermes health (cached for 60s), fall back to NOUS if API key present, then fall back to SkipProvider. Implements circuit-breaker-style cascading.

## Steps
1. Create `src/research/provider-selector.ts`.
2. Export class `ResearchProviderSelector` with constructor taking `{ hermesUrl: string | null; nousApiKey: string | null }`.
3. Implement `private async checkHermesHealth(url: string): Promise<boolean>` — sends GET to `${url}/health` with 5-second timeout. Returns true on 2xx, false otherwise.
4. Implement a health cache: store `{ healthy: boolean; checkedAt: number }`. If `Date.now() - checkedAt < 60_000`, return cached value.
5. Export `async selectProvider(): Promise<ResearchProvider>` implementing the logic:
   - If hermesUrl is non-null and health check passes → return new HermesProvider({ baseUrl: hermesUrl })
   - Else if nousApiKey is non-null and non-empty → return new NousProvider({ apiKey: nousApiKey })
   - Else → return new SkipProvider()
6. Log at each decision point: `[research:selector] Selected provider: hermes|nous|skip (reason: ...)`.
7. Export `async executeWithFallback(query: ResearchQuery): Promise<ResearchResult>` that:
   - Calls selectProvider() to get primary provider
   - Tries primary.execute(query)
   - On ResearchProviderError from HermesProvider, invalidate health cache, try NousProvider if available, else SkipProvider
   - On ResearchProviderError from NousProvider, fall through to SkipProvider
   - Never throws — always returns a ResearchResult.

## Validation
Unit tests: (1) When Hermes health returns 200, HermesProvider is selected. (2) When Hermes health fails and NOUS_API_KEY is set, NousProvider is selected. (3) When both unavailable, SkipProvider is selected. (4) Health cache test: second call within 60s does not make HTTP request. (5) Health cache test: call after 60s makes a new HTTP request. (6) executeWithFallback: HermesProvider throws → NousProvider is tried. (7) executeWithFallback: both throw → SkipProvider result returned, no exception propagated. (8) Verify structured logs indicate selected provider and reason.