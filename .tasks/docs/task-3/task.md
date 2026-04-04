## Integrate Hermes Research in Deliberation Path (Nova - Bun/Elysia)

### Objective
Extend the PM server's deliberation stage to call the Hermes SaaS API for research content generation. The integration must include a circuit breaker with configurable timeout (default 30s) to prevent external API latency from blocking the Bun event loop. When Hermes is unavailable or the circuit is open, the system must fall back to default research memo behavior without blocking the pipeline.

### Ownership
- Agent: nova
- Stack: Bun/Elysia
- Priority: high
- Status: pending
- Dependencies: 1

### Implementation Details
1. Add the Hermes API client module: accepts a research query string, calls the Hermes SaaS endpoint with `NOUS_API_KEY` from the external-secrets-managed secret, returns structured research content.
2. Implement a circuit breaker around the Hermes client. Evaluate existing dependencies first — check if `opossum` or a similar library is already in `package.json`. If not, add the lightest viable option (`opossum` is ~50KB, well-maintained for Node-compatible runtimes). Configure: timeout = 30s (from env `HERMES_TIMEOUT_MS` with default 30000), failure threshold = 3 consecutive failures, reset timeout = 60s.
3. Integrate the Hermes call into the deliberation pipeline stage: after initial PRD parsing and before task generation, invoke the Hermes client with a research query derived from the PRD content. Store the response in the deliberation path artifacts.
4. Implement fallback behavior: when the circuit is open or Hermes returns an error, generate a default research memo with `{ source: 'fallback', reason: 'hermes_unavailable' | 'circuit_open' | 'timeout', content: null }`. Log at `warn` level with structured fields.
5. Implement availability gating: if `NOUS_API_KEY` is not set or empty, skip the Hermes call entirely and log `{ stage: 'hermes_research', action: 'skipped', reason: 'no_api_key' }`.
6. Ensure the Hermes response is persisted in the deliberation output (DB or in-memory pipeline state) so Task 8 can validate its content.
7. Add a health check sub-path or status field to `GET /api/pipeline/status` that reports the Hermes circuit breaker state (closed/open/half-open).

### Subtasks
- [ ] Implement Hermes API client module with NOUS_API_KEY authentication: Create a standalone TypeScript module that encapsulates all HTTP communication with the Hermes SaaS API, including authentication, request formatting, and response parsing into a structured research content type.
- [ ] Implement circuit breaker wrapper with configurable timeout, failure threshold, and reset: Create a reusable circuit breaker module that wraps async functions with open/closed/half-open state management, configurable via environment variables. Evaluate opossum vs. cockatiel vs. a lightweight custom implementation for Bun compatibility.
- [ ] Implement availability gating for missing NOUS_API_KEY: Add a pre-check that skips the Hermes research call entirely when NOUS_API_KEY is not set or empty, logging a structured skip event and producing a skip-type memo.
- [ ] Implement fallback behavior for circuit-open, timeout, and error scenarios: Create the fallback memo generator that produces a structured default research memo when the Hermes call fails for any reason, with differentiated reason codes and warn-level structured logging.
- [ ] Integrate Hermes research call into the deliberation pipeline stage: Wire the Hermes research orchestrator (client + circuit breaker + availability gate + fallback) into the existing deliberation pipeline, invoking it after PRD parsing and before task generation, and persisting the research memo in the deliberation artifacts.
- [ ] Add circuit breaker state to GET /api/pipeline/status health check: Extend the existing pipeline status endpoint to include the Hermes circuit breaker state (closed/open/half-open) in its response payload.
- [ ] Write comprehensive test suite for the Hermes integration: Create end-to-end and edge-case tests covering the full Hermes research integration: happy path, timeout, circuit breaker state transitions, no-API-key gating, and artifact persistence.