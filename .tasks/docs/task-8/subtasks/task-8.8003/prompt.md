Implement subtask 8003: Implement Discord webhook collector service for notification verification

## Objective
Build a lightweight HTTP server that acts as a Discord webhook collector — receives POST payloads, stores them in memory, and exposes a GET endpoint for the test suite to query received messages.

## Steps
1. Create `tests/e2e/discord-collector/server.ts` — a Bun HTTP server (port from `COLLECTOR_PORT` env, default 9876).
2. `POST /webhook` — accepts Discord webhook-format JSON payloads, stores them in an in-memory array with timestamps.
3. `GET /messages` — returns all collected messages as a JSON array.
4. `DELETE /messages` — clears collected messages (used between test runs for isolation).
5. `GET /health` — returns 200 with `{status: 'ok', messageCount: N}`.
6. The collector should be started as a background process before tests and stopped after. Add a helper `tests/e2e/helpers/collector.ts` that spawns/kills the process.
7. The PM server's Discord webhook URL must be configured to point to this collector during E2E runs (via PM_DISCORD_WEBHOOK_URL env var pointing to `http://localhost:9876/webhook` or the CI-accessible URL).
8. Document that in production the real Discord webhook is used; this collector is test-only.

## Validation
Start the collector, POST a sample Discord webhook payload to /webhook, GET /messages and verify the payload is returned. DELETE /messages and confirm GET /messages returns empty array. Verify /health returns correct messageCount.