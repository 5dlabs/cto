Implement subtask 7002: Integrate Signal-CLI for inbound/outbound messaging

## Objective
Implement the Signal messaging channel for Morgan, connecting to Signal-CLI for receiving and sending messages, handling message parsing, conversation threading, and session management for concurrent users.

## Steps
Step 1: Set up the Signal-CLI client connection (REST API or JSON-RPC interface to the Signal-CLI pod/sidecar). Step 2: Implement an inbound message handler that receives Signal messages, extracts sender identity, message body, and any attachments. Step 3: Implement conversation session management — map Signal phone numbers to active conversation contexts. Step 4: Implement outbound message sending — format agent responses and send via Signal-CLI, supporting text, links, and basic formatting. Step 5: Handle Signal-specific edge cases: group messages (ignore or respond), delivery receipts, read receipts, typing indicators. Step 6: Implement connection pooling and reconnection logic to support 500+ concurrent Signal connections. Step 7: Add structured logging for all inbound/outbound Signal messages.

## Validation
Send a test Signal message to Morgan's number and receive a coherent response; verify conversation threading with multiple concurrent senders; confirm reconnection after Signal-CLI restart; log entries appear for each message exchange.