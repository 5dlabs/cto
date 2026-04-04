Implement subtask 5004: Implement best-effort error handling for notification dispatch

## Objective
Wrap the transport send() calls with error handling that catches connection refused, timeouts, and HTTP error responses (4xx/5xx), logging warnings without throwing or failing the pipeline.

## Steps
1. In the notify() facade, wrap each transport.send() call in a try/catch block.
2. On catch (connection refused, timeout, network error): log a structured warning with `{ service: 'discord' | 'linear', error: message, pipeline_id }` and resolve the promise normally.
3. On non-2xx HTTP response: log a structured warning with `{ service, statusCode, pipeline_id }` and resolve normally.
4. Use `Promise.allSettled()` to dispatch to both bridges concurrently and handle each independently — a failure in one bridge must not prevent notification to the other.
5. Ensure the notify() function always resolves (never rejects) regardless of transport errors.

## Validation
1. Unit test: When DISCORD_BRIDGE_URL is unreachable (mock fetch to throw connection refused), notify() logs a warning containing 'discord' and resolves without throwing. 2. Unit test: When LINEAR_BRIDGE_URL returns HTTP 500, notify() logs a warning containing 'linear' and status 500, and resolves without throwing. 3. Unit test: When both bridges fail, notify() still resolves and both warnings are logged.