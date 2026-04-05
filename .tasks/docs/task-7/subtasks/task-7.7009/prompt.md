Implement subtask 7009: Implement web chat widget integration

## Objective
Build the web chat widget interface and backend WebSocket/HTTP endpoint that connects website visitors to Morgan's agent runtime.

## Steps
1. Design the web chat widget API: define a WebSocket or HTTP streaming endpoint that the website chat widget will connect to.
2. Implement the chat backend handler: accept messages from the widget, route them to the OpenClaw agent runtime, and stream responses back.
3. Handle session management: create and maintain conversation sessions for anonymous website visitors with optional identification.
4. Implement typing indicators, message delivery status, and error messaging for the widget protocol.
5. Define the widget communication contract (JSON message format: { type, content, sessionId, timestamp }) so the Blaze frontend team can integrate.
6. Handle concurrent sessions and ensure isolation between different visitor conversations.
7. Add logging for web chat sessions (session ID, message count, duration, response latencies).

## Validation
WebSocket/HTTP endpoint accepts connections and exchanges messages; a test client can send a message and receive a Morgan response; sessions are isolated between concurrent clients; typing indicators and delivery status are emitted; logs capture session metadata and latencies.