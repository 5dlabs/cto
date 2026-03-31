Implement subtask 7002: Integrate Signal-CLI messaging channel

## Objective
Configure Signal-CLI as a messaging channel for Morgan so customers can interact via Signal. Set up the Signal-CLI process (sidecar or separate pod), register/link the phone number, and wire inbound/outbound message flow to the OpenClaw agent.

## Steps
1. Deploy Signal-CLI (self-hosted) as a sidecar container or dedicated pod alongside the Morgan agent.
2. Register or link the designated business phone number with Signal-CLI.
3. Configure Signal-CLI to expose a JSON-RPC or REST interface for sending and receiving messages.
4. Implement the OpenClaw channel adapter that: (a) listens for inbound Signal messages, (b) forwards them to the Morgan agent as user turns, (c) sends agent responses back through Signal-CLI.
5. Handle message types: text, and gracefully reject unsupported media with a helpful reply.
6. Configure retry logic for transient Signal-CLI failures.
7. Verify round-trip message flow: send a Signal message → receive agent response on Signal.

## Validation
Send a test Signal message to the registered number and receive an intelligent agent response within 10 seconds. Verify inbound messages are logged. Verify outbound messages are delivered. Confirm the adapter reconnects after a simulated Signal-CLI restart.