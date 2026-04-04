Implement subtask 5001: Implement base NotificationService module with resilient HTTP client and retry logic

## Objective
Create the foundational NotificationService class/module that provides a resilient HTTP POST helper with 1-retry / 2-second-backoff semantics, structured logging for every call (bridge URL, payload size, response status, latency), and graceful degradation (never throws on notification failure). Read DISCORD_BRIDGE_URL and LINEAR_BRIDGE_URL from the sigma-1-infra-endpoints ConfigMap via envFrom.

## Steps
1. Create `src/services/notification.service.ts`.
2. In the constructor / init, read `DISCORD_BRIDGE_URL` and `LINEAR_BRIDGE_URL` from `process.env` (populated via ConfigMap envFrom).
3. Implement a private `postWithRetry(url: string, payload: unknown): Promise<{ok: boolean; status?: number; latencyMs: number}>` method:
   - Uses `fetch` (Bun-native) to POST JSON.
   - On 5xx or network error, wait 2 seconds and retry once.
   - After final failure, log a structured warning (level: warn) with bridge URL, error message, and latency — then return `{ok: false}`.
4. Wrap every call in a try/catch so no notification failure can propagate an exception to the caller.
5. Add structured JSON logging (e.g., via `console.log(JSON.stringify({...}))` or a lightweight logger) capturing: event type, bridge URL, payload byte size, HTTP status, latency in ms.
6. Export the singleton or factory for use by Discord and Linear integration modules.

## Validation
Unit test: mock fetch to return 200 — verify postWithRetry returns {ok: true} and logs include status 200 and latency. Unit test: mock fetch to return 503 twice — verify exactly 2 fetch calls occur (initial + 1 retry), a warning is logged, and the method returns {ok: false} without throwing. Unit test: mock fetch to throw a network error — verify retry occurs and warning is logged. Unit test: verify DISCORD_BRIDGE_URL and LINEAR_BRIDGE_URL are read from process.env.