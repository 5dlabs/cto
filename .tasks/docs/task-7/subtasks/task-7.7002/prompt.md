Implement subtask 7002: Integrate Signal-CLI for bidirectional messaging

## Objective
Set up Signal-CLI for Morgan to receive and send messages via Signal, including message parsing, response routing, and conversation state management.

## Steps
1. Deploy or configure Signal-CLI (self-hosted per PRD) and register a Signal phone number for Morgan.
2. Implement a message listener that polls or subscribes to incoming Signal messages from Signal-CLI's REST/JSON-RPC interface.
3. Parse incoming messages (text, attachments) and route them to the OpenClaw agent runtime as conversation inputs.
4. Implement a response handler that sends agent responses back via Signal-CLI's send API.
5. Handle conversation threading/state: map Signal sender numbers to ongoing conversation contexts.
6. Implement error handling for Signal-CLI connectivity issues (retry logic, dead-letter logging).
7. Add structured logging for all inbound/outbound Signal messages (sender, timestamp, message length, response latency).

## Validation
Send a test Signal message to Morgan's number and receive a coherent response within 10s; verify conversation context is maintained across multiple messages; logs show inbound and outbound message metadata; error handling triggers on simulated Signal-CLI downtime.