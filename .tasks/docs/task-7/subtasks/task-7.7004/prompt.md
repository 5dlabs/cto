Implement subtask 7004: Implement web chat widget channel

## Objective
Build the web chat interface channel that Morgan uses to communicate with website visitors, implementing WebSocket-based real-time messaging, session management, and a lightweight embeddable widget API.

## Steps
Step 1: Implement a WebSocket server endpoint for real-time web chat communication. Step 2: Define the chat protocol: message format (JSON with type, content, timestamp, session_id), handshake, heartbeat, and disconnect handling. Step 3: Implement session management — create sessions on first connection, maintain context across messages, handle session expiry/timeout. Step 4: Wire the web chat channel into the OpenClaw agent runtime as an input/output channel. Step 5: Expose an HTTP endpoint for widget initialization that returns session tokens and WebSocket URLs. Step 6: Implement typing indicators and message delivery confirmations. Step 7: Add CORS configuration for cross-origin widget embedding from the Next.js frontend.

## Validation
Connect to the WebSocket endpoint and exchange messages with Morgan; verify session persistence across multiple messages; confirm CORS headers allow embedding from the frontend domain; typing indicators are sent during agent processing.