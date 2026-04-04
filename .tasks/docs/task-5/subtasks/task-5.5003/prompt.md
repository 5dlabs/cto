Implement subtask 5003: Implement HTTP POST transport for linear-bridge

## Objective
Implement the NotificationTransport interface for HTTP POST dispatch to the linear-bridge service. Format the payload per the Linear bridge's expected schema and POST to the URL from the ConfigMap.

## Steps
1. Create `src/notification-dispatch/transports/http-linear.ts` implementing the `send()` method for the 'linear' target.
2. Use Bun's native `fetch()` to POST to `LINEAR_BRIDGE_URL` with `Content-Type: application/json`.
3. Format the JSON body with fields: `{ event, pipeline_id, status, task_count, assigned_count, pr_url, linear_session_url, timestamp }`. The Linear bridge may expect slightly different field names — check the bridge API contract and map accordingly.
4. Set a reasonable timeout (5 seconds) on the fetch.
5. Return the response status for the error handling layer to inspect.

## Validation
Unit test: Call the Linear HTTP transport's send() with a pipeline.complete event. Verify the outgoing HTTP POST targets LINEAR_BRIDGE_URL with JSON body containing task_count, assigned_count, pr_url, and linear_session_url.