Implement subtask 7008: Implement web chat widget endpoint

## Objective
Create a web chat endpoint that the frontend website can connect to for real-time chat with Morgan. Expose a WebSocket or SSE-based API that the web chat widget will consume.

## Steps
1. Implement a WebSocket (or SSE) endpoint on the Morgan agent service for web chat connections.
2. Define the chat protocol: message format (JSON with role, content, timestamp, conversation_id), connection handshake, and keepalive/ping.
3. Implement session management: create a new conversation_id on connect, maintain conversation history for the session duration.
4. Wire inbound web chat messages to the Morgan agent as user turns and stream agent responses back to the WebSocket.
5. Handle connection lifecycle: graceful disconnect, reconnection with conversation resumption, and timeout after inactivity.
6. Add CORS configuration for the expected frontend origin.
7. Document the endpoint URL and protocol for the frontend team (Blaze agent).

## Validation
Connect to the WebSocket endpoint using a test client (e.g., wscat). Send a chat message and verify an agent response is streamed back. Verify conversation_id is assigned and consistent across messages. Test reconnection: disconnect and reconnect with the same conversation_id, verify history is maintained. Test inactivity timeout.