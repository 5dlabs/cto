Implement subtask 1005: Deploy Signal-CLI pod for Morgan agent

## Objective
Deploy Signal-CLI as a standalone pod (or sidecar-ready deployment) in the sigma1 namespace, configured with a registered Signal account for the Morgan agent.

## Steps
1. Create a Deployment manifest for Signal-CLI using the official signal-cli-rest-api image (bbernhard/signal-cli-rest-api or equivalent).
2. Mount a PersistentVolumeClaim for Signal-CLI data directory (~/.local/share/signal-cli) to persist registration state.
3. Expose the Signal-CLI REST API on an internal ClusterIP Service (port 8080).
4. Create a Kubernetes Secret for any Signal account registration data (phone number, verification).
5. Apply the Deployment and Service to the sigma1 namespace.
6. Verify the pod starts and the REST API is reachable.
7. Record the internal service URL (e.g., http://signal-cli.sigma1.svc.cluster.local:8080) for the ConfigMap.

## Validation
Confirm the Signal-CLI pod is Running and Ready; curl the health/about endpoint from a test pod and verify a valid JSON response; confirm the PVC is bound and the ClusterIP Service resolves correctly.