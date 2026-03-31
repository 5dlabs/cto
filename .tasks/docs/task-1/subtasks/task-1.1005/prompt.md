Implement subtask 1005: Deploy Signal-CLI as a standalone pod for Morgan agent integration

## Objective
Deploy Signal-CLI as a standalone Deployment in the sigma1 namespace to enable the Morgan AI agent to send and receive Signal messages. Expose it via an internal ClusterIP service with a REST/JSON-RPC interface.

## Steps
1. Create a Deployment manifest for Signal-CLI using the `bbernhard/signal-cli-rest-api` container image (or equivalent) in the `sigma1` namespace.
2. Configure a PersistentVolumeClaim (5Gi) mounted at `/home/.local/share/signal-cli` for Signal account data persistence.
3. Expose the REST API on port 8080 via a ClusterIP Service named `sigma1-signal-cli`.
4. Create an init container or Job that registers/links the Signal account (this will require a phone number — document the manual step for phone number verification).
5. Set resource requests: 256Mi memory, 100m CPU; limits: 512Mi memory, 500m CPU.
6. Add a liveness probe on the `/v1/about` endpoint and a readiness probe on `/v1/health`.
7. Record the service URL (`http://sigma1-signal-cli.sigma1.svc.cluster.local:8080`) for the ConfigMap.
8. Create a Kubernetes Secret `sigma1-signal-config` for any Signal account credentials or trust store data.

## Validation
Verify the Signal-CLI pod is Running with passing health checks. Port-forward to the service and call `GET /v1/about` to confirm the API is responsive. Verify the PVC is bound and mounted. Check pod logs for successful Signal daemon startup with no error loops.