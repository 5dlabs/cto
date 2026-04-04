Implement task 3: Integrate Hermes Research into Deliberation Path (Nova - Bun/Elysia)

## Goal
Add three-tier research integration to the cto/cto-pm deliberation path: in-cluster Hermes agent as primary, NOUS API as fallback when NOUS_API_KEY is present, and graceful skip with warning log if neither is available. Research is conditional enrichment — it must never block the pipeline.

## Task Context
- Agent owner: nova
- Stack: Bun/Elysia
- Priority: high
- Dependencies: 1

## Implementation Plan
Step-by-step implementation:

1. Discover the Hermes agent endpoint:
   - Check `HERMES_URL` from the `sigma-1-infra-endpoints` ConfigMap
   - If empty, attempt service discovery by querying `hermes.cto.svc.cluster.local` and `hermes.cto-tools.svc.cluster.local`
   - Log the discovered endpoint or log warning if none found

2. Implement a `ResearchProvider` abstraction in cto-pm with three strategies:
   - `HermesProvider`: calls in-cluster Hermes endpoint with a 30-second timeout
   - `NousProvider`: calls external NOUS API using `NOUS_API_KEY` from ExternalSecret, 30-second timeout
   - `SkipProvider`: returns empty research result with `{ skipped: true, reason: 'no research provider available' }`

3. Implement provider selection logic (circuit-breaker pattern):
   ```
   if (hermes_available && hermes_healthy) → HermesProvider
   else if (NOUS_API_KEY is non-empty) → NousProvider
   else → SkipProvider (log warning, continue pipeline)
   ```

4. Health check for Hermes: send a lightweight ping/health request before committing to Hermes for the full research call. Cache health status for 60 seconds.

5. Integrate into the deliberation path:
   - After PRD parsing and before task generation, invoke the selected research provider
   - Write research results to the deliberation output path as research memos
   - If SkipProvider is used, write a memo stating research was unavailable with timestamp

6. Error handling:
   - HermesProvider timeout or 5xx → fall through to NousProvider
   - NousProvider timeout or 5xx → fall through to SkipProvider
   - Never throw from the research integration — all errors are caught and logged

7. Add structured logging at each decision point: which provider was selected, response time, content length, or skip reason.

## Acceptance Criteria
1. Unit test: when Hermes endpoint is reachable and returns 200, HermesProvider is selected and research content is written to deliberation path. 2. Unit test: when Hermes is unreachable and NOUS_API_KEY is set, NousProvider is selected as fallback. 3. Unit test: when both Hermes is unreachable and NOUS_API_KEY is empty/missing, SkipProvider is selected — pipeline continues without error, and a warning log entry is emitted containing 'no research provider available'. 4. Unit test: HermesProvider timeout (simulate 30s+ response) triggers fallback to NousProvider. 5. Integration test: run deliberation path and verify a research memo file exists in the deliberation output directory — either with Hermes/NOUS content or with a skip notice. 6. Verify that the pipeline completes successfully even when both HERMES_URL is empty and NOUS_API_KEY is absent (pure skip scenario).

## Subtasks
- Implement Hermes endpoint discovery with ConfigMap and service discovery fallback: Create a module that resolves the Hermes agent endpoint URL by first checking the HERMES_URL environment variable (sourced from sigma-1-infra-endpoints ConfigMap), then falling back to Kubernetes service discovery at hermes.cto.svc.cluster.local and hermes.cto-tools.svc.cluster.local, and finally returning null if neither is reachable.
- Implement ResearchProvider interface and HermesProvider concrete implementation: Define the ResearchProvider abstraction (TypeScript interface) and implement HermesProvider that calls the in-cluster Hermes agent with a 30-second timeout and returns structured research results.
- Implement NousProvider concrete implementation: Implement NousProvider that calls the external NOUS API using NOUS_API_KEY with a 30-second timeout and returns structured research results following the ResearchProvider interface.
- Implement SkipProvider concrete implementation: Implement SkipProvider that returns an empty research result with skipped=true and a descriptive reason, logging a warning. This provider never throws.
- Implement provider selection logic with health check and cached health status: Create the provider selector that implements the three-tier fallback strategy: check Hermes health (cached for 60s), fall back to NOUS if API key present, then fall back to SkipProvider. Implements circuit-breaker-style cascading.
- Integrate research provider into the deliberation path and write research memos: Hook the research provider selector into the existing cto-pm deliberation pipeline — invoke it after PRD parsing and before task generation, write results as research memo files to the deliberation output directory.
- Add structured logging across all research providers and selection logic: Ensure every decision point, fallback transition, timing measurement, and error in the research integration emits structured JSON log entries with consistent fields.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.