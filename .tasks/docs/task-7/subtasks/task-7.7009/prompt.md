Implement subtask 7009: Implement web chat WebSocket endpoint with conversation persistence

## Objective
Build the WebSocket endpoint that the Next.js frontend connects to for real-time chat with Morgan, including JSON message protocol and conversation history persistence on the workspace PVC.

## Steps
1. WebSocket server:
   a. Expose a WebSocket endpoint at a configurable path (e.g., `/ws/chat`) within the Morgan agent container.
   b. Handle connection lifecycle: open, message, close, error.
   c. Authenticate connections: validate a session token or API key on connection handshake.
2. Message protocol:
   a. Inbound messages: `{"type": "user_message", "content": "text here", "metadata": {"session_id": "...", "timestamp": "..."}}`
   b. Outbound messages: `{"type": "agent_message", "content": "response text", "metadata": {"session_id": "...", "timestamp": "...", "tool_calls": [...]}}`
   c. Typing indicator: `{"type": "typing", "content": "", "metadata": {"active": true}}`
   d. Error messages: `{"type": "error", "content": "error description", "metadata": {"code": "..."}}`
3. Conversation persistence:
   a. Store conversation history per session_id in the morgan-workspace PVC (JSON files or SQLite).
   b. On reconnect, load previous conversation history so Morgan has context.
   c. Implement conversation TTL — expire old conversations after configurable period (e.g., 24 hours).
4. Concurrent connections:
   a. Support multiple simultaneous WebSocket connections (different customers).
   b. Isolate conversation state per session.
5. Forward messages to Morgan's agent pipeline, receive responses, and send back via WebSocket.

## Validation
Connect via WebSocket client, send a user_message JSON, verify an agent_message JSON response is received with correct structure within 10 seconds. Disconnect and reconnect with the same session_id, verify conversation history is preserved. Open 5 concurrent WebSocket connections and verify each maintains isolated conversation state.