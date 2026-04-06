Implement subtask 7002: Integrate Signal-CLI sidecar for Signal messaging channel

## Objective
Configure and deploy a Signal-CLI sidecar container alongside the OpenClaw agent pod to enable sending and receiving Signal messages. Wire the sidecar's REST/JSON-RPC API to the agent's messaging interface.

## Steps
1. Add a Signal-CLI sidecar container to the OpenClaw Deployment manifest.
2. Mount a persistent volume or secret for the Signal-CLI account data (phone number registration, keys).
3. Configure Signal-CLI to expose its JSON-RPC or REST API on localhost within the pod.
4. Implement the agent's inbound message handler: listen for incoming Signal messages via the sidecar API, parse them, and route to the OpenClaw agent's conversation endpoint.
5. Implement the agent's outbound message handler: take agent responses and call Signal-CLI send API.
6. Handle message types: text, images/attachments (at minimum text for v1).
7. Add liveness/readiness probes for the Signal-CLI sidecar.
8. Configure environment variables for Signal phone number, API port, etc.

## Validation
Send a Signal message to the registered number; verify the agent receives it (check agent logs); verify the agent sends a response back via Signal; sidecar readiness probe passes; round-trip message delivery completes within 15 seconds.