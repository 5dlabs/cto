Implement subtask 7002: Integrate Signal-CLI for messaging channel

## Objective
Deploy and configure Signal-CLI as a sidecar or separate pod, link it to the Morgan agent so inbound Signal messages are forwarded to the agent and responses are sent back via Signal.

## Steps
1. Deploy Signal-CLI REST API as a sidecar container in the agent pod (or as a separate Deployment if preferred per dp-8 decision).
2. Register/link the Signal phone number by providing the Signal account credentials or linking device.
3. Configure Signal-CLI to forward incoming messages to the Morgan agent's ingest endpoint via HTTP webhook.
4. Implement the outbound path: Morgan agent calls Signal-CLI REST API to send replies.
5. Handle group messages and direct messages.
6. Configure retry logic for message delivery failures.
7. Store Signal-CLI data directory on a persistent volume so registration survives restarts.

## Validation
Send a Signal message to the registered number; verify it appears in agent logs within 5 seconds. Agent responds and the reply is received on the Signal client. Signal-CLI pod restarts without losing registration.