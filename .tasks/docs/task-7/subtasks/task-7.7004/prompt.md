Implement subtask 7004: Implement web chat widget endpoint for website integration

## Objective
Create the web chat HTTP/WebSocket endpoint that the Sigma-1 website chat widget will connect to, enabling real-time bidirectional messaging with the Morgan agent.

## Steps
1. Implement a WebSocket endpoint (e.g., /ws/chat) for real-time bidirectional communication.
2. Implement a REST fallback endpoint (e.g., POST /api/chat) for environments where WebSocket is unavailable.
3. Define the message protocol: { type, content, sessionId, timestamp, metadata }.
4. Implement session creation and management for web chat users (anonymous sessions with optional identification).
5. Handle message routing: web chat input → agent conversation loop → response back to WebSocket/REST.
6. Implement CORS configuration to allow requests from the Sigma-1 website domain.
7. Add rate limiting per session to prevent abuse.

## Validation
Connect to WebSocket endpoint from a test client; send a message and receive a coherent agent response; verify session persistence across multiple messages; verify CORS headers are correctly set; REST fallback returns proper response.