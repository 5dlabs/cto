Implement subtask 7008: Implement web chat widget WebSocket endpoint

## Objective
Create the WebSocket/HTTP endpoint that the website frontend will use to embed Morgan as a real-time chat widget.

## Steps
1. Implement a WebSocket endpoint (e.g., /ws/chat) on the Morgan agent's HTTP server for real-time bidirectional messaging.
2. Support session management: each connecting client gets a unique session ID, with conversation history maintained for the session duration.
3. Implement a fallback HTTP polling endpoint (POST /chat) for environments where WebSocket is blocked.
4. Define the message protocol: JSON messages with fields for type (user_message, agent_response, typing_indicator, error), content, timestamp, and session_id.
5. Add CORS headers to allow connections from the website frontend domain.
6. Implement connection lifecycle: on connect, send a greeting; on disconnect, persist session state for potential reconnection.
7. Expose the endpoint via a Kubernetes Service (and eventually Ingress) for frontend access.

## Validation
Connect to the WebSocket endpoint using wscat or a test client; send a message and verify a response is received; verify session persistence across multiple messages; test CORS headers; test HTTP polling fallback.