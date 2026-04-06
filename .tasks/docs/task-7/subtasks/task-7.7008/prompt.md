Implement subtask 7008: Implement web chat widget endpoint

## Objective
Create the HTTP/WebSocket endpoint that the website's embedded chat widget will connect to for real-time conversations with the Morgan agent.

## Steps
1. Implement a WebSocket (or SSE) endpoint on the OpenClaw agent's service for web chat connections.
2. Define the message protocol: JSON messages with fields for sender, text, timestamp, session ID.
3. Implement session management: create/resume chat sessions, maintain conversation context.
4. Route incoming web chat messages to the agent's conversation engine.
5. Stream agent responses back to the client in real time.
6. Expose the endpoint via a Kubernetes Service on a well-known port.
7. Add CORS configuration to allow connections from the frontend domain.
8. Implement basic rate limiting per session to prevent abuse.
9. Document the endpoint URL and protocol for the frontend team (Blaze).

## Validation
Connect to the WebSocket endpoint from a test client; send a text message; verify a response is streamed back; verify session persistence across multiple messages; CORS headers allow the frontend origin; concurrent sessions are isolated; connection handles graceful disconnect.