Implement subtask 3007: Add structured logging across all research providers and selection logic

## Objective
Ensure every decision point, fallback transition, timing measurement, and error in the research integration emits structured JSON log entries with consistent fields.

## Steps
1. Define a consistent log schema for research events in `src/research/logging.ts`:
   - `{ component: 'research', event: string, provider?: string, duration_ms?: number, content_length?: number, reason?: string, level: 'info' | 'warn' | 'error' }`
2. Create a `researchLogger` helper that wraps the project's logger with the `component: 'research'` prefix.
3. Audit all research modules (hermes-discovery, each provider, provider-selector, memo-writer) and replace any ad-hoc console.log/warn calls with structured `researchLogger` calls.
4. Ensure these events are logged:
   - `discovery.resolved` — which source provided the Hermes URL
   - `discovery.failed` — no Hermes endpoint found
   - `provider.selected` — which provider was chosen and why
   - `provider.success` — provider returned, with duration and content length
   - `provider.error` — provider failed, with error message and status code
   - `provider.fallback` — transitioning from one provider to the next
   - `pipeline.complete` — research phase finished, summary stats
5. Verify no sensitive data (API keys) appears in any log entry.

## Validation
Unit test: (1) Trigger each log event path and capture log output — verify JSON structure matches the defined schema. (2) Verify provider.error log includes error message but not API key. (3) Verify pipeline.complete log includes duration_ms and provider name. (4) Run full research flow with mocked providers and verify the expected sequence of log events appears in order.