Implement subtask 7008: Set up web chat endpoint for Morgan agent

## Objective
Expose a WebSocket (or HTTP streaming) endpoint from the Morgan agent for the web chat frontend to connect to, enabling real-time conversational interaction via browser.

## Steps
1. Implement a WebSocket endpoint on the Morgan agent (e.g., /ws/chat) that accepts browser connections.
2. Define the message protocol: JSON messages with fields for session_id, message_text, message_type (user/agent), and metadata.
3. Handle session management: create new sessions, resume existing sessions by session_id.
4. Stream agent responses token-by-token or chunk-by-chunk for real-time feel.
5. Implement CORS configuration to allow connections from the frontend domain.
6. Create an Ingress rule or update the existing Service to expose the WebSocket endpoint externally.
7. Handle graceful disconnection and reconnection.

## Validation
Connect to the WebSocket endpoint using wscat or a browser test page. Send a message and verify a streamed response is received. Disconnect and reconnect with the same session_id; verify session continuity. Verify CORS headers allow frontend origin.