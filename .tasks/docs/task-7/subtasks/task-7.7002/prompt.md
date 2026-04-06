Implement subtask 7002: Integrate Signal-CLI for bidirectional messaging

## Objective
Configure Signal-CLI as a sidecar or companion pod to enable Morgan to send and receive Signal messages from customers.

## Steps
1. Deploy Signal-CLI as a sidecar container within the Morgan agent pod (or as a separate Deployment with a shared Service).
2. Register/link a Signal phone number for the business using signal-cli's registration flow.
3. Implement an adapter/bridge that listens for incoming Signal messages (via signal-cli's JSON RPC or dbus interface) and forwards them to the OpenClaw agent's input endpoint.
4. Implement the outbound path: when the agent produces a response, send it back through signal-cli to the originating Signal user.
5. Handle message types: text, images (for equipment photos), and group messages if required.
6. Store the Signal data directory in a PersistentVolumeClaim so registration state survives pod restarts.
7. Add error handling for delivery failures and retry logic.

## Validation
Send a test Signal message to Morgan's number and verify a response is received within 10 seconds; verify message content is correct; restart the pod and confirm registration persists; test image sending/receiving.