Implement subtask 7002: Implement Signal-CLI integration for inbound/outbound messaging

## Objective
Configure Signal-CLI as a messaging channel adapter for the Morgan agent, enabling inbound message reception and outbound message sending via the Signal protocol.

## Steps
1. Deploy or configure Signal-CLI daemon alongside the agent (or as a sidecar container).
2. Register/link the Signal phone number using secrets for Signal credentials.
3. Implement the inbound message handler: receive Signal messages → parse → forward to the OpenClaw agent conversation loop.
4. Implement the outbound message handler: agent responses → format → send via Signal-CLI.
5. Handle media attachments (images from equipment catalog, quote PDFs) in Signal messages.
6. Configure retry logic for message delivery failures.
7. Reference Signal-CLI endpoint from sigma1-infra-endpoints ConfigMap if applicable.

## Validation
Send a test Signal message to the registered number; verify the agent receives it, processes it, and sends a coherent response back via Signal within 10 seconds; verify media attachment sending works.