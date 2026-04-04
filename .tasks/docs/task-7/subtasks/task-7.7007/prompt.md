Implement subtask 7007: Implement Signal messaging integration (inbound/outbound via Signal-CLI REST API)

## Objective
Build the Signal channel adapter that connects Morgan to the Signal-CLI sidecar REST API for receiving and sending messages, handling photos, and distinguishing group vs direct messages.

## Steps
1. Implement inbound message listener:
   a. Poll Signal-CLI REST API at `localhost:8080/v1/receive/{number}` on a configurable interval (1-2 seconds).
   b. Parse incoming messages: extract sender, timestamp, body text, attachments (photos), group info.
   c. Determine message type: direct message (1:1 with Morgan) vs group message (only respond when @mentioned or directly addressed).
   d. For direct messages: forward full message body to Morgan's agent processing pipeline.
   e. For group messages: detect if Morgan is being addressed, extract relevant content, forward to agent.
2. Implement outbound message sending:
   a. Send text responses via `POST /v2/send` with recipient number and message body.
   b. Support sending photos/attachments (e.g., quote PDFs, curated social media previews) via attachment API.
   c. Handle message length limits — split long messages if necessary.
3. Photo handling:
   a. When photos are received, download attachment from Signal-CLI, store in morgan-workspace PVC.
   b. Pass file path to social-media skill or other processing pipeline.
   c. For outbound photos (quote previews), upload to Signal-CLI attachment endpoint first, then reference in send.
4. Conversation context:
   a. Map Signal phone numbers to customer identities/conversation threads.
   b. Maintain conversation continuity across multiple messages from the same sender.
5. Error handling: retry failed sends (3 attempts with backoff), log failed messages for manual follow-up.

## Validation
Send a test message via Signal-CLI REST API to Morgan's number, verify it is received and parsed correctly within 2 seconds. Send a response and verify it arrives at the sender. Send a photo attachment and verify it is downloaded and stored in workspace PVC. Send a group message without @mention and verify Morgan ignores it; send with @mention and verify Morgan responds.