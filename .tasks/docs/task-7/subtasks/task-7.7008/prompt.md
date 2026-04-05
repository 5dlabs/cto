Implement subtask 7008: Deploy Signal-CLI sidecar container with persistent volume and webhook routing

## Objective
Configure and deploy the signal-cli-rest-api as a sidecar container in the Morgan pod, including persistent volume for device registration data, inbound webhook configuration, and outbound message sending integration.

## Steps
1. Add signal-cli-rest-api container to the Morgan pod spec as a sidecar:
   - Image: `bbernhard/signal-cli-rest-api:latest` (or pinned version)
   - Ports: 8080 (REST API)
   - Resource limits: 512Mi memory, 250m CPU
2. Create PersistentVolumeClaim for Signal-CLI data:
   - Mount path: `/home/.local/share/signal-cli` (default signal-cli data dir)
   - Size: 1Gi (stores device registration, keys, message history)
   - Access mode: ReadWriteOnce
3. Configure signal-cli webhook for incoming messages:
   - Set environment variable `SIGNAL_CLI_WEBHOOK_URL=http://localhost:<morgan-port>/api/signal/incoming`
   - This routes all incoming Signal messages to Morgan's HTTP handler
4. Implement Morgan-side HTTP endpoint `/api/signal/incoming`:
   - Parse incoming webhook payload: { envelope: { source, sourceDevice, message: { text, attachments } } }
   - Extract sender phone number, message text, any attachments
   - Route to conversation state manager with channel='signal'
5. Implement outbound message function:
   - Morgan agent → HTTP POST to `http://localhost:8080/v2/send` on signal-cli sidecar
   - Payload: { message: string, number: string, attachments?: [base64] }
   - Handle send failures gracefully (queue for retry)
6. Configure liveness probe for signal-cli: GET /v1/about should return registration status.
7. Document the one-time device registration process (requires QR code scan or phone number verification) — this must be done manually before the agent can send/receive messages.

## Validation
Verify signal-cli sidecar starts and /v1/about returns 200 with registration info. Send a test POST to /v2/send and verify it's accepted (or returns expected error if not registered). Simulate incoming webhook to /api/signal/incoming and verify Morgan's handler parses the payload correctly. Verify PVC is mounted and persists across pod restarts.