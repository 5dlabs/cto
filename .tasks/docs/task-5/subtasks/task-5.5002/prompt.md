Implement subtask 5002: Implement HTTP POST transport for discord-bridge-http

## Objective
Implement the NotificationTransport interface for HTTP POST dispatch to the discord-bridge-http service. Format the payload per the Discord bridge's expected schema and POST to the URL from the ConfigMap.

## Steps
1. Create `src/notification-dispatch/transports/http-discord.ts` implementing the `send()` method for the 'discord' target.
2. Use Bun's native `fetch()` to POST to `DISCORD_BRIDGE_URL` with `Content-Type: application/json`.
3. Format the JSON body as `{ event, pipeline_id, status, task_count, assigned_count, pr_url, linear_session_url, timestamp }`.
4. Set a reasonable timeout (5 seconds) on the fetch to avoid hanging on unresponsive bridges.
5. Return the response status for the error handling layer to inspect.

## Validation
Unit test: Call the Discord HTTP transport's send() with a pipeline.start event. Verify the outgoing HTTP POST targets the correct URL with the correct JSON body shape including all required fields (event, pipeline_id, timestamp).