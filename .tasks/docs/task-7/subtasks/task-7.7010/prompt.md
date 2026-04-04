Implement subtask 7010: Create Kubernetes deployment manifests for Morgan agent with Signal-CLI sidecar

## Objective
Write the Kubernetes Deployment, Service, PVC, and related manifests to deploy Morgan in the openclaw namespace with the Signal-CLI sidecar container, workspace PVC, API key secrets, and health probes.

## Steps
1. Deployment manifest:
   a. Namespace: `openclaw`
   b. Main container: Morgan agent image with workspace PVC mounted at `/data/workspace`.
   c. Sidecar container: Signal-CLI REST API image, resource limits 512Mi memory / 500m CPU, restartPolicy Always.
   d. Signal-CLI sidecar exposes port 8080 (localhost only, no Service exposure).
   e. Morgan agent exposes port 8081 for WebSocket chat and port 8082 for voice adapter HTTP endpoint.
2. PVC:
   a. `morgan-workspace` PVC: 10Gi, ReadWriteOnce, mounted in Morgan container.
   b. Signal-CLI data PVC: for Signal-CLI state/keys persistence across restarts.
3. Secrets:
   a. Mount `sigma1-service-api-keys` secret as environment variables for backend service authentication.
   b. Mount ElevenLabs API key secret.
   c. Mount Twilio credentials secret (account SID, auth token, phone number).
   d. Mount Signal-CLI registration credentials.
4. Services:
   a. ClusterIP Service exposing Morgan's WebSocket port (8081) for frontend access.
   b. ClusterIP Service exposing voice adapter port (8082) for ElevenLabs callbacks.
5. Health probes:
   a. Liveness probe on Morgan agent: HTTP GET /healthz on port 8081.
   b. Readiness probe: HTTP GET /readyz (checks MCP tool server connectivity).
   c. Liveness probe on Signal-CLI sidecar: HTTP GET /v1/about on port 8080.
6. Resource limits: Morgan agent 1Gi memory / 1 CPU (adjust based on model inference location — if inference is remote API, lower limits suffice).
7. Cloudflare Tunnel annotation or sidecar for external access to voice/Signal endpoints.

## Validation
Apply manifests to openclaw namespace, verify pod starts with both containers running. Verify PVCs are bound and writable. Verify secrets are mounted as environment variables. Verify liveness probes pass for both containers. Verify ClusterIP services route traffic correctly to Morgan's ports.