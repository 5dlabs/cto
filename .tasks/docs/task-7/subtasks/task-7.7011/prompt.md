Implement subtask 7011: Implement web chat WebSocket endpoint with session management and streaming responses

## Objective
Build the /ws/chat WebSocket endpoint for real-time web chat, including Valkey-backed session management with 24h TTL, session continuity via tokens, and streaming LLM response tokens to the client.

## Steps
1. Implement WebSocket server endpoint at `/ws/chat`:
   - Accept WebSocket upgrade on connection
   - On connect: check for session_token query parameter or header
   - If session_token provided and valid (exists in Valkey): resume session, load conversation history
   - If no token or expired: create new session, generate UUID session_token, store in Valkey with 24h TTL
   - Send initial message to client: { type: 'session', session_token: string, resumed: boolean }
2. Define message protocol:
   - Client → Server: { type: 'user', content: string }
   - Server → Client: { type: 'agent_start', message_id: string } (signals start of response)
   - Server → Client: { type: 'agent_token', message_id: string, token: string } (streaming tokens)
   - Server → Client: { type: 'agent_end', message_id: string, content: string } (full response)
   - Server → Client: { type: 'error', message: string }
3. Implement streaming response delivery:
   - When Morgan generates a response via LLM, stream individual tokens to the WebSocket client
   - Buffer complete response and store in conversation history on completion
4. Session data structure in Valkey:
   - Key: `session:{session_token}`
   - Value: JSON { session_id, created_at, last_active, channel: 'web', messages: [{ role, content, timestamp }] }
   - TTL: 86400 seconds (24h), refreshed on each message
5. Implement connection lifecycle:
   - On disconnect: keep session alive in Valkey (client can reconnect)
   - On reconnect with token: send last N messages as history replay
   - Heartbeat ping/pong every 30 seconds to detect stale connections
6. Rate limiting: max 5 messages per 10 seconds per session to prevent abuse.

## Validation
Connect to /ws/chat via WebSocket client and verify session_token is returned. Send a user message and verify streaming tokens arrive followed by agent_end. Disconnect and reconnect with same session_token — verify conversation history is preserved. Verify Valkey TTL is set to 24h. Send 6 messages in 10 seconds and verify rate limiting kicks in.